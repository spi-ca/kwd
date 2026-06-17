use path_clean::PathClean;
use std::env::args_os;
use std::ffi::{c_char, CString, OsStr, OsString};
use std::fs::{File, FileTimes};
use std::io;
use std::os::fd::FromRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::{fs, path};

const O_RDONLY: i32 = 0;
const O_WRONLY: i32 = 1;
const O_CREAT: i32 = 0o100;
const O_TRUNC: i32 = 0o1000;
const O_DIRECTORY: i32 = 0o200000;
const O_NOFOLLOW: i32 = 0o400000;
const O_CLOEXEC: i32 = 0o2000000;
const O_PATH: i32 = 0o10000000;

unsafe extern "C" {
    fn openat(dirfd: i32, pathname: *const c_char, flags: i32, mode: u32) -> i32;
}

fn normalize_path_result(path_opt: Option<OsString>) -> Result<PathBuf, String> {
    let path = match path_opt {
        Some(path) => PathBuf::from(path),
        None => return Err("path not provided".to_string()),
    };
    let path = match path::absolute(&path) {
        Ok(path) => path.clean(),
        Err(err) => {
            return Err(format!(
                "failed to make absolute path({}): {}",
                path.display(),
                err
            ))
        }
    };

    Ok(path.clean())
}

fn normalize_path(path_opt: Option<OsString>) -> PathBuf {
    match normalize_path_result(path_opt) {
        Ok(path) => path,
        Err(err) => panic!("{}", err),
    }
}

fn get_metadata(file: &File) -> Option<(fs::Permissions, fs::FileTimes)> {
    let src_meta = match file.metadata() {
        Ok(meta) => meta,
        Err(_) => return None,
    };

    let perm = src_meta.permissions();

    let modified = match src_meta.modified() {
        Ok(at) => at,
        Err(_) => return None,
    };

    let accessed = match src_meta.accessed() {
        Ok(at) => at,
        Err(_) => return None,
    };

    let tm = FileTimes::new()
        .set_modified(modified)
        .set_accessed(accessed);

    Some((perm, tm))
}

fn has_existing_symlink_component(path: &Path) -> bool {
    path.ancestors().any(|ancestor| {
        fs::symlink_metadata(ancestor)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
    })
}

fn resolve_destination(dst: PathBuf, src_filename: &OsStr) -> Result<PathBuf, String> {
    if has_existing_symlink_component(&dst) {
        return Err(format!(
            "destination path contains a symlink component: {}",
            dst.display()
        ));
    }

    let resolved = if !dst.exists() {
        if let Some(parent) = dst.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(format!(
                    "failed to create destination directory({}): {}",
                    parent.display(),
                    err
                ));
            }
        }
        dst
    } else if dst.is_dir() {
        dst.join(src_filename)
    } else {
        dst
    };

    if has_existing_symlink_component(&resolved) {
        return Err(format!(
            "destination path contains a symlink component: {}",
            resolved.display()
        ));
    }

    Ok(resolved)
}

fn resolve_source(src: PathBuf) -> Result<PathBuf, String> {
    if let Ok(metadata) = fs::symlink_metadata(&src) {
        if metadata.file_type().is_symlink() {
            return Err(format!("source({}) is a symlink!", src.display()));
        }
    }

    if !src.exists() {
        Err(format!("source({}) not exists!", src.display()))
    } else if src.is_file() {
        Ok(src)
    } else {
        Err(format!("source({}) is not a file!", src.display()))
    }
}

fn path_component_cstring(component: &OsStr) -> io::Result<CString> {
    CString::new(component.as_bytes()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "path component contains an interior NUL byte",
        )
    })
}

fn open_no_symlink_at(dirfd: i32, name: &OsStr, flags: i32, mode: u32) -> io::Result<File> {
    let name = path_component_cstring(name)?;
    let fd = unsafe { openat(dirfd, name.as_ptr(), flags | O_NOFOLLOW | O_CLOEXEC, mode) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn open_path_without_symlinks(path: &Path, flags: i32, mode: u32) -> io::Result<File> {
    let mut components = path.components().peekable();
    let mut dir = match components.next() {
        Some(Component::RootDir) => {
            open_no_symlink_at(-100, OsStr::new("/"), O_PATH | O_DIRECTORY, 0)?
        }
        Some(Component::Normal(first)) => {
            if components.peek().is_none() {
                return open_no_symlink_at(-100, first, flags, mode);
            }
            open_no_symlink_at(-100, first, O_PATH | O_DIRECTORY, 0)?
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path must be absolute or contain normal components",
            ))
        }
    };

    while let Some(component) = components.next() {
        let name = match component {
            Component::Normal(name) => name,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "path contains unsupported component",
                ))
            }
        };

        let is_last = components.peek().is_none();
        let file = if is_last {
            open_no_symlink_at(dir_fd(&dir), name, flags, mode)?
        } else {
            open_no_symlink_at(dir_fd(&dir), name, O_PATH | O_DIRECTORY, 0)?
        };
        dir = file;
    }

    Ok(dir)
}

