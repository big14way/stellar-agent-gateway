#!/bin/bash
set -uo pipefail
PASS=0; FAIL=0

ok()   { echo "  PASS: $1"; ((PASS++)); }
fail() { echo "  FAIL: $1 — $2"; ((FAIL++)); }

echo "================================================"
echo "  Integration Tests"
echo "================================================"
echo ""

# Check services are running
STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 http://localhost:3001/health 2>/dev/null || echo "000")
if [ "$STATUS" = "200" ]; then
  ok "Services running (health=200)"
else
  echo "  x402 services are not running on port 3001."
  echo "  Start them first: make dev-services"
  echo ""
  echo "Skipping remaining tests."
  exit 1
fi

# Test that each paid route returns 402
for ROUTE in "weather?city=Lagos" "price" "account?address=GABC"; do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3001/$ROUTE")
  if [ "$STATUS" = "402" ]; then
    ok "$ROUTE returns 402 without payment"
  else
    fail "$ROUTE returns 402" "got $STATUS"
  fi
done

# Test PAYMENT-REQUIRED header is present and valid base64 JSON
HEADER_RAW=$(curl -sI "http://localhost:3001/weather?city=Test" | grep -i "^payment-required:" | head -1)
if [ -n "$HEADER_RAW" ]; then
  ok "PAYMENT-REQUIRED header present"
  B64=$(echo "$HEADER_RAW" | sed 's/^[Pp]ayment-[Rr]equired: //' | tr -d '\r\n ')
  DECODED=$(echo "$B64" | base64 -d 2>/dev/null)
  if echo "$DECODED" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d.get('x402Version')==2" 2>/dev/null; then
    ok "PAYMENT-REQUIRED is valid x402 V2 JSON"
  else
    fail "PAYMENT-REQUIRED JSON" "not valid x402 V2 format"
  fi
else
  fail "PAYMENT-REQUIRED header" "not present in 402 response"
fi

echo ""
echo "Results: $PASS passed, $FAIL failed."
[ "$FAIL" -eq 0 ] && echo "All tests passed." && exit 0 || exit 1
