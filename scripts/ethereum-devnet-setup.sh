#!/bin/bash

# Ethereum Anvil Devnet Info Script
# This script displays information about the running Anvil devnet

set -e

echo "Ethereum Anvil Devnet Information"
echo "================================="

# Wait for Anvil to be ready
echo "Checking if Anvil node is ready..."
until curl -s -X POST \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545 > /dev/null 2>&1; do
  echo "Waiting for Anvil node..."
  sleep 2
done

echo "✓ Anvil node is ready!"
echo ""

# Get network info
echo "Network Information:"
echo "==================="
echo "RPC URL: http://localhost:8545"
echo "Chain ID: 1337"
echo "Block time: 2 seconds"
echo ""

# Get pre-funded accounts
echo "Pre-funded Development Accounts (10,000 ETH each):"
echo "=================================================="

for i in {0..9}; do
  ACCOUNT=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_accounts\",\"params\":[],\"id\":1}" \
    http://localhost:8545 | jq -r ".result[$i]" 2>/dev/null)

  if [ "$ACCOUNT" != "null" ] && [ -n "$ACCOUNT" ]; then
    BALANCE=$(curl -s -X POST \
      -H "Content-Type: application/json" \
      --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$ACCOUNT\",\"latest\"],\"id\":1}" \
      http://localhost:8545 | jq -r '.result' 2>/dev/null)

    echo "Account $((i+1)): $ACCOUNT"
  fi
done

echo ""
echo "Anvil devnet is ready for development!"
echo "Use any of the above accounts for testing."