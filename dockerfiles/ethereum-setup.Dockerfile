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

# Create deployment script that mimics starknet-setup pattern
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'set -e' >> /entrypoint.sh && \
    echo 'export FOUNDRY_PROFILE=default' >> /entrypoint.sh && \
    echo 'echo "Deploying InvoicePayment contract to devnet..."' >> /entrypoint.sh && \
    echo 'forge script script/InvoicePayment.s.sol:InvoicePaymentScript --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" --broadcast' >> /entrypoint.sh && \
    echo 'echo "Contract deployment completed successfully"' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]