# Paynet Web App

A Rust web server for handling Starknet deposit routes with a modern frontend build pipeline.

## HTTPS Configuration

To run this app with feature `tls` enabled use `mkcert` (https://github.com/FiloSottile/mkcert) for local development.

```shell
mkcert -install
mkdir -p certs
mkcert -key-file certs/key.pem -cert-file certs/cert.pem localhost 127.0.0.1 ::1
```

## Quick Start

### Prerequisites

- Node.js 18+ 
- pnpm 8+
- Rust (for backend)

### Installation

```bash
# Install frontend dependencies
pnpm install

# Build frontend for production
pnpm run build

# Or run in development mode (with file watching)
pnpm run dev

# Run the webserver
cargo run -p web-app
```