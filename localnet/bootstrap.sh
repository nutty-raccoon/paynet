#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# localnet bootstrap runner
# Runs all phases in sequence and exports env variables to your shell
#
# Usage:
#   source ./bootstrap.sh

set -euo pipefail

# compose files
COMPOSE_PREP="./docker-compose.localnet-1-prep.yml"
COMPOSE_GENESIS="./docker-compose.localnet-2-genesis.yml"
COMPOSE_INIT="./docker-compose.localnet-3-init.yml"
COMPOSE_RUN="./docker-compose.localnet-4-run.yml"

# env file written by prep
VALUES_ENV="./localnet/config/values.env"

# must be sourced so exported env persists
(return 0 2>/dev/null) || { echo "Please run with: source $0"; exit 1; }

# sanity checks
for f in "$COMPOSE_PREP" "$COMPOSE_GENESIS" "$COMPOSE_INIT" "$COMPOSE_RUN"; do
  [ -f "$f" ] || { echo "Missing file: $f" >&2; return 1; }
done

echo "==> Phase 1/4: PREP"
docker compose -f "$COMPOSE_PREP" up   # foreground to ensure completion

echo "==> Exporting env from $VALUES_ENV"
[ -f "$VALUES_ENV" ] || { echo "Missing $VALUES_ENV" >&2; return 1; }
set -a
. "$VALUES_ENV"
set +a

echo "==> Phase 2/4: GENESIS"
docker compose -f "$COMPOSE_GENESIS" up -d

echo "==> Phase 3/4: INIT"
docker compose -f "$COMPOSE_INIT" up -d

echo "==> Phase 4/4: RUN"
docker compose -f "$COMPOSE_RUN" up -d

echo "✅ All phases completed."
