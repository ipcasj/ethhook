#!/bin/bash

# Setup Demo Account with Pre-configured Applications and Endpoints
# This script creates a fully-featured demo environment for client demonstrations

set -e

API_URL="http://104.248.15.178:3000/api/v1"
DEMO_EMAIL="demo@ethhook.com"
DEMO_PASSWORD="Demo1234!"
WEBHOOK_URL="http://104.248.15.178:8000/webhook"

echo "🔐 Logging in as demo user..."
TOKEN=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$DEMO_EMAIL\", \"password\": \"$DEMO_PASSWORD\"}" | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "❌ Failed to login"
  exit 1
fi

echo "✅ Logged in successfully"
echo ""

# Get existing applications
echo "📱 Fetching existing applications..."
APPS=$(curl -s -X GET "$API_URL/applications" -H "Authorization: Bearer $TOKEN")
DEFI_APP_ID=$(echo "$APPS" | jq -r '.applications[] | select(.name=="DeFi Protocol Monitor") | .id')
NFT_APP_ID=$(echo "$APPS" | jq -r '.applications[] | select(.name=="NFT Marketplace Tracker") | .id')
BRIDGE_APP_ID=$(echo "$APPS" | jq -r '.applications[] | select(.name=="Multi-Chain Bridge Monitor") | .id')

echo "  DeFi App: $DEFI_APP_ID"
echo "  NFT App: $NFT_APP_ID"
echo "  Bridge App: $BRIDGE_APP_ID"
echo ""

# Create endpoints for DeFi Protocol Monitor
echo "📡 Creating DeFi Protocol endpoints..."

# USDC Transfers
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$DEFI_APP_ID\",
    \"name\": \"USDC Transfers\",
    \"description\": \"USD Coin (USDC) transfer events on Ethereum mainnet\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48\"],
    \"event_signatures\": [\"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef\"]
  }" | jq -r '.name' && echo "  ✅ USDC Transfers"

# Uniswap V3 ETH/USDC Pool
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$DEFI_APP_ID\",
    \"name\": \"Uniswap V3: ETH/USDC Swaps\",
    \"description\": \"Swap events from the most liquid Uniswap V3 pool\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640\"],
    \"event_signatures\": [\"0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67\"]
  }" | jq -r '.name' && echo "  ✅ Uniswap V3 Swaps"

# WETH Deposits/Withdrawals
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$DEFI_APP_ID\",
    \"name\": \"WETH Deposits & Withdrawals\",
    \"description\": \"Wrapped ETH deposit and withdrawal events\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2\"],
    \"event_signatures\": [\"0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c\", \"0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65\"]
  }" | jq -r '.name' && echo "  ✅ WETH Deposits & Withdrawals"

# Aave V3 Pool
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$DEFI_APP_ID\",
    \"name\": \"Aave V3: Supply & Borrow\",
    \"description\": \"Aave V3 lending pool supply and borrow events\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2\"],
    \"event_signatures\": [\"0x2b627736bca15cd5381dcf80b0bf11fd197d01a037c52b927a881a10fb73ba61\", \"0xb3d084820fb1a9decffb176436bd02558d15fac9b0ddfed8c465bc7359d7dce0\"]
  }" | jq -r '.name' && echo "  ✅ Aave V3 Supply & Borrow"

echo ""

# Create endpoints for NFT Marketplace Tracker
echo "🖼️  Creating NFT Marketplace endpoints..."

# OpenSea Seaport
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$NFT_APP_ID\",
    \"name\": \"OpenSea: Order Fulfilled\",
    \"description\": \"OpenSea Seaport order fulfilled events (NFT sales)\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0x00000000000000adc04c56bf30ac9d3c0aaf14dc\"],
    \"event_signatures\": [\"0x9d9af8e38d66c62e2c12f0225249fd9d721c54b83f48d9352c97c6cacdcb6f31\"]
  }" | jq -r '.name' && echo "  ✅ OpenSea Order Fulfilled"

# Blur Marketplace
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$NFT_APP_ID\",
    \"name\": \"Blur: Order Executed\",
    \"description\": \"Blur marketplace NFT sale events\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0x000000000000ad05ccc4f10045630fb830b95127\"],
    \"event_signatures\": [\"0x61cbb2a3dee0b6064c2e681aadd61677fb4ef319f0b547508d495626f5a62f64\"]
  }" | jq -r '.name' && echo "  ✅ Blur Order Executed"

# BAYC Transfers
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$NFT_APP_ID\",
    \"name\": \"BAYC: Token Transfers\",
    \"description\": \"Bored Ape Yacht Club NFT transfers\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d\"],
    \"event_signatures\": [\"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef\"]
  }" | jq -r '.name' && echo "  ✅ BAYC Transfers"

# Azuki Transfers
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$NFT_APP_ID\",
    \"name\": \"Azuki: Token Transfers\",
    \"description\": \"Azuki NFT collection transfers\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0xed5af388653567af2f388e6224dc7c4b3241c544\"],
    \"event_signatures\": [\"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef\"]
  }" | jq -r '.name' && echo "  ✅ Azuki Transfers"

echo ""

# Create endpoints for Multi-Chain Bridge Monitor
echo "🌉 Creating Bridge Monitor endpoints..."

# Arbitrum Bridge
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$BRIDGE_APP_ID\",
    \"name\": \"Arbitrum Bridge: Deposits\",
    \"description\": \"Arbitrum bridge deposit events from Ethereum\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0x8315177ab297ba92a06054ce80a67ed4dbd7ed3a\"],
    \"event_signatures\": [\"0x85291dff2161a93c2f12c819d31889c96c63042116f5bc5a205aa701c2c429f5\"]
  }" | jq -r '.name' && echo "  ✅ Arbitrum Deposits"

# Optimism Bridge
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$BRIDGE_APP_ID\",
    \"name\": \"Optimism Bridge: Deposits\",
    \"description\": \"Optimism bridge ETH and ERC20 deposit events\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0x99c9fc46f92e8a1c0dec1b1747d010903e884be1\"],
    \"event_signatures\": [\"0x73d170910aba9e6d50b102db522b1dbcd796216f5128b445aa2135272886497e\"]
  }" | jq -r '.name' && echo "  ✅ Optimism Deposits"

# Base Bridge
curl -s -X POST "$API_URL/endpoints" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"application_id\": \"$BRIDGE_APP_ID\",
    \"name\": \"Base Bridge: Deposits\",
    \"description\": \"Base L2 bridge deposit events\",
    \"webhook_url\": \"$WEBHOOK_URL\",
    \"chain_ids\": [1],
    \"contract_addresses\": [\"0x49048044d57e1c92a77f79988d21fa8faf74e97e\"],
    \"event_signatures\": [\"0x73d170910aba9e6d50b102db522b1dbcd796216f5128b445aa2135272886497e\"]
  }" | jq -r '.name' && echo "  ✅ Base Deposits"

echo ""
echo "✅ Demo account setup complete!"
echo ""
echo "📊 Summary:"
curl -s -X GET "$API_URL/endpoints" -H "Authorization: Bearer $TOKEN" | jq -r '"\(.total) endpoints across \(.endpoints | group_by(.application_id) | length) applications"'
echo ""
echo "🌐 Login at: http://104.248.15.178:3002"
echo "📧 Email: $DEMO_EMAIL"
echo "🔑 Password: $DEMO_PASSWORD"
