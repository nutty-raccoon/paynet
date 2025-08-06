FROM ghcr.io/streamingfast/firehose-starknet:v1.1.1

ENV COMMON_FIRST_STREAMABLE_BLOCK=0
ENV ETH_ENDPOINT_API=""
ENV LATEST_BLOCK_RETRY_INTERVAL=1s
ENV ADVERTISE_CHAIN_NAME=devnet
ENV BLOCK_FETCH_BATCH_SIZE=1

EXPOSE 10016

# Copy the entrypoint script
COPY ./dockerfiles/starknet-firehose-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
