# Remove any old "localnet" network if it exists
docker network rm localnet || true

# Create a fresh bridge network named "localnet"
#  - Uses driver: bridge (standard user-defined network)
#  - Allocates IPs from the 172.19.0.0/16 range
docker network create \
  --driver bridge \
  --subnet 172.19.0.0/16 \
  localnet
#  - This network is used by the localnet containers to communicate with each other
#  - The IPs are static and predictable, which is useful for local development and testing