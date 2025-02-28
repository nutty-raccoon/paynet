FROM rust:1.85.0 as builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
COPY ./proto/ ./proto/

RUN cargo build --release -p signer

#------------

FROM debian:bookworm-slim

COPY --from=builder ./target/release/signer ./

CMD ["./signer"]
