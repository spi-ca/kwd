---
name: "run-kwd-smoke-checks"
description: "Run and record focused smoke checks for kwd's kaniko wrapper behavior in a disposable workspace or container image"
version: 2
created: "2026-06-14"
updated: "2026-06-17"
---

# KWD Kaniko Wrapper Smoke Checks

## When to Use

Use when changes affect `kwd` argument construction, `KANIKO_*` environment handling, `KANIKO_BIN` resolution, destination tag expansion, or operator-facing prompts/docs for those behaviors.

## Procedure

1. Build the release binary with `cargo build --release` and use `target/release/kwd` as the artifact.
2. Create a disposable temporary directory with a fake kaniko executor script that records its argv and selected environment to files, then exits successfully. Point `KANIKO_BIN` to that script.
3. Run `kwd` with representative inputs:
   - `KANIKO_IMAGE_REPOSITORY`
   - `KANIKO_IMAGE_NAME`
   - `KANIKO_IMAGE_TAGS`
   - passthrough CLI args such as build context or Dockerfile flags
4. Inspect the recorded argv and confirm that passthrough CLI args were preserved and that `kwd` added one `--destination=<repo>/<name>:<tag>` argument for every valid tag plus the default `latest` tag.
5. Re-run with duplicate, blank, and invalid tags to confirm invalid values are ignored, duplicates collapse safely, and execution still reaches the fake executor.
6. If the change affects packaging or runtime assumptions, repeat the check inside a disposable container image or CI-like environment that matches the expected kaniko filesystem layout, including the default `/kaniko/executor` path and mounted registry credentials.
7. When relevant, inspect the fake executor's environment capture and confirm that control variables such as `KANIKO_BIN`, `KANIKO_IMAGE_REPOSITORY`, `KANIKO_IMAGE_NAME`, and `KANIKO_IMAGE_TAGS` were not forwarded unexpectedly.

## Pitfalls

- Do not depend on destination argument ordering; the current implementation builds tags from a set.
- Do not log real registry credentials or other secrets while capturing executor environment.
- Prefer a fake executor for smoke checks. Use a real kaniko binary only in a disposable environment with safe registry settings.

## Verification

1. `cargo build --release` succeeds and produces `target/release/kwd`.
2. The fake executor capture shows the expected passthrough args and computed `--destination=` flags.
3. A tag edge-case run confirms invalid tags are skipped and duplicates do not create duplicate destinations.
4. If environment capture is used, the forwarded environment does not unexpectedly include the `KANIKO_*` control variables removed by `kwd` before `exec`.
5. If a container-image smoke check is run, the expected `/kaniko/executor` path and mounted auth/config inputs are present in that disposable environment.
