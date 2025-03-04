# kwd [![Crates.io][crates-badge]][crates-url] [![hub.docker.com][docker-badge]][docker-url] [![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/kwd.svg
[crates-url]: https://crates.io/crates/kwd
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/spi-ca/kwd/blob/main/LICENSE
[docker-badge]: https://img.shields.io/docker/v/sangbumkim/kwd
[docker-url]: https://hub.docker.com/r/sangbumkim/kwd

## Description

This is a tool that performs destination tasks when attaching multiple tags to a single kaniko image build.


## Configuration

kwd is basically a binary wrapper for kaniko.

When running kwd, it reads the environment variables specified below and passes the corresponding arguments to kaniko.
If execution arguments are given to kwd, these are passed transparently to the kaniko process.

| Environment variables              | Default Value                    | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                |
|------------------------------------|----------------------------------| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `KANIKO_BIN`                       |  `/kaniko/executor`                          | Specifies the path to `kaniko` binary. By default, the path used by `gcr.io/kaniko-project/executor` is used.                                                                                                                                                                                                                                 |
| `KANIKO_IMAGE_REPOSITORY`          |                                  | Enable TLS or not. Delete the `ssl-redirect` annotations in `expose.ingress.annotations` when TLS is disabled and `expose.type` is `ingress`. Note: if the `expose.type` is `ingress` and TLS is disabled, the port must be included in the command when pulling/pushing images. Refer to https://github.com/goharbor/harbor/issues/5291 for details.                                                                                                      |
| `KANIKO_IMAGE_NAME`                |                                  | The source of the TLS certificate. Set as `auto`, `secret` or `none` and fill the information in the corresponding section: 1) auto: generate the TLS certificate automatically 2) secret: read the TLS certificate from the specified secret. The TLS certificate can be generated manually or by cert manager 3) none: configure no TLS certificate for the ingress. If the default TLS certificate is configured in the ingress controller, choose this option|
| `KANIKO_IMAGE_TAGS`                |                                  | The common name used to generate the certificate, it's necessary when the type isn't `ingress`                                                                                                                                                                                                                                                                                                                                                             |

