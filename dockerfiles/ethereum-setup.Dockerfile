FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef

WORKDIR app

#------------

FROM chef AS planner
COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
RUN cargo chef prepare --recipe-path recipe.json --bin eth-on-chain-setup

#------------

FROM chef AS builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY ./Cargo.toml ./
COPY ./crates/ ./crates/

RUN cargo build --release -p eth-on-chain-setup

# ----------------                            

FROM rust:1.86.0 as forge-builder

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Set up architecture detection
WORKDIR /tools
RUN curl -s -L https://github.com/foundry-rs/foundry/releases/download/v1.3.0/foundry_v1.3.0_linux_amd64.tar.gz | tar xz -C /tools/

COPY ./contracts/ethereum/ /contracts/ethereum/
WORKDIR /contracts/ethereum/invoice
RUN /tools/foundry_v1.3.0_linux_amd64/forge build

# ----------------

FROM debian as executable

COPY --from=forge-builder /contracts/ethereum/invoice/out/InvoicePayment.sol/InvoicePayment.json /contract/
COPY --from=forge-builder /contracts/ethereum/invoice/out/WETH.sol/WETH9.json /contract/
COPY --from=builder /app/target/release/eth-on-chain-setup /rust/
                               
WORKDIR /
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'set -e' >> /entrypoint.sh && \
    echo 'export RUST_LOG=info' >> /entrypoint.sh && \
    echo 'echo "Deploying InvoicePayment contract..."' >> /entrypoint.sh && \
    echo 'exec "/rust/eth-on-chain-setup" "$@" "deploy" \
    "--abi_json=/contract/InvoicePayment.json" \
    "--bytecode_json_or_hex=/contract/InvoicePayment.json"' >> /entrypoint.sh && \
    echo 'echo "Deploying WETH9 contract..."' >> /entrypoint.sh && \
    echo 'exec "/rust/eth-on-chain-setup" "$@" "deploy" \
    "--abi_json=/contract/WETH9.json" \
    "--bytecode_json_or_hex=/contract/WETH9.json"' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
