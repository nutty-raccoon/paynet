FROM ghcr.io/streamingfast/firehose-ethereum:e120c71-geth-v1.16.1-fh3.0-2

WORKDIR /app

ENV LOCALNET_NODE_IP=172.19.0.11

RUN cat > /app/entrypoint.sh <<'SH'
#!/bin/sh
set -eu

# Initialize chain data once
if [ ! -d "/execution-data/geth/geth/chaindata" ] && [ ! -d "/execution-data/geth/chaindata" ]; then
  echo "[entrypoint] Running geth init..."
  geth init --datadir /execution-data /el-cl-genesis-data/metadata/genesis.json
else
  echo "[entrypoint] Skipping geth init (already initialized)."
fi

echo "[entrypoint] Starting geth..."
exec geth \
  --networkid=31337 \
  --state.scheme=path \
  --verbosity=3 \
  --datadir=/execution-data \
  --http \
  --http.addr=0.0.0.0 \
  --http.port=8545 \
  --http.vhosts=* \
  --http.corsdomain=* \
  --http.api=admin,engine,net,eth,web3,debug,txpool \
  --ws \
  --ws.addr=0.0.0.0 \
  --ws.port=8546 \
  --ws.api=admin,engine,net,eth,web3,debug,txpool \
  --ws.origins=* \
  --allow-insecure-unlock \
  --nat=extip:${LOCALNET_NODE_IP} \
  --authrpc.port=8551 \
  --authrpc.addr=0.0.0.0 \
  --authrpc.vhosts=* \
  --authrpc.jwtsecret=/el-cl-genesis-data/jwt/jwtsecret \
  --syncmode=full \
  --rpc.allow-unprotected-txs \
  --metrics \
  --metrics.addr=0.0.0.0 \
  --metrics.port=6060 \
  --port=30303 \
  --discovery.port=30303
SH

EXPOSE 8545 8546 8551 30303 6060

RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
