FROM ghcr.io/streamingfast/firehose-starknet:main

ENV STARKNET_NODE_URL=http://host.docker.internal:5050
ENV READER_NODE_ARGUMENTS="fetch 0 --state-dir fire-starknet-state-dir --block-fetch-batch-size=1 --interval-between-fetch=0s --latest-block-retry-interval=5s --starknet-endpoints=${STARKNET_NODE_URL} --eth-endpoints=https://eth-mainnet.public.blastapi.io"

EXPOSE 10016

ENTRYPOINT ["/bin/sh", "-c"]

CMD ["/app/firecore start reader-node merger relayer --config-file='' --reader-node-path=/app/firestarknet --common-first-streamable-block=0 --reader-node-arguments=\"$READER_NODE_ARGUMENTS\" & /app/firecore start firehose substreams-tier1 substreams-tier2 --config-file='' --common-first-streamable-block=0 --advertise-chain-name=starknet-devnet  & wait"]
