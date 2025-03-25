FROM rust:1.85.0 as scarb-builder

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

WORKDIR /tools
RUN curl -s -L https://github.com/software-mansion/scarb/releases/download/v2.9.2/scarb-v2.9.2-aarch64-unknown-linux-gnu.tar.gz | tar xvz -C /tools/
RUN curl -s -L https://github.com/xJonathanLEI/starkli/releases/download/v0.3.8/starkli-aarch64-unknown-linux-gnu.tar.gz | tar xvz -C /tools/

COPY ./contracts/ /contracts/
WORKDIR /contracts/invoice
RUN /tools/scarb-v2.9.2-aarch64-unknown-linux-gnu/bin/scarb --profile release build
RUN /tools/starkli class-hash ./target/release/invoice_payment_InvoicePayment.compiled_contract_class.json > ./compiled_class_hash.txt 

# ----------------

FROM rust:1.85.0 as rust-builder

COPY ./Cargo.toml ./rust/
COPY ./crates/ ./rust/crates/

WORKDIR /rust
RUN cargo build --release -p starknet-on-chain-setup 

# ----------------

FROM debian as executable

COPY --from=scarb-builder /contracts/invoice/compiled_class_hash.txt /contract/
COPY --from=scarb-builder /contracts/invoice/target/release/invoice_payment_InvoicePayment.contract_class.json /contract/
COPY --from=rust-builder /rust/target/release/starknet-on-chain-setup /rust/

WORKDIR /
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'export RUST_LOG=info' >> /entrypoint.sh && \
    echo 'exec "/rust/starknet-on-chain-setup" "declare" "$@" \
    "--sierra-json=/contract/invoice_payment_InvoicePayment.contract_class.json" \
    "--compiled-class-hash=$(cat /contract/compiled_class_hash.txt)"' \
    >> /entrypoint.sh && chmod +x /entrypoint.sh


ENTRYPOINT ["/entrypoint.sh"]
