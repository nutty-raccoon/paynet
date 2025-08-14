FROM ghcr.io/streamingfast/firehose-ethereum:8191665

# ENV ETHEREUM_NODE_URL=http://host.docker.internal:8545
ENV READER_NODE_ARGUMENTS="--dev --http --http.addr=0.0.0.0 --http.api eth,web3,net" 

EXPOSE 10016

ENTRYPOINT [ "/bin/sh", "-c" ]

CMD ["/app/fireeth start reader-node merger relayer --config-file='' --reader-node-path=/app/fireeth --common-first-streamable-block=0 --reader-node-arguments=\"${READER_NODE_ARGUMENTS}\" & wait"]