#!/bin/bash
set -e 

if ! [ -x "$(command -v docker compose)" ]; then
  echo 'Error: docker compose is not installed.' >&2
  exit 1
fi

DOMAIN_NAME="${DOMAIN_NAME}"
EMAIL="${EMAIL}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export COMPOSE_FILE="$SCRIPT_DIR/lets-encrypt-setup/lets-encrypt-setup.yaml"

host_certbot_path="$SCRIPT_DIR/certbot"

if [ -d "$host_certbot_path" ]; then
  read -p "Existing data found for $domains. Continue and replace existing certificate? (y/N) " decision
  if [ "$decision" != "Y" ] && [ "$decision" != "y" ]; then
    exit
  fi
fi

if [ ! -e "$host_certbot_path/conf/options-ssl-nginx.conf" ] || [ ! -e "$host_certbot_path/conf/ssl-dhparams.pem" ]; then
  echo "### Downloading recommended TLS parameters ..."
  mkdir -p "$host_certbot_path/conf"
  curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf > "$host_certbot_path/conf/options-ssl-nginx.conf"
  curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem > "$host_certbot_path/conf/ssl-dhparams.pem"
  echo
fi


echo "Getting certificate for ${DOMAIN_NAME}..."
docker compose up --force-recreate -d nginx

sleep 3

docker-compose run --rm certbot certonly \
    --webroot \
    --webroot-path /var/www/certbot \
    --email ${EMAIL} \
    --agree-tos \
    --no-eff-email \
    -d ${DOMAIN_NAME}

docker-compose down nginx

export COMPOSE_FILE="$SCRIPT_DIR/node-starknet-sepolia.yaml"

ROOT_KEY="${ROOT_KEY}"
PG_URL="${PG_URL}"
STARKNET_CASHIER_ADDRESS="${STARKNET_CASHIER_ADDRESS}"
STARKNET_CASHIER_PRIVATE_KEY="${STARKNET_CASHIER_PRIVATE_KEY}"

docker-compose up -d
