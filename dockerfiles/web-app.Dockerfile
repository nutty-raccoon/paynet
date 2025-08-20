FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef

WORKDIR /app

#------------

FROM chef AS planner
COPY ./Cargo.toml ./
COPY ./crates/ ./crates/
RUN cargo chef prepare --recipe-path recipe.json --bin web-app

#------------

FROM chef AS rust-builder

RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --features="tls"

COPY ./Cargo.toml ./
COPY ./rust-toolchain.toml ./
COPY ./crates/ ./crates/

RUN cargo build --release --bin web-app --features="tls"

#------------
 
FROM node:18-alpine AS frontend-builder

RUN npm install -g pnpm@8

WORKDIR /app/frontend

COPY crates/bins/web-app/package.json crates/bins/web-app/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY crates/bins/web-app/frontend ./frontend
COPY crates/bins/web-app/webpack.config.js ./
RUN pnpm run build

#------------

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false appuser

WORKDIR /app

COPY --from=rust-builder /app/target/release/web-app ./web-app
COPY --from=rust-builder /app/crates/bins/web-app/templates ./crates/bins/web-app/templates
COPY --from=rust-builder /app/crates/bins/web-app/static ./crates/bins/web-app/static
COPY --from=frontend-builder /app/frontend/static/dist ./crates/bins/web-app/static/dist

RUN chown -R appuser:appuser /app

USER appuser

ENV PORT=3005
ENV CERT_PATH=/certs/cert.pem
ENV KEY_PATH=/certs/key.pem

EXPOSE ${PORT}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
     CMD curl -f -k https://localhost:${PORT}/ || exit 1

CMD ["./web-app"]
