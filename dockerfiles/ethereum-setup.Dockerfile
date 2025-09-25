# syntax=docker/dockerfile:1.4
FROM ghcr.io/foundry-rs/foundry:latest AS contract-builder

WORKDIR /contracts/ethereum/invoice

# Initialize git with proper configuration
RUN git init && \
    git config --global user.email "ci@example.com" && \
    git config --global user.name "CI" && \
    git config --global init.defaultBranch main

# Copy dependency files first for better layer caching
COPY ./contracts/ethereum/invoice/foundry.toml ./
COPY ./contracts/ethereum/invoice/remappings.txt ./

# Install dependencies (this layer will be cached if dependencies don't change)
RUN forge install foundry-rs/forge-std && \
    forge install openzeppelin/openzeppelin-contracts

# Copy source files after dependencies
COPY ./contracts/ethereum/invoice/src ./src/
COPY ./contracts/ethereum/invoice/script ./script/
COPY ./contracts/ethereum/invoice/test ./test/

# Build contracts with optimization and proper error handling
RUN forge build --optimize && \
    forge test --no-match-test ".*fork.*" # Skip fork tests in build

# ----------------

FROM ghcr.io/foundry-rs/foundry:latest AS runtime

# Install required system packages
USER root
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        jq \
        netcat-openbsd \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

USER foundry
WORKDIR /home/foundry

# Copy built contracts from builder stage
COPY --from=contract-builder --chown=foundry:foundry /contracts/ethereum/invoice/ ./contract/

# Create deployment script with better error handling and logging
COPY --chown=foundry:foundry <<'EOF' /home/foundry/entrypoint.sh
#!/bin/bash
set -euo pipefail

# Configuration
ANVIL_HOST=${ANVIL_HOST:-0.0.0.0}
ANVIL_PORT=${ANVIL_PORT:-8545}
CHAIN_ID=${CHAIN_ID:-1337}
BLOCK_TIME=${BLOCK_TIME:-2}
ACCOUNTS=${ACCOUNTS:-10}
BALANCE=${BALANCE:-10000}
GAS_LIMIT=${GAS_LIMIT:-30000000}
MAX_WAIT=${MAX_WAIT:-30}

# Use first account as deployer if no private key provided
PRIVATE_KEY=${PRIVATE_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

cleanup() {
    log "Cleaning up..."
    if [[ -n "${ANVIL_PID:-}" ]]; then
        kill $ANVIL_PID 2>/dev/null || true
        wait $ANVIL_PID 2>/dev/null || true
    fi
    exit 0
}

trap cleanup SIGTERM SIGINT

# Start Anvil in background
log "Starting Anvil devnet on ${ANVIL_HOST}:${ANVIL_PORT}..."
anvil \
    --host="$ANVIL_HOST" \
    --port="$ANVIL_PORT" \
    --chain-id="$CHAIN_ID" \
    --accounts="$ACCOUNTS" \
    --balance="$BALANCE" \
    --gas-limit="$GAS_LIMIT" \
    --gas-price=1000000000 \
    --block-time="$BLOCK_TIME" \
    --state-interval=10 \
    --dump-state=/tmp/anvil-state.json &

ANVIL_PID=$!

# Wait for Anvil to be ready with timeout
log "Waiting for Anvil to be ready (max ${MAX_WAIT}s)..."
for i in $(seq 1 $MAX_WAIT); do
    if curl -sf -X POST \
        -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "http://localhost:$ANVIL_PORT" >/dev/null 2>&1; then
        log "✓ Anvil is ready!"
        break
    fi
    
    if ! kill -0 $ANVIL_PID 2>/dev/null; then
        log "❌ Anvil process died during startup"
        exit 1
    fi
    
    if [[ $i -eq $MAX_WAIT ]]; then
        log "❌ Timeout waiting for Anvil to be ready"
        cleanup
        exit 1
    fi
    
    sleep 1
done

# Get network info
BLOCK_NUMBER=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    "http://localhost:$ANVIL_PORT" | jq -r '.result')

log "Current block number: $((BLOCK_NUMBER))"

# Deploy contracts
cd /home/foundry/contract
export FOUNDRY_PROFILE=default

log "Deploying InvoicePayment contract..."
if forge script script/InvoicePayment.s.sol:InvoicePaymentScript \
    --rpc-url "http://localhost:$ANVIL_PORT" \
    --private-key "$PRIVATE_KEY" \
    --broadcast; then
    log "✅ Contract deployment completed successfully"
    
    # Extract deployment address if available
    if [[ -f broadcast/InvoicePayment.s.sol/$CHAIN_ID/run-latest.json ]]; then
        CONTRACT_ADDRESS=$(jq -r '.transactions[0].contractAddress // empty' \
            broadcast/InvoicePayment.s.sol/$CHAIN_ID/run-latest.json)
        if [[ -n "$CONTRACT_ADDRESS" && "$CONTRACT_ADDRESS" != "null" ]]; then
            log "📝 Contract deployed at: $CONTRACT_ADDRESS"
            echo "$CONTRACT_ADDRESS" > /tmp/contract-address
        fi
    fi
else
    log "❌ Contract deployment failed"
    cleanup
    exit 1
fi

# Health check endpoint setup
log "Setting up health check..."
curl -sf "http://localhost:$ANVIL_PORT" >/dev/null 2>&1 && \
    log "✅ Health check passed"

log "🚀 Ethereum devnet is ready!"
log "   RPC URL: http://localhost:$ANVIL_PORT"
log "   Chain ID: $CHAIN_ID"
log "   Deployer: $(cast wallet address $PRIVATE_KEY)"
[[ -f /tmp/contract-address ]] && log "   Contract: $(cat /tmp/contract-address)"

# Keep container running and handle signals gracefully
wait $ANVIL_PID
EOF

RUN chmod +x /home/foundry/entrypoint.sh

# Health check
HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=3 \
    CMD curl -sf -X POST \
        -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "http://localhost:${ANVIL_PORT:-8545}" || exit 1

# Expose the RPC port
EXPOSE 8545

# Set default environment variables
ENV ANVIL_HOST=0.0.0.0
ENV ANVIL_PORT=8545
ENV CHAIN_ID=1337

ENTRYPOINT ["/home/foundry/entrypoint.sh"]
