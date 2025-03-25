FROM rust:1.85.0 as builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN GRPC_HEALTH_PROBE_VERSION=v0.4.13 && \
    wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
    chmod +x /bin/grpc_health_probe

COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
COPY ./proto/ ./proto/
COPY ./.sqlx/ ./.sqlx/

RUN cargo build --release -p node

#------------

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libsqlite3-0 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /bin/grpc_health_probe /bin/grpc_health_probe
COPY --from=builder ./target/release/node ./
COPY --from=builder ./crates/bin/node/config/local.toml ./config.toml

ENV RUST_LOG=info

CMD ["./node", "--config", "./config.toml"]
