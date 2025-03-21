FROM rust:1.85.1 as builder

COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
COPY ./proto/ ./proto/

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN GRPC_HEALTH_PROBE_VERSION=v0.4.13 && \
    wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
    chmod +x /bin/grpc_health_probe

RUN cargo build --release -p starknet-cashier

#------------

FROM debian:bookworm-slim

COPY --from=builder ./target/release/starknet-cashier ./
COPY --from=builder /bin/grpc_health_probe /bin/grpc_health_probe

ENV RUST_LOG=info

CMD ["./starknet-cashier"]
