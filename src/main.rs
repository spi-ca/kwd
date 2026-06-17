use std::collections::BTreeSet;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
struct KanikoConfig {
    kaniko_bin: String,
    image: String,
    tags: BTreeSet<String>,
}

fn take_trimmed_env(name: &str) -> Option<String> {
    let value = env::var(name).ok()?;
    env::remove_var(name);
    Some(value.trim().to_string())
}

fn is_valid_tag(tag: &str) -> bool {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    tag.len() <= 128
        && (first.is_ascii_alphanumeric() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn parse_tags(value: Option<&str>) -> BTreeSet<String> {
    let mut tags = BTreeSet::new();
    tags.insert("latest".to_string());

    if let Some(value) = value {
        for tag in value.split(',') {
            let tag = tag.trim();
            if !is_valid_tag(tag) {
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

fn destination_args(image: &str, tags: &BTreeSet<String>) -> Vec<String> {
    if image.is_empty() {
        return Vec::new();
    }

    tags.iter()
        .map(|tag| format!("--destination={}:{}", image, tag))
        .collect()
}

fn run() -> Result<(), String> {
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
    Err(format!("failed to execv: {}", err))
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(config.tags, BTreeSet::from(["latest".to_string()]));
        assert_eq!(
            destination_args(&config.image, &config.tags),
            vec!["--destination=repo/example/app:latest".to_string()]
        );
    }

    #[test]
    fn build_image_handles_empty_and_slash_only_inputs() {
        assert_eq!(build_image("", ""), "");
        assert_eq!(build_image("///", "///"), "");
        assert_eq!(build_image("repo/example", ""), "repo/example");
        assert_eq!(build_image("", "app"), "app");
    }

    #[test]
    fn build_image_normalizes_repository_and_name_slashes() {
        assert_eq!(build_image("repo/example/", "/app"), "repo/example/app");
        assert_eq!(
            build_image("repo/example///", "///team/app"),
            "repo/example/team/app"
        );
    }

    #[test]
    fn parse_tags_trims_dedupes_and_skips_invalid_values() {
        let tags = parse_tags(Some(" latest, v1 ,v1,bad/tag,,tag.two, ok_1 "));

        assert_eq!(
            tags,
            BTreeSet::from([
                "latest".to_string(),
                "v1".to_string(),
                "tag.two".to_string(),
                "ok_1".to_string(),
            ])
        );
    }

    #[test]
    fn parse_tags_requires_valid_oci_tag_shape() {
        let too_long = "a".repeat(129);
        let input = format!(".,-,.tag,-tag,_ok,9.ok,A.tag,a-b,c_d,{}", too_long);
        let tags = parse_tags(Some(&input));

        assert_eq!(
            tags,
            BTreeSet::from([
                "9.ok".to_string(),
                "A.tag".to_string(),
                "_ok".to_string(),
                "a-b".to_string(),
                "c_d".to_string(),
                "latest".to_string(),
            ])
        );
    }

    #[test]
    fn parse_tags_accepts_128_byte_tag_and_rejects_longer_tags() {
        let max_len = "a".repeat(128);
        let too_long = "b".repeat(129);
        let input = format!("{},{}", max_len, too_long);
        let tags = parse_tags(Some(&input));

        assert!(tags.contains(&max_len));
        assert!(!tags.contains(&too_long));
    }

    #[test]
    fn destination_args_follow_sorted_tag_order() {
        let tags = parse_tags(Some("z,a,latest,b"));

        assert_eq!(
            destination_args("repo/app", &tags),
            vec![
                "--destination=repo/app:a".to_string(),
                "--destination=repo/app:b".to_string(),
                "--destination=repo/app:latest".to_string(),
                "--destination=repo/app:z".to_string(),
            ]
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
