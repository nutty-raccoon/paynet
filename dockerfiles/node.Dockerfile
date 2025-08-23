FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef

WORKDIR /app

#------------

FROM chef AS planner
COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
RUN cargo chef prepare --recipe-path recipe.json --bin node 

#------------

FROM chef AS builder 

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
ARG CARGO_FEATURES
RUN cargo chef cook --release --recipe-path recipe.json --features "${CARGO_FEATURES}"

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

COPY ./.sqlx/ ./.sqlx/

ARG CARGO_FEATURES

RUN cargo build --release -p node --no-default-features --features "${CARGO_FEATURES}"

#------------

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libsqlite3-0 libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /bin/grpc_health_probe /bin/grpc_health_probe
COPY --from=builder /app/target/release/node /usr/local/bin/node

ENV RUST_LOG=info

# Create an entrypoint script to handle arguments and set STARKNET_INDEXER_START_BLOCK and STARKNET_CHAIN_ID
RUN echo '#!/bin/sh\n\
    # Try to read fork block from shared volume\n\
    if [ -f /shared/fork_block.txt ]; then\n\
    FORK_BLOCK=$(cat /shared/fork_block.txt 2>/dev/null || echo "0")\n\
    export STARKNET_INDEXER_START_BLOCK=$FORK_BLOCK\n\
    echo "Using fork block from volume: $FORK_BLOCK"\n\
    else\n\
    export STARKNET_INDEXER_START_BLOCK=0\n\
    echo "No fork block file found, using default: 0"\n\
    fi\n\
    # Try to read chain ID from shared volume (optional)\n\
    if [ -f /shared/chain_id.txt ]; then\n\
    CHAIN_ID=$(cat /shared/chain_id.txt 2>/dev/null)\n\
    if [ -n "$CHAIN_ID" ]; then\n\
    export STARKNET_CHAIN_ID=$CHAIN_ID\n\
    echo "Using chain ID from volume: $CHAIN_ID"\n\
    else\n\
    echo "chain_id.txt is empty, STARKNET_CHAIN_ID not set"\n\
    fi\n\
    else\n\
    echo "No chain_id.txt found, STARKNET_CHAIN_ID not set"\n\
    fi\n\
    exec /usr/local/bin/node "$@"' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
