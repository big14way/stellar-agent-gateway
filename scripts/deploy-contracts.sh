#!/bin/bash
set -euo pipefail
source .env 2>/dev/null || { echo "ERROR: .env not found"; exit 1; }

echo "================================================"
echo "  Stellar Agent Gateway — Deploy Contracts"
echo "================================================"
echo ""

echo "[1/4] Building Soroban contracts..."
stellar contract build
echo "      Done."
echo ""

echo "[2/4] Deploying ServiceRegistry..."
REGISTRY_OUT=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/service_registry.wasm \
  --source-account server \
  --network testnet \
  -- --admin "$(stellar keys address server)" 2>&1)
SERVICE_REGISTRY_CONTRACT=$(echo "$REGISTRY_OUT" | grep -E '^[A-Z0-9]{56}$' | tail -1)
if [ -z "$SERVICE_REGISTRY_CONTRACT" ]; then
  echo "ERROR: Could not extract contract ID. Output was:"
  echo "$REGISTRY_OUT"
  exit 1
fi
echo "      Contract: $SERVICE_REGISTRY_CONTRACT"
echo "      Explorer: https://stellar.expert/explorer/testnet/contract/$SERVICE_REGISTRY_CONTRACT"
echo ""

echo "[3/4] Deploying SmartAccount..."
ACCOUNT_OUT=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/smart_account.wasm \
  --source-account server \
  --network testnet \
  -- --admin "$(stellar keys address server)" 2>&1)
SMART_ACCOUNT_CONTRACT=$(echo "$ACCOUNT_OUT" | grep -E '^[A-Z0-9]{56}$' | tail -1)
if [ -z "$SMART_ACCOUNT_CONTRACT" ]; then
  echo "ERROR: Could not extract contract ID. Output was:"
  echo "$ACCOUNT_OUT"
  exit 1
fi
echo "      Contract: $SMART_ACCOUNT_CONTRACT"
echo "      Explorer: https://stellar.expert/explorer/testnet/contract/$SMART_ACCOUNT_CONTRACT"
echo ""

echo "[4/4] Writing contract addresses to .env..."
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s|^SERVICE_REGISTRY_CONTRACT=.*|SERVICE_REGISTRY_CONTRACT=$SERVICE_REGISTRY_CONTRACT|" .env
  sed -i '' "s|^SMART_ACCOUNT_CONTRACT=.*|SMART_ACCOUNT_CONTRACT=$SMART_ACCOUNT_CONTRACT|" .env
else
  sed -i "s|^SERVICE_REGISTRY_CONTRACT=.*|SERVICE_REGISTRY_CONTRACT=$SERVICE_REGISTRY_CONTRACT|" .env
  sed -i "s|^SMART_ACCOUNT_CONTRACT=.*|SMART_ACCOUNT_CONTRACT=$SMART_ACCOUNT_CONTRACT|" .env
fi
echo "      .env updated."
echo ""
echo "Deployment complete."
