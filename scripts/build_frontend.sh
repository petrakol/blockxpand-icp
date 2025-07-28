#!/bin/sh
set -e
CID=${CANISTER_ID:-$(dfx canister id aggregator 2>/dev/null || true)}
if [ -z "$CID" ]; then
  echo "Error: CANISTER_ID not provided and dfx query failed" >&2
  exit 1
fi
CALL_PRICE=${CALL_PRICE_CYCLES:-0}
CLAIM_PRICE=${CLAIM_PRICE_CYCLES:-0}
mkdir -p frontend/dist
sed -e "s/<CANISTER_ID>/$CID/g" \
    -e "s/<CALL_PRICE>/$CALL_PRICE/g" \
    -e "s/<CLAIM_PRICE>/$CLAIM_PRICE/g" \
    frontend/index.html > frontend/dist/index.html

# Copy Cloudflare headers file so GLB served with correct MIME type
if [ -f frontend/_headers ]; then
  cp frontend/_headers frontend/dist/_headers
fi

echo "Generated frontend/dist/index.html with canister ID $CID"