fn dir_fd(file: &File) -> i32 {
    use std::os::fd::AsRawFd;
    file.as_raw_fd()
}

fn ensure_not_same_file(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        return Ok(());
    }

    let canonical_src = src
        .canonicalize()
        .map_err(|err| format!("failed to canonicalize source({}): {}", src.display(), err))?;
    let canonical_dst = dst.canonicalize().map_err(|err| {
        format!(
            "failed to canonicalize destination({}): {}",
            dst.display(),
            err
        )
    })?;

    if canonical_src == canonical_dst {
        return Err("source and destination is same!".to_string());
    }

    let src_meta = fs::metadata(src)
        .map_err(|err| format!("failed to stat source({}): {}", src.display(), err))?;
    let dst_meta = fs::metadata(dst)
        .map_err(|err| format!("failed to stat destination({}): {}", dst.display(), err))?;
    if src_meta.dev() == dst_meta.dev() && src_meta.ino() == dst_meta.ino() {
        return Err("source and destination is same!".to_string());
    }

    Ok(())
}

fn copy_with_metadata(src: &Path, dst: &Path) -> Result<(), String> {
    let mut src_file = open_path_without_symlinks(src, O_RDONLY, 0)
        .map_err(|err| format!("failed to open source without following symlinks: {}", err))?;

    let (src_perm, src_tm) = match get_metadata(&src_file) {
        Some((perm, tm)) => (perm, tm),
        None => return Err("failed to read metadata".to_string()),
    };

    let mut dst_file = open_path_without_symlinks(dst, O_WRONLY | O_CREAT | O_TRUNC, 0o666)
        .map_err(|err| {
            format!(
                "failed to open destination without following symlinks: {}",
                err
            )
        })?;

    if let Err(err) = io::copy(&mut src_file, &mut dst_file) {
        return Err(format!(
            "failed to copy: {} -> {}, {}",
            src.display(),
            dst.display(),
            err
        ));
    }

    if let Err(err) = dst_file.set_permissions(src_perm) {
        return Err(format!("failed to set permissions: {}", err));
    }
    if let Err(err) = dst_file.set_times(src_tm) {
        return Err(format!("failed to set file times: {}", err));
    }

    Ok(())
}

