# syntax=docker/dockerfile:1

FROM rust:1.75-slim AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY thirdparty ./thirdparty
RUN cargo build --release -p laval-manager

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/laval-manager /usr/local/bin/laval-manager

ENV RUST_LOG=info
CMD ["laval-manager"]
