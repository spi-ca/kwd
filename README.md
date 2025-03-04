# kwd [![Crates.io][crates-badge]][crates-url] [![hub.docker.com][docker-badge]][docker-url] [![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/kwd.svg

[crates-url]: https://crates.io/crates/kwd

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg

[mit-url]: https://github.com/spi-ca/kwd/blob/main/LICENSE

[docker-badge]: https://img.shields.io/docker/v/sangbumkim/kwd

[docker-url]: https://hub.docker.com/r/sangbumkim/kwd

## Description

Attaching multiple tags to a single kaniko image build.

## Configuration

kwd is basically a binary wrapper for kaniko.

When running kwd, it reads the environment variables specified below and passes the corresponding arguments to kaniko.
If execution arguments are given to kwd, these are passed transparently to the kaniko process.

| Environment variables     | Required | Default Value      | Description                                                                                                   |
|---------------------------|----------|--------------------|---------------------------------------------------------------------------------------------------------------|
| `KANIKO_BIN`              | no       | `/kaniko/executor` | Specifies the path to `kaniko` binary. By default, the path used by `gcr.io/kaniko-project/executor` is used. |
| `KANIKO_IMAGE_REPOSITORY` | yes      |                    | Indicates the repository prefix to be used for Docker image push. e.g. `gcr.io/google-containers`             |
| `KANIKO_IMAGE_NAME`       | yes      |                    | Indicates the repository name to be used for Docker image push. e.g. `pause`                                  |
| `KANIKO_IMAGE_TAGS`       | yes      |                    | Specifies a comma-separated list of container image tags. e.g. `latest,v1.0.0,v1.0.0-1`                       |

