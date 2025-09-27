# syntax=docker/dockerfile:1.4

FROM rust:alpine AS builder
WORKDIR /app

RUN apk add --no-cache openssl protoc cmake musl-dev curl bash clang
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall sccache wild-linker

RUN echo '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-Clink-arg=--ld-path=wild"]' > /usr/local/cargo/config.toml


ENV RUSTC_WRAPPER="/usr/local/cargo/bin/sccache" \
    SCCACHE_DIR="/sccache" \
    SCCACHE_CACHE_SIZE="10G" \
    CARGO_INCREMENTAL="0"

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY thirdparty ./thirdparty
RUN --mount=type=cache,id=manager-sccache,target=/sccache,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p laval-manager && \
    mkdir -p /binout && \
    cp /app/target/release/laval-manager /binout/

FROM alpine AS runtime

WORKDIR /app

COPY --from=builder /binout/laval-manager /usr/local/bin/laval-manager

ENV RUST_LOG=info
EXPOSE 8080 50051
CMD ["laval-manager"]
