FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef

WORKDIR app

#------------

FROM chef AS planner
COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
RUN cargo chef prepare --recipe-path recipe.json --bin signer

#------------

FROM chef AS builder 

RUN apt-get update && apt-get install -y protobuf-compiler libgtk-3-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libatk1.0-dev libwebkit2gtk-4.1-dev   && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

RUN GRPC_HEALTH_PROBE_VERSION=v0.4.13 && \
    ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then \
    PROBE_ARCH="amd64"; \
    elif [ "$ARCH" = "aarch64" ]; then \
    PROBE_ARCH="arm64"; \
    else \
    echo "Unsupported architecture: $ARCH" && exit 1; \
    fi && \
    wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-${PROBE_ARCH} && \
    chmod +x /bin/grpc_health_probe

COPY ./Cargo.toml ./
COPY ./proto/ ./proto/
COPY ./crates/ ./crates/

RUN cargo build --release -p signer

#------------

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /bin/grpc_health_probe /bin/grpc_health_probe
COPY --from=builder /app/target/release/signer /usr/local/bin/signer

ENV RUST_LOG=info

CMD ["/usr/local/bin/signer"]
