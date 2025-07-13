#!/bin/bash

# Script to run multiple node instances with unique generated keys
# Usage: ./run-multiple-nodes.sh <number_of_instances>

set -e  # Exit on any error

# Check if number of instances is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <number_of_instances>"
    echo "Example: $0 3"
    exit 1
fi

MAX_INSTANCES=$1

# Validate input
if ! [[ "$MAX_INSTANCES" =~ ^[0-9]+$ ]] || [ "$MAX_INSTANCES" -lt 1 ]; then
    echo "Error: Please provide a valid positive number of instances"
    exit 1
fi

echo "Building gen-btc-xpriv binary..."
cargo build --release -p gen-btc-xpriv

echo "Starting $MAX_INSTANCES node instances..."

# Loop through instances
for i in $(seq 1 $MAX_INSTANCES); do
    echo ""
    echo "=== Starting Node Instance $i ==="
    
    # Generate a new private key
    echo "Generating private key for instance $i..."
    ROOT_KEY=$(./target/release/gen-btc-xpriv)
    
    if [ -z "$ROOT_KEY" ]; then
        echo "Error: Failed to generate root key for instance $i"
        exit 1
    fi
    
    echo "Generated root key: ${ROOT_KEY:0:20}..."
    
    # Calculate ports for this instance
    DB_PORT=$((5430 + i))
    SIGNER_PORT=$((i * 10000 + 1))
    NODE_PORT=$((i * 10000 + 3))
    
    echo "Instance $i configuration:"
    echo "  DB Port: $DB_PORT"
    echo "  Signer Port: $SIGNER_PORT"
    echo "  Node Port: $NODE_PORT"
    echo "  Project Name: node$i"
    
    # Start the docker compose stack
    echo "Starting docker-compose for instance $i..."
    ROOT_KEY="$ROOT_KEY" DB_PORT=$DB_PORT SIGNER_PORT=$SIGNER_PORT NODE_PORT=$NODE_PORT \
        docker-compose -f docker-compose.app-mock.yml -p node$i up -d
    
    if [ $? -eq 0 ]; then
        echo "✅ Instance $i started successfully"
    else
        echo "❌ Failed to start instance $i"
        exit 1
    fi
    
    # Small delay to avoid overwhelming the system
    sleep 2
done

echo ""
echo "🎉 All $MAX_INSTANCES instances started successfully!"
echo ""
echo "Instance summary:"
for i in $(seq 1 $MAX_INSTANCES); do
    DB_PORT=$((5430 + i))
    SIGNER_PORT=$((i * 10000 + 1))
    NODE_PORT=$((i * 10000 + 3))
    echo "  Instance $i: DB=$DB_PORT, Signer=$SIGNER_PORT, Node=$NODE_PORT (project: node$i)"
done

echo ""
echo "To stop all instances, run:"
for i in $(seq 1 $MAX_INSTANCES); do
    echo "  docker-compose -p node$i down"
done

echo ""
echo "Or use this one-liner to stop all:"
echo "  for i in \$(seq 1 $MAX_INSTANCES); do docker-compose -p node\$i down; done"
