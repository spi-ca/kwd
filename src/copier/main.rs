use path_clean::PathClean;
use std::env::args_os;
use std::ffi::{OsStr, OsString};
use std::fs::{File, FileTimes};
use std::path::{Path, PathBuf};
use std::{fs, path};

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

fn get_metadata(path: impl AsRef<Path>) -> Option<(fs::Permissions, fs::FileTimes)> {
    let src_meta = match fs::metadata(path) {
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

    if !dst.exists() {
        if let Some(parent) = dst.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(format!(
                    "failed to create destination directory({}): {}",
                    parent.display(),
                    err
                ));
            }
        }
        Ok(dst)
    } else if dst.is_dir() {
        Ok(dst.join(src_filename))
    } else {
        Ok(dst)
    }
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

    Ok(())
}

fn copy_with_metadata(src: &Path, dst: &Path) -> Result<(), String> {
    let (src_perm, src_tm) = match get_metadata(src) {
        Some((perm, tm)) => (perm, tm),
        None => return Err("failed to read metadata".to_string()),
    };

    if let Err(err) = fs::copy(src, dst) {
        return Err(format!(
            "failed to copy: {} -> {}, {}",
            src.display(),
            dst.display(),
            err
        ));
    }

    let dst_file = match File::open(dst) {
        Ok(file) => file,
        Err(err) => return Err(format!("failed to open file: {}", err)),
    };

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
    } else if !src.exists() {
        panic!("source({}) not exists!", src.display())
    } else if src.is_file() {
        src
    } else if src.is_symlink() {
        match fs::read_link(&src) {
            Ok(src) => normalize_path(Some(src.as_os_str().to_os_string())),
            Err(err) => panic!("failed to read link {}: {}", src.display(), err),
        }
    } else {
        panic!("source({}) is not a file!", src.display())
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

        let dir_err =
            resolve_destination(link_dir.join("out.txt"), OsStr::new("source.txt")).unwrap_err();
        let file_err = resolve_destination(link_file, OsStr::new("source.txt")).unwrap_err();

        assert!(dir_err.contains("symlink component"));
        assert!(file_err.contains("symlink component"));
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
}
