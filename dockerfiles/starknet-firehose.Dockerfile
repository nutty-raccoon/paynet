FROM ghcr.io/streamingfast/firehose-starknet:v1.1.1

ENV STARKNET_NODE_URL=http://host.docker.internal:5050
ENV COMMON_FIRST_STREAMABLE_BLOCK=0
ENV ETH_ENDPOINT_API=""
ENV LATEST_BLOCK_RETRY_INTERVAL=1s
ENV READER_NODE_ARGUMENTS="fetch ${COMMON_FIRST_STREAMABLE_BLOCK} --state-dir fire-starknet-state-dir --block-fetch-batch-size=1 --interval-between-fetch=0s --latest-block-retry-interval=${LATEST_BLOCK_RETRY_INTERVAL} --starknet-endpoints=${STARKNET_NODE_URL} --eth-endpoints=${ETH_ENDPOINT_API}"

EXPOSE 10016

ENTRYPOINT ["/bin/sh", "-c"]

CMD ["/app/firecore start reader-node merger relayer --config-file='' --reader-node-path=/app/firestarknet --common-first-streamable-block=${COMMON_FIRST_STREAMABLE_BLOCK} --reader-node-arguments=\"$READER_NODE_ARGUMENTS\" & /app/firecore start firehose substreams-tier1 substreams-tier2 --config-file='' --common-first-streamable-block=${COMMON_FIRST_STREAMABLE_BLOCK} --advertise-chain-name=devnet & wait"]
