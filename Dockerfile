# syntax=docker/dockerfile:1

##
## Build
##
FROM rust:alpine AS build
LABEL org.opencontainers.image.authors="Sangbum Kim <sangbumkim@amuz.es>"

# set the workdir and copy the source into it
WORKDIR /app
COPY . /app
ENV RUSTFLAGS='-Cpanic=abort -Clink-args=-Wl,-x,-s,--as-needed,--gc-sections,--build-id=none,--no-eh-frame-hdr'
RUN set -x && \
    cargo build --release

##
## Deploy
##
FROM scratch
COPY --from=build /app/target/release/kwd  /app/target/release/copier /
ENTRYPOINT ["/kwd"]
