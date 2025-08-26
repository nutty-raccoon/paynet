FROM ethpandaops/ethereum-genesis-generator:5.0.0

RUN cat > ./genesis-entrypoint.sh <<'SH'
#!/bin/sh
set -eu

echo "[*] Cleaning old data"
rm -rvf "/nodes/node1/execution-data" \
        "/nodes/node1/consensus-data" \
        "/nodes/node2/execution-data" \
        "/nodes/node2/consensus-data"

TS="$(date -u +%s)"
export GENESIS_TIMESTAMP="$TS"

echo "[genesis] GENESIS_TIMESTAMP=$GENESIS_TIMESTAMP"

# Run the normal genesis generator entrypoint
exec ./entrypoint.sh all
SH

RUN chmod +x ./genesis-entrypoint.sh

ENTRYPOINT ["./genesis-entrypoint.sh"]
