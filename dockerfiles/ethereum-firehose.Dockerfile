# Use the Firehose Ethereum image
FROM ghcr.io/streamingfast/firehose-ethereum:e120c71-geth-v1.16.1-fh3.0-2

# Expose the gRPC ports
EXPOSE 10015
EXPOSE 10016

# Start firehose with Substreams enabled
CMD ["start", \
     "--config-file=", \
     "firehose", "relayer", "merger", \
     "--substreams-rpc-endpoints=http://host.docker.internal:8545", \
     "--common-first-streamable-block=0"]
