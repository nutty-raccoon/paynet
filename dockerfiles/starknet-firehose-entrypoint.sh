#!/bin/sh

# Read the dynamic block number from shared volume
if [ -f /shared/fork_block.txt ]; then
    DYNAMIC_BLOCK=$(cat /shared/fork_block.txt)
    export COMMON_FIRST_STREAMABLE_BLOCK=$DYNAMIC_BLOCK
    echo "Using dynamic fork block: $DYNAMIC_BLOCK"
else
    echo "No dynamic block found, using default: $COMMON_FIRST_STREAMABLE_BLOCK"
fi

# Start the firehose services with the dynamic block number
exec /app/firecore start reader-node merger relayer --config-file='' \
    --reader-node-path=/app/firestarknet \
    --common-first-streamable-block=${COMMON_FIRST_STREAMABLE_BLOCK} \
    --reader-node-arguments="fetch ${COMMON_FIRST_STREAMABLE_BLOCK} --state-dir fire-starknet-state-dir --block-fetch-batch-size=${BLOCK_FETCH_BATCH_SIZE} --interval-between-fetch=0s --latest-block-retry-interval=${LATEST_BLOCK_RETRY_INTERVAL} --starknet-endpoints=http://host.docker.internal:5050 --eth-endpoints=${ETH_ENDPOINT_API}" & \
/app/firecore start firehose substreams-tier1 substreams-tier2 --config-file='' \
    --common-first-streamable-block=${COMMON_FIRST_STREAMABLE_BLOCK} \
    --advertise-chain-name=${ADVERTISE_CHAIN_NAME} & \
wait
