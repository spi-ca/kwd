# syntax=docker/dockerfile:1

##
## Build
##
FROM rust:alpine AS build
LABEL org.opencontainers.image.authors="Sangbum Kim <sangbumkim@amuz.es>"

# set the workdir and build dependencies before copying the full source.
# This keeps dependency artifacts cached when README/docs or application code changes.
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
ENV RUSTFLAGS='-Cpanic=abort -Clink-args=-Wl,-x,-s,--as-needed,--gc-sections,--build-id=none,--no-eh-frame-hdr'
RUN set -x && \
    mkdir -p src/copier && \
    printf 'fn main() {}\n' > src/main.rs && \
    printf 'fn main() {}\n' > src/copier/main.rs && \
    cargo build --release --locked && \
    rm -rf src

COPY src ./src
RUN set -x && \
    rm -rf \
        target/release/.fingerprint/kwd-* \
        target/release/build/kwd-* \
        target/release/deps/kwd-* \
        target/release/deps/copier-* \
        target/release/kwd \
        target/release/copier && \
    cargo build --release --locked --bins

# Previous cache attempt kept for reference:
# RUN set -x && \
#     mkdir -p src/copier && \
#     printf 'fn main() {}\n' > src/main.rs && \
#     printf 'fn main() {}\n' > src/copier/main.rs && \
#     cargo build --release --locked && \
#     rm -rf src
#
# COPY src ./src
# RUN set -x && \
#     cargo build --release --locked

##
## Deploy
##
FROM scratch
COPY --from=build /app/target/release/kwd  /app/target/release/copier /
ENTRYPOINT ["/kwd"]
