use path_clean::PathClean;
use std::env::args_os;
use std::ffi::OsString;
use std::fs::{File, FileTimes};
use std::path::{Path, PathBuf};
use std::{fs, path};

fn normalize_path(path_opt: Option<OsString>) -> PathBuf {
    let path = match path_opt {
        Some(path) => PathBuf::from(path),
        None => panic!("path not provided"),
    };
    let path = match path::absolute(&path) {
        Ok(path) => path.clean(),
        Err(err) => panic!("failed to make absolute path({}): {}", path.display(), err),
    };

    path.clean()
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

    let tm = FileTimes::new();
    tm.set_modified(modified);
    tm.set_accessed(accessed);

    Some((perm, tm))
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

    let (src_perm, src_tm) = match get_metadata(&src) {
        Some((perm, tm)) => (perm, tm),
        None => panic!("failed to read metadata"),
    };

    let dst = if !dst.exists() {
        if let Some(parent) = &dst.parent() {
            if let Err(err) = fs::create_dir_all(&parent) {
                panic!(
                    "failed to create destination directory({}): {}",
                    parent.display(),
                    err
                );
            }
        }
        dst
    } else if !dst.is_dir() {
        dst.join(src_filename)
    } else {
        dst
    };

    eprint!("{} -> {}: ", src.display(),dst.display());

    if let Err(err) = fs::copy(&src, &dst) {
        panic!(
            "failed to copy: {} -> {}, {}",
            &src.display(),
            &dst.display(),
            err
        );
    }

    let dst_file = match File::open(dst) {
        Ok(file) => file,
        Err(err) => panic!("failed to open file: {}", err),
    };

    if let Err(err) = dst_file.set_permissions(src_perm) {
        panic!("failed to set permissions: {}", err);
    }
    if let Err(err) = dst_file.set_times(src_tm) {
        panic!("failed to set file times: {}", err);
    }

    eprintln!("OK")
}
