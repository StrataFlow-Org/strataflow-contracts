#!/usr/bin/env bash
set -euo pipefail

# StrataFlow Contracts - Stellar Testnet Deployment Script
# Requirements: stellar-cli (or soroban-cli)

NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org"
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
ADMIN_IDENTITY="${ADMIN_IDENTITY:-default}"

echo "=========================================="
echo "🚀 Building StrataFlow Soroban Contract"
echo "=========================================="
cargo rustc --target wasm32-unknown-unknown --release --crate-type cdylib

WASM_PATH="target/wasm32-unknown-unknown/release/strataflow_contracts.wasm"

if [ ! -f "$WASM_PATH" ]; then
    echo "❌ Error: Wasm binary not found at $WASM_PATH"
    exit 1
fi

echo "📦 Installing contract code onto Stellar $NETWORK..."
WASM_HASH=$(stellar contract install \
    --wasm "$WASM_PATH" \
    --source "$ADMIN_IDENTITY" \
    --network "$NETWORK")

echo "✅ Contract Code Installed! Hash: $WASM_HASH"

echo "⚡ Deploying StrataFlow Contract Instance..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm-hash "$WASM_HASH" \
    --source "$ADMIN_IDENTITY" \
    --network "$NETWORK")

echo "🎉 StrataFlow Deployed Successfully!"
echo "Contract ID: $CONTRACT_ID"
echo "Network: $NETWORK"
echo "Save this CONTRACT_ID to your strataflow-backend and strataflow-web .env files."

