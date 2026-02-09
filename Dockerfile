FROM rust:1.91-bookworm AS builder
RUN apt-get update && apt-get install -y cmake perl pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release --bin gqlforge

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gqlforge /usr/local/bin/gqlforge
ENTRYPOINT ["gqlforge"]
