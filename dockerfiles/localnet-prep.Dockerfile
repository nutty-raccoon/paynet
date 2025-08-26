FROM alpine:latest AS prep

RUN apk add --no-cache curl

CMD ["/bin/sh", "-c", "\
  set -eu; \
  FILE=/config/values.env; \
  [ -f \"$FILE\" ] || { echo \"$FILE not found\"; exit 1; }; \
  TS=$(date -u +%s); \
  IP=$(curl -4 -fsSL https://icanhazip.com | tr -d '\r\n'); \
  [ -n \"$IP\" ] || { echo 'failed to fetch public IP' >&2; exit 1; }; \
  sed -i \"s/GENESIS_TIMESTAMP=[0-9]*/GENESIS_TIMESTAMP=$TS/\" \"$FILE\"; \
  sed -i \"s/IP_ADDRESS=.*/IP_ADDRESS=$IP/\" \"$FILE\"; \
  echo \"updated $FILE -> GENESIS_TIMESTAMP=$TS GENESIS_TIME=$TS IP_ADDRESS=$IP\" \
"]