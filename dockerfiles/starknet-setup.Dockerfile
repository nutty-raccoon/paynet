FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef

WORKDIR app

#------------

FROM chef AS planner
COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
COPY ./mobile/ ./mobile/
RUN cargo chef prepare --recipe-path recipe.json --bin starknet-setup

#------------

FROM chef AS builder 

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --no-default-features

COPY ./Cargo.toml ./
COPY ./crates/ ./crates/

#------------
# Everything up to there is common with node, signer
# which mean common layers, cached together increasing speed.
# What comes next is binary specific.
#------------

RUN cargo build --release -p starknet-on-chain-setup 

# ----------------

FROM rust:1.86.0 as scarb-builder

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Set up architecture detection
WORKDIR /tools
RUN curl -s -L https://github.com/software-mansion/scarb/releases/download/v2.9.2/scarb-v2.9.2-$(uname -m)-unknown-linux-gnu.tar.gz | tar xz -C /tools/ && \
    curl -s -L https://github.com/xJonathanLEI/starkli/releases/download/v0.3.8/starkli-$(uname -m)-unknown-linux-gnu.tar.gz | tar xz -C /tools/

COPY ./contracts/ /contracts/
WORKDIR /contracts/invoice
RUN /tools/scarb-v2.9.2-$(uname -m)-unknown-linux-gnu/bin/scarb --profile release build
RUN /tools/starkli class-hash ./target/release/invoice_payment_InvoicePayment.compiled_contract_class.json > ./compiled_class_hash.txt 

# ----------------

FROM debian as executable

COPY --from=scarb-builder /contracts/invoice/compiled_class_hash.txt /contract/
COPY --from=scarb-builder /contracts/invoice/target/release/invoice_payment_InvoicePayment.contract_class.json /contract/
COPY --from=builder /app/target/release/starknet-on-chain-setup /rust/

WORKDIR /
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'export RUST_LOG=info' >> /entrypoint.sh && \
    echo 'exec "/rust/starknet-on-chain-setup" "$@"  "declare" \
    "--sierra-json=/contract/invoice_payment_InvoicePayment.contract_class.json" \
    "--compiled-class-hash=$(cat /contract/compiled_class_hash.txt)"' \
    >> /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
