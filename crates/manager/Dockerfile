FROM rust:alpine AS builder
WORKDIR /app

RUN apk add --no-cache openssl protoc cmake musl-dev

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY thirdparty ./thirdparty
RUN cargo build --release -p laval-manager

FROM alpine AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/laval-manager /usr/local/bin/laval-manager

ENV RUST_LOG=info
EXPOSE 8080 50051
CMD ["laval-manager"]
