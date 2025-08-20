FROM alpine:3.20 AS prep

WORKDIR /app

# Build-time tools only
RUN apk add --no-cache curl jq wget ca-certificates && rm -rf /var/lib/apt/lists/*

ARG EL_CLIENT_URL="http://host.docker.internal:8545"

# JSON-RPC payload that calls admin_nodeInfo and returns .result.enode
ADD ./localnet/firehose/el.request.json /app/el.request.json

# Query EL to extract enode at build-time (best-effort; leaves empty file if unreachable)
RUN echo "[prep] Fetching enode from ${EL_CLIENT_URL}" \
  && (curl -s -m 3 -H "Content-Type: application/json" \
       --data @/app/el.request.json "${EL_CLIENT_URL}" \
       | jq -r '.result.enode // empty' > /app/enode.txt || true) \
  && echo "[prep] enode=$(cat /app/enode.txt || true)"


FROM ghcr.io/streamingfast/firehose-ethereum:e120c71-geth-v1.16.1-fh3.0-2
WORKDIR /app

# copy artifacts from prep
COPY --from=prep /app/enode.txt  /app/enode.txt

ENV EL_NODE_ENDPOINTS="http://host.docker.internal:8545"

ENV LOCALNET_NODE_IP="172.19.0.111"

RUN cat > /entrypoint.sh <<'SH'
#!/bin/sh
set -eu

: "${FIREHOSE_CONFIG:=/firehose/firehose-rmr.yaml}"

# If EL_BOOTNODES isn't provided at runtime and enode.txt exists, use it.
if [ -z "${EL_BOOTNODES:-}" ] && [ -s /app/enode.txt ]; then
  EL_BOOTNODES="$(cat /app/enode.txt)"
  export EL_BOOTNODES
  echo "[entrypoint] Using EL_BOOTNODES from /app/enode.txt: $EL_BOOTNODES"
else
  echo "[entrypoint] EL_BOOTNODES already set or /app/enode.txt missing/empty."
fi

if [ ! -d "/execution-data/geth/geth/chaindata" ] && [ ! -d "/execution-data/geth/chaindata" ]; then
  echo "[entrypoint] Running 'geth init'..."
  geth init --datadir /execution-data /el-cl-genesis-data/metadata/genesis.json
else
  echo "[entrypoint] Skipping 'geth init' (already initialized)."
fi

exec fireeth -c "${FIREHOSE_CONFIG}" start "$@"
SH

RUN chmod +x /entrypoint.sh

EXPOSE 10015 10016 9545 9546 30304 7060 9551

ENTRYPOINT ["/entrypoint.sh"]
