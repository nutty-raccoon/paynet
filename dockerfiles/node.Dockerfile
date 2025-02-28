FROM rust:1.85.0 as builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
COPY ./proto/ ./proto/
COPY ./.sqlx/ ./.sqlx/

RUN cargo build --release -p node

#------------

FROM debian:bookworm-slim

COPY --from=builder ./target/release/node ./
COPY --from=builder ./crates/bin/node/config/local.toml ./config.toml

CMD ["./node", "--config", "./config.toml"]
