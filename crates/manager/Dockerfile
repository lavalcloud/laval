# syntax=docker/dockerfile:1.4

FROM rust:alpine AS builder
WORKDIR /app

RUN apk add --no-cache openssl protoc cmake musl-dev

RUN cargo install --locked sccache

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
    cargo build --release -p laval-manager

FROM alpine AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/laval-manager /usr/local/bin/laval-manager

ENV RUST_LOG=info
EXPOSE 8080 50051
CMD ["laval-manager"]
