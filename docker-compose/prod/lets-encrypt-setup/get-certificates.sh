#!/bash/shell
set -e 

COMPOSE_FILE="${COMPOSE_FILE}"
DOMAIN_NAME="${DOMAIN_NAME}"
EMAIL="${EMAIL}"

echo "Getting certificate for ${DOMAIN_NAME}..."
 
docker-compose up -d nginx

sleep 3

docker-compose run --rm certbot certonly \
    --webroot \
    --webroot-path /var/www/certbot \
    --email ${EMAIL} \
    --agree-tos \
    --no-eff-email \
    -d ${DOMAIN_NAME}

docker-compose down nginx


