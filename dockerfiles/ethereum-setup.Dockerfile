FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef
WORKDIR /app

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
FROM rust:1.86.0 AS forge-builder
RUN apt-get update && apt-get upgrade -y && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
WORKDIR /tools
RUN wget -q https://github.com/foundry-rs/foundry/releases/download/v1.3.1/foundry_v1.3.1_linux_amd64.tar.gz \
    && tar -xzf foundry_v1.3.1_linux_amd64.tar.gz -C /tools \
    && rm foundry_v1.3.1_linux_amd64.tar.gz
COPY ./contracts/ethereum/ /contracts/ethereum/
WORKDIR /contracts/ethereum/invoice
RUN /tools/forge build

# ----------------
FROM debian AS executable

# contract artifacts + binary
COPY --from=forge-builder /contracts/ethereum/invoice/out/InvoicePayment.sol/InvoicePayment.json /contract/
COPY --from=forge-builder /contracts/ethereum/invoice/out/WETH.sol/WETH9.json /contract/
COPY --from=builder /app/target/release/eth-on-chain-setup /rust/

WORKDIR /

# entrypoint: deploy both contracts sequentially
RUN cat > /entrypoint.sh <<'SH'
#!/usr/bin/env bash
set -euo pipefail

export RUST_LOG="${RUST_LOG:-info}"

echo "Deploying InvoicePayment contract..."
/rust/eth-on-chain-setup "$@" deploy \
  --abi-json=/contract/InvoicePayment.json \
  --bytecode-json-or-hex=/contract/InvoicePayment.json

echo "Deploying WETH9 contract..."
/rust/eth-on-chain-setup "$@" deploy \
  --abi-json=/contract/WETH9.json \
  --bytecode-json-or-hex=/contract/WETH9.json

echo "Contracts deployed successfully."
exit 0
SH

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
