#!/bin/bash
set -euo pipefail
source .env 2>/dev/null || { echo "ERROR: .env not found"; exit 1; }

echo "================================================"
echo "  Agent Account Setup"
echo "================================================"
echo ""
echo "Agent address: $AGENT_STELLAR_ADDRESS"
echo ""

# Step 1: Add USDC trustline for the agent account
# The agent needs a trustline to hold USDC before receiving/spending it.
echo "[1/2] Adding USDC trustline for agent..."
stellar tx new change-trust \
  --source agent \
  --line "USDC:${USDC_ISSUER}" \
  --network testnet \
  --sign \
  --send
echo "      Trustline created."
echo ""

# Step 2: The agent also needs XLM for the sequence number account (source account in tx)
# This is already funded by `stellar keys generate agent --fund`
# Reminder: go to faucet.circle.com to get test USDC after trustline is set
echo "[2/2] Agent setup complete."
echo ""
echo "NEXT STEPS:"
echo "  1. Get test USDC for the agent:"
echo "     Visit: https://faucet.circle.com"
echo "     Select: Stellar Testnet"
echo "     Paste: $AGENT_STELLAR_ADDRESS"
echo "     Request 20 USDC"
echo ""
echo "  2. Verify agent USDC balance:"
echo "     curl -s https://horizon-testnet.stellar.org/accounts/$AGENT_STELLAR_ADDRESS | python3 -c \"import sys,json; b=[b for b in json.load(sys.stdin)['balances'] if b.get('asset_code')=='USDC']; print('USDC:', b[0]['balance'] if b else '0 (no trustline?)')\""
echo ""
