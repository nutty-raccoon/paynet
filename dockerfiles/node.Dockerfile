FROM rust:1.86.0 as builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

ARG CARGO_FEATURES="starknet"

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
COPY ./crates/ ./crates/
COPY ./proto/ ./proto/
COPY ./.sqlx/ ./.sqlx/

RUN cargo build --release -p node --no-default-features --features=${CARGO_FEATURES}

#------------

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libsqlite3-0 libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /bin/grpc_health_probe /bin/grpc_health_probe
COPY --from=builder ./target/release/node /usr/local/bin/node

ENV RUST_LOG=info

# Create an entrypoint script to handle arguments
RUN echo '#!/bin/sh\nexec /usr/local/bin/node "$@"' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
