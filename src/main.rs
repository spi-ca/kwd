use regex_lite::Regex;
use std::collections::HashSet;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    let mut kaniko_bin = "/kaniko/executor".to_string();
    if let Ok(value) = env::var("KANIKO_BIN") {
        env::remove_var("KANIKO_BIN");
        kaniko_bin = value.trim().to_string();
    }

    let mut repository = "".to_string();
    if let Ok(value) = env::var("KANIKO_IMAGE_REPOSITORY") {
        env::remove_var("KANIKO_IMAGE_REPOSITORY");
        repository = value.trim().to_string()
    }

    let mut name = "".to_string();
    if let Ok(value) = env::var("KANIKO_IMAGE_NAME") {
        env::remove_var("KANIKO_IMAGE_NAME");
        name = value.trim().to_string();
    }

    let mut tags: HashSet<String> = HashSet::new();

    tags.insert("latest".to_string());

    if let Ok(value) = env::var("KANIKO_IMAGE_TAGS") {
        env::remove_var("KANIKO_IMAGE_TAGS");
        let pattern = Regex::new(r"^[-a-zA-Z0-9_\.]+$").unwrap();
        for tag in value.split(',') {
            let tag = tag.trim();
            if !pattern.is_match(tag) {
                continue;
            }
            tags.insert(tag.to_string());
        }
    }

    let image = format!(
        "{}/{}",
        repository.trim_end_matches("/"),
        name.trim_start_matches("/")
    )
    .trim_matches(&['/'] as &[_])
    .to_string();

    let mut args: Vec<_> = env::args().skip(1).collect();
    if !image.is_empty() {
        for tag in tags {
            args.push(format!("--destination={}:{}", image, tag));
        }
    }

    let err = Command::new(kaniko_bin).args(&args).exec();
    panic!("failed to execv:{}", err);
}
