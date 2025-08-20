FROM alpine:3.20 AS prep

WORKDIR /app

# Build-time tools only
RUN apk add --no-cache curl jq ca-certificates

ARG NODE1_ADDRESS="host.docker.internal"
ARG NODE1_HTTP_PORT="5052"


RUN echo "[prep] Fetching CL identity from http://${NODE1_ADDRESS}:${NODE1_HTTP_PORT}/eth/v1/node/identity" \
 && (curl -sS -m 3 -H "Accept: application/json" \
       "http://${NODE1_ADDRESS}:${NODE1_HTTP_PORT}/eth/v1/node/identity" \
     | tee /tmp/identity.json \
     | jq -r '.data.peer_id // empty' > /app/cl_peer_id.txt || true) \
 && (cat /tmp/identity.json | jq -r '.data.enr // empty' > /app/cl_enr.txt || true) \
 && echo "[prep] peer_id=$(cat /app/cl_peer_id.txt || true)" \
 && echo "[prep] enr=$(cat /app/cl_enr.txt || true)"


FROM sigp/lighthouse:latest
WORKDIR /app

COPY --from=prep /app/cl_peer_id.txt /app/cl_peer_id.txt
COPY --from=prep /app/cl_enr.txt     /app/cl_enr.txt

ENV ENR_ADDRESS=172.19.0.121
ENV EXECUTION_ENDPOINTS=http://host.docker.internal:9551
ENV CL_CHECKPOINT=http://host.docker.internal:5052

RUN cat > /entrypoint.sh <<'SH'
#!/bin/sh
set -eu

# Fill CL_TRUSTPEERS / CL_BOOTNODES from prepared files if not provided
if [ -z "${CL_TRUSTPEERS:-}" ] && [ -s /app/cl_peer_id.txt ]; then
  export CL_TRUSTPEERS="$(cat /app/cl_peer_id.txt)"
  echo "[entrypoint] Using CL_TRUSTPEERS from /app/cl_peer_id.txt: ${CL_TRUSTPEERS}"
fi

if [ -z "${CL_BOOTNODES:-}" ] && [ -s /app/cl_enr.txt ]; then
  export CL_BOOTNODES="$(cat /app/cl_enr.txt)"
  echo "[entrypoint] Using CL_BOOTNODES from /app/cl_enr.txt: ${CL_BOOTNODES}"
fi

# Build args
args="
  beacon_node
  --debug-level=info
  --datadir=/consensus-data
  --testnet-dir=/el-cl-genesis-data/metadata
  --listen-address=0.0.0.0
  --port=10000
  --enr-address=${ENR_ADDRESS}
  --enr-udp-port=10000
  --enr-tcp-port=10000
  --enr-quic-port=10001
  --http
  --http-address=0.0.0.0
  --http-port=6052
  --http-allow-origin=*
  --execution-endpoints=${EXECUTION_ENDPOINTS}
  --jwt-secrets=/el-cl-genesis-data/jwt/jwtsecret
  --metrics
  --metrics-address=0.0.0.0
  --metrics-port=6054
  --metrics-allow-origin=*
  --disable-peer-scoring
  --trusted-peers=${CL_TRUSTPEERS}
  --boot-nodes=${CL_BOOTNODES}
  --checkpoint-sync-url=${CL_CHECKPOINT}
  --disable-packet-filter
  --enable-private-discovery
  --purge-db-force
"

echo "[entrypoint] Exec: lighthouse $args $*"
exec lighthouse $args "$@"
SH

RUN chmod +x /entrypoint.sh

EXPOSE 10000 10001 6052 6054

ENTRYPOINT ["/entrypoint.sh"]
