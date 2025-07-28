FROM ghcr.io/streamingfast/firehose-ethereum:main

ENV ETHEREUM_NODE_URL=http://ethereum-devnet:8545
ENV READER_NODE_ARGUMENTS="fetch 0 --state-dir fire-ethereum-state-dir --block-fetch-batch-size=1 --interval-between-fetch=0s --latest-block-retry-interval=5s --ethereum-endpoints=${ETHEREUM_NODE_URL}"

EXPOSE 10017
EXPOSE 10018

ENTRYPOINT ["/bin/sh", "-c"]

CMD ["/app/firecore start reader-node merger relayer --config-file='' --reader-node-path=/app/fireeth --common-first-streamable-block=0 --reader-node-arguments=\"$READER_NODE_ARGUMENTS\" & /app/firecore start firehose substreams-tier1 substreams-tier2 --config-file='' --common-first-streamable-block=0 --advertise-chain-name=ethereum-local & wait"]