fn main() {
    let mut args = args_os().skip(1);

    let src = normalize_path(args.next());
    let dst = normalize_path(args.next());

    let src = if src.eq(&dst) {
        panic!("source and destination is same!")
    } else {
        match resolve_source(src) {
            Ok(src) => src,
            Err(err) => panic!("{}", err),
        }
    };

    let src_filename = match src.file_name() {
        None => panic!("source({}) doesn't a filename!", src.display()),
        Some(name) => name,
    };

    let dst = match resolve_destination(dst, src_filename) {
        Ok(dst) => dst,
        Err(err) => panic!("{}", err),
    };

    eprint!("{} -> {}: ", src.display(), dst.display());

    if let Err(err) = ensure_not_same_file(&src, &dst) {
        panic!("{}", err);
    }

    if let Err(err) = copy_with_metadata(&src, &dst) {
        panic!("{}", err);
    }

    eprintln!("OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::process;
    use std::time::{Duration, SystemTime};

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "kwd-copier-tests-{}-{}-{}",
            process::id(),
            name,
            unique
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn normalize_path_result_requires_path() {
        assert_eq!(
            normalize_path_result(None),
            Err("path not provided".to_string())
        );
    }

    #[test]
    fn resolve_source_rejects_symlink_sources() {
        let dir = test_dir("source-symlink");
        let real_file = dir.join("real.txt");
        let link_file = dir.join("link.txt");
        fs::write(&real_file, b"hello").unwrap();
        symlink(&real_file, &link_file).unwrap();

        let err = resolve_source(link_file).unwrap_err();

        assert!(err.contains("is a symlink"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn resolve_destination_creates_missing_parent_and_keeps_missing_path() {
        let dir = test_dir("resolve-missing");
        let dst = dir.join("nested/output.txt");

        let resolved = resolve_destination(dst.clone(), OsStr::new("source.txt")).unwrap();

        assert_eq!(resolved, dst);
        assert!(dir.join("nested").is_dir());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn resolve_destination_rejects_existing_symlink_components() {
        let dir = test_dir("resolve-symlink");
        let real_dir = dir.join("real-dir");
        let link_dir = dir.join("link-dir");
        let link_file = dir.join("link-file");
        fs::create_dir_all(&real_dir).unwrap();
        symlink(&real_dir, &link_dir).unwrap();
        symlink(real_dir.join("out.txt"), &link_file).unwrap();
        symlink(real_dir.join("target.txt"), dir.join("source.txt")).unwrap();

        let dir_err =
            resolve_destination(link_dir.join("out.txt"), OsStr::new("source.txt")).unwrap_err();
        let file_err = resolve_destination(link_file, OsStr::new("source.txt")).unwrap_err();
        let final_name_err =
            resolve_destination(dir.clone(), OsStr::new("source.txt")).unwrap_err();

        assert!(dir_err.contains("symlink component"));
        assert!(file_err.contains("symlink component"));
        assert!(final_name_err.contains("symlink component"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn resolve_destination_handles_existing_file_and_directory_paths() {
        let dir = test_dir("resolve-existing");
        let existing_file = dir.join("existing.txt");
        let existing_dir = dir.join("existing-dir");
        fs::write(&existing_file, b"dst").unwrap();
        fs::create_dir_all(&existing_dir).unwrap();

        let resolved_file =
            resolve_destination(existing_file.clone(), OsStr::new("source.txt")).unwrap();
        let resolved_dir =
            resolve_destination(existing_dir.clone(), OsStr::new("source.txt")).unwrap();

        assert_eq!(resolved_file, existing_file);
        assert_eq!(resolved_dir, existing_dir.join("source.txt"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn copy_with_metadata_preserves_permissions_and_file_times() {
        let dir = test_dir("copy-metadata");
        let src = dir.join("source.txt");
        let dst = dir.join("out/copied.txt");
        fs::write(&src, b"hello").unwrap();

        let src_file = File::options().write(true).open(&src).unwrap();
        let expected_accessed = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let expected_modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_123);
        let src_times = FileTimes::new()
            .set_accessed(expected_accessed)
            .set_modified(expected_modified);
        src_file.set_times(src_times).unwrap();
        fs::set_permissions(&src, fs::Permissions::from_mode(0o640)).unwrap();

        let resolved_dst = resolve_destination(dst.clone(), OsStr::new("source.txt")).unwrap();
        copy_with_metadata(&src, &resolved_dst).unwrap();

        let src_meta = fs::metadata(&src).unwrap();
        let dst_meta = fs::metadata(&resolved_dst).unwrap();
        assert_eq!(
            dst_meta.permissions().mode() & 0o777,
            src_meta.permissions().mode() & 0o777
        );
        assert_eq!(dst_meta.modified().unwrap(), src_meta.modified().unwrap());
        assert_eq!(dst_meta.accessed().unwrap(), expected_accessed);
        assert_eq!(fs::read(&resolved_dst).unwrap(), b"hello");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ensure_not_same_file_rejects_existing_destination_alias() {
        let dir = test_dir("same-file");
        let src = dir.join("source.txt");
        fs::write(&src, b"hello").unwrap();
        let alias = dir.join(".").join("source.txt");

        assert_eq!(
            ensure_not_same_file(&src, &alias),
            Err("source and destination is same!".to_string())
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ensure_not_same_file_rejects_hard_link_destination() {
        let dir = test_dir("same-file-hard-link");
        let src = dir.join("source.txt");
        let hard_link = dir.join("hard-link.txt");
        fs::write(&src, b"hello").unwrap();
        fs::hard_link(&src, &hard_link).unwrap();

        assert_eq!(
            ensure_not_same_file(&src, &hard_link),
            Err("source and destination is same!".to_string())
        );
        let _ = fs::remove_dir_all(dir);
    }
}
