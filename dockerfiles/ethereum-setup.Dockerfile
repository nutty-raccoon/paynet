FROM ghcr.io/foundry-rs/foundry:latest AS contract-builder

WORKDIR /contracts/ethereum/invoice

# Initialize git first
RUN git init && git config user.email "ci@example.com" && git config user.name "CI"

# Copy contract files (excluding lib directory to avoid conflicts)
COPY ./contracts/ethereum/invoice/src ./src/
COPY ./contracts/ethereum/invoice/script ./script/
COPY ./contracts/ethereum/invoice/test ./test/
COPY ./contracts/ethereum/invoice/foundry.toml ./
COPY ./contracts/ethereum/invoice/remappings.txt ./

# Install dependencies fresh
RUN forge install foundry-rs/forge-std
RUN forge install openzeppelin/openzeppelin-contracts
RUN forge build

# ----------------

FROM ghcr.io/foundry-rs/foundry:latest AS executable

# Install curl for healthchecks and other utilities
USER root
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Copy built contracts
COPY --from=contract-builder /contracts/ethereum/invoice/ /contract/

WORKDIR /contract

# Create combined script that runs Anvil and deploys contracts
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'set -e' >> /entrypoint.sh && \
    echo '' >> /entrypoint.sh && \
    echo '# Start Anvil in background' >> /entrypoint.sh && \
    echo 'echo "Starting Anvil devnet..."' >> /entrypoint.sh && \
    echo 'anvil --host=0.0.0.0 --port=8545 --chain-id=1337 --accounts=10 --balance=10000 --gas-limit=30000000 --gas-price=1000000000 --block-time=2 &' >> /entrypoint.sh && \
    echo 'ANVIL_PID=$!' >> /entrypoint.sh && \
    echo '' >> /entrypoint.sh && \
    echo '# Wait for Anvil to be ready' >> /entrypoint.sh && \
    echo 'echo "Waiting for Anvil to be ready..."' >> /entrypoint.sh && \
    echo 'until curl -s -X POST -H "Content-Type: application/json" --data '"'"'{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'"'"' http://localhost:8545 > /dev/null 2>&1; do' >> /entrypoint.sh && \
    echo '  sleep 1' >> /entrypoint.sh && \
    echo 'done' >> /entrypoint.sh && \
    echo 'echo "✓ Anvil is ready!"' >> /entrypoint.sh && \
    echo '' >> /entrypoint.sh && \
    echo '# Deploy contracts' >> /entrypoint.sh && \
    echo 'export FOUNDRY_PROFILE=default' >> /entrypoint.sh && \
    echo 'echo "Deploying InvoicePayment contract to devnet..."' >> /entrypoint.sh && \
    echo 'forge script script/InvoicePayment.s.sol:InvoicePaymentScript --rpc-url "http://localhost:8545" --private-key "$PRIVATE_KEY" --broadcast' >> /entrypoint.sh && \
    echo 'echo "Contract deployment completed successfully"' >> /entrypoint.sh && \
    echo '' >> /entrypoint.sh && \
    echo '# Keep Anvil running' >> /entrypoint.sh && \
    echo 'echo "Ethereum devnet is ready at http://localhost:8545"' >> /entrypoint.sh && \
    echo 'wait $ANVIL_PID' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]