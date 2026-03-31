#!/bin/bash
set -euo pipefail
source .env 2>/dev/null || { echo "ERROR: .env not found"; exit 1; }

if [ -z "${SERVICE_REGISTRY_CONTRACT:-}" ]; then
  echo "ERROR: SERVICE_REGISTRY_CONTRACT is empty."
  echo "       Run: bash scripts/deploy-contracts.sh"
  exit 1
fi

echo "================================================"
echo "  Register Services On-Chain"
echo "================================================"
echo ""

OWNER=$(stellar keys address server)

register() {
  local NAME=$1 URL=$2 DESC=$3 PRICE=$4
  echo "Registering '$NAME' ($PRICE stroops, ~\$$(echo "scale=4; $PRICE/10000000" | bc) USDC)..."
  stellar contract invoke \
    --id "$SERVICE_REGISTRY_CONTRACT" \
    --source-account server \
    --network testnet \
    -- register_service \
      --owner "$OWNER" \
      --name "$NAME" \
      --url  "$URL" \
      --description "$DESC" \
      --price_usdc "$PRICE" \
      --tags '[]'
  echo "  Registered."
}

register "weather-api"      "http://localhost:3001/weather" "Real-time weather data for any city"           10000
register "xlm-price-feed"   "http://localhost:3001/price"   "XLM/USDC price and 24h market data"            10000
register "account-analyzer" "http://localhost:3001/account" "Stellar account balance and activity analysis" 20000

echo ""
echo "Done. View on Stellar Expert:"
echo "  https://stellar.expert/explorer/testnet/contract/$SERVICE_REGISTRY_CONTRACT"
