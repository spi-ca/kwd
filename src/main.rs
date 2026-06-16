use regex_lite::Regex;
use std::collections::HashSet;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
struct KanikoConfig {
    kaniko_bin: String,
    image: String,
    tags: HashSet<String>,
}

fn take_trimmed_env(name: &str) -> Option<String> {
    let value = env::var(name).ok()?;
    env::remove_var(name);
    Some(value.trim().to_string())
}

fn parse_tags(value: Option<&str>) -> HashSet<String> {
    let mut tags = HashSet::new();
    tags.insert("latest".to_string());

    if let Some(value) = value {
        let pattern = Regex::new(r"^[-a-zA-Z0-9_\.]+$").unwrap();
        for tag in value.split(',') {
            let tag = tag.trim();
            if !pattern.is_match(tag) {
                continue;
            }
            tags.insert(tag.to_string());
        }
    }

    tags
}

fn build_image(repository: &str, name: &str) -> String {
    format!(
        "{}/{}",
        repository.trim_end_matches('/'),
        name.trim_start_matches('/')
    )
    .trim_matches(&['/'] as &[_])
    .to_string()
}

fn build_config(
    kaniko_bin: Option<&str>,
    repository: Option<&str>,
    name: Option<&str>,
    tags: Option<&str>,
) -> KanikoConfig {
    KanikoConfig {
        kaniko_bin: kaniko_bin.unwrap_or("/kaniko/executor").trim().to_string(),
        image: build_image(repository.unwrap_or("").trim(), name.unwrap_or("").trim()),
        tags: parse_tags(tags),
    }
}

fn destination_args(image: &str, tags: &HashSet<String>) -> Vec<String> {
    if image.is_empty() {
        return Vec::new();
    }

    tags.iter()
        .map(|tag| format!("--destination={}:{}", image, tag))
        .collect()
}

fn main() {
    let kaniko_bin = take_trimmed_env("KANIKO_BIN");
    let repository = take_trimmed_env("KANIKO_IMAGE_REPOSITORY");
    let name = take_trimmed_env("KANIKO_IMAGE_NAME");
    let tags = take_trimmed_env("KANIKO_IMAGE_TAGS");

    let config = build_config(
        kaniko_bin.as_deref(),
        repository.as_deref(),
        name.as_deref(),
        tags.as_deref(),
    );

    let mut args: Vec<_> = env::args().skip(1).collect();
    args.extend(destination_args(&config.image, &config.tags));

    let err = Command::new(config.kaniko_bin).args(&args).exec();
    panic!("failed to execv:{}", err);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_destinations(image: &str, tags: &HashSet<String>) -> Vec<String> {
        let mut destinations = destination_args(image, tags);
        destinations.sort();
        destinations
    }

    #[test]
    fn build_config_uses_defaults_and_trims_inputs() {
        let config = build_config(
            Some("  /custom/kaniko  "),
            Some(" repo/example/ "),
            Some(" /app "),
            None,
        );

        assert_eq!(config.kaniko_bin, "/custom/kaniko");
        assert_eq!(config.image, "repo/example/app");
        assert_eq!(config.tags, HashSet::from(["latest".to_string()]));
        assert_eq!(
            sorted_destinations(&config.image, &config.tags),
            vec!["--destination=repo/example/app:latest".to_string()]
        );
    }

    #[test]
    fn parse_tags_trims_dedupes_and_skips_invalid_values() {
        let tags = parse_tags(Some(" latest, v1 ,v1,bad/tag,,tag.two, ok_1 "));

        assert_eq!(
            tags,
            HashSet::from([
                "latest".to_string(),
                "v1".to_string(),
                "tag.two".to_string(),
                "ok_1".to_string(),
            ])
        );
    }

    #[test]
    fn destination_args_are_empty_for_empty_image_after_slash_trimming() {
        let image = build_image("///", "///");
        let tags = parse_tags(Some("stable"));

        assert!(image.is_empty());
        assert!(destination_args(&image, &tags).is_empty());
    }
}
