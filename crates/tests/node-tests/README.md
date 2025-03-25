# Node Integration Tests

This crate contains the integration tests for the **node** service.

---

## How to Run

### 1. **Start Required Services**

To run the node integration tests, you must have the following running:

- A **PostgreSQL** database
- A **signer server**
- The **node server** itself

You can run them manually or via Docker Compose.

---

### Manual Setup

#### Required Environment Variables

Make sure the following variables are set **in your terminal** before running the services:

```bash
# Required to run the node server
export GRPC_IP="[::0]" 
export GRPC_PORT=20001
export PG_URL="postgres://postgres:password@localhost:5432/node" // set your pg url
export SIGNER_URL="http://localhost:10000"

# Required to run the signer server
export GRPC_PORT=10000
export ROOT_KEY="tprv8ZgxMBicQKsPeb6rodrmEXb1zRucvxYJgTKDhqQkZtbz8eY4Pf2EgbsT2swBXnnbDPQChQeFrFqHN72yFxzKfFAVsHdPeRWq2xqyUT2c4wH"
```

---

#### Start the Signer Server

```bash
cargo run --release --bin signer
```

---

#### Start the Node Server

```bash
cargo run --release --bin node --no-default-features -- --config ./config/local.toml
```

> The node server requires the environment variables above to run properly.

---

### 🐳 Alternatively: Use Docker Compose

To start everything at once:

```bash
docker-compose -p paynet -f ./docker-compose.yml up -d
```

This will launch PostgreSQL, Signer, and Node with the proper environment variables already set.

---

## 2. Run the Integration Tests

```bash
GRPC_IP="[::0]" GRPC_PORT=20001 cargo test -p node-tests
```

> The tests will wait for the gRPC server to be ready at `http://$GRPC_IP:$GRPC_PORT`.
