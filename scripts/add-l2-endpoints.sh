#!/bin/bash

# Add L2 Chain Support to Demo Account Endpoints
# This script updates existing endpoints to include Arbitrum, Optimism, and Base chains
# for high-volume DeFi contracts

set -e

API_URL="http://104.248.15.178:3000/api/v1"
WEBHOOK_URL="http://104.248.15.178:8000/webhook"

echo "🔐 Logging in as demo user..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "demo@ethhook.com",
    "password": "Demo1234!"
  }')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ Login failed!"
  echo "$LOGIN_RESPONSE"
  exit 1
fi

echo "✅ Logged in successfully"

# ERC20 Transfer event signature
TRANSFER_SIG="0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"

# WETH Deposit/Withdrawal signatures
DEPOSIT_SIG="0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c"
WITHDRAWAL_SIG="0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65"

echo ""
echo "📊 Fetching existing applications..."
APPS_RESPONSE=$(curl -s -X GET "$API_URL/applications" \
  -H "Authorization: Bearer $TOKEN")

DEFI_APP_ID=$(echo $APPS_RESPONSE | jq -r '.applications[] | select(.name == "DeFi Protocol Monitor") | .id')
BRIDGE_APP_ID=$(echo $APPS_RESPONSE | jq -r '.applications[] | select(.name == "Multi-Chain Bridge Monitor") | .id')

if [ -z "$DEFI_APP_ID" ]; then
  echo "❌ DeFi Protocol Monitor application not found!"
  exit 1
fi

echo "✅ Found DeFi Protocol Monitor: $DEFI_APP_ID"
echo "✅ Found Multi-Chain Bridge Monitor: $BRIDGE_APP_ID"

# Contract addresses for each chain
# USDC addresses
USDC_ETHEREUM="0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
USDC_ARBITRUM="0xaf88d065e77c8cc2239327c5edb3a432268e5831"
USDC_OPTIMISM="0x0b2c639c8136258849498ef890c8c58e5b5e95cd"
USDC_BASE="0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"

# USDT addresses
USDT_ETHEREUM="0xdac17f958d2ee523a2206206994597c13d831ec7"
USDT_ARBITRUM="0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9"
USDT_OPTIMISM="0x94b008aa00579c1307b0ef2c499ad98a8ce58e58"
# Base doesn't have native USDT bridged yet

# WETH addresses
WETH_ETHEREUM="0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
WETH_ARBITRUM="0x82af49447d8a07e3bd95bd0d56f35241523fbab1"
WETH_OPTIMISM="0x4200000000000000000000000000000000000006"
WETH_BASE="0x4200000000000000000000000000000000000006"

echo ""
echo "💎 Creating multi-chain USDC endpoint..."
curl -s -X POST "$API_URL/endpoints" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$DEFI_APP_ID"'",
    "name": "USDC Transfers (All Chains)",
    "description": "Monitor USDC transfers across Ethereum, Arbitrum, Optimism, and Base",
    "webhook_url": "'"$WEBHOOK_URL"'",
    "chain_ids": [1, 42161, 10, 8453],
    "contract_addresses": ["'"$USDC_ETHEREUM"'", "'"$USDC_ARBITRUM"'", "'"$USDC_OPTIMISM"'", "'"$USDC_BASE"'"],
    "event_signatures": ["'"$TRANSFER_SIG"'"],
    "is_active": true
  }' | jq -r '.id' > /dev/null && echo "  ✅ USDC (All Chains)" || echo "  ⚠️  USDC (may already exist)"

echo "💎 Creating multi-chain USDT endpoint..."
curl -s -X POST "$API_URL/endpoints" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$DEFI_APP_ID"'",
    "name": "USDT Transfers (All Chains)",
    "description": "Monitor USDT transfers across Ethereum, Arbitrum, and Optimism",
    "webhook_url": "'"$WEBHOOK_URL"'",
    "chain_ids": [1, 42161, 10],
    "contract_addresses": ["'"$USDT_ETHEREUM"'", "'"$USDT_ARBITRUM"'", "'"$USDT_OPTIMISM"'"],
    "event_signatures": ["'"$TRANSFER_SIG"'"],
    "is_active": true
  }' | jq -r '.id' > /dev/null && echo "  ✅ USDT (All Chains)" || echo "  ⚠️  USDT (may already exist)"

echo "💎 Creating multi-chain WETH endpoint..."
curl -s -X POST "$API_URL/endpoints" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$DEFI_APP_ID"'",
    "name": "WETH Activity (All Chains)",
    "description": "Monitor WETH deposits, withdrawals, and transfers across all chains",
    "webhook_url": "'"$WEBHOOK_URL"'",
    "chain_ids": [1, 42161, 10, 8453],
    "contract_addresses": ["'"$WETH_ETHEREUM"'", "'"$WETH_ARBITRUM"'", "'"$WETH_OPTIMISM"'", "'"$WETH_BASE"'"],
    "event_signatures": ["'"$TRANSFER_SIG"'", "'"$DEPOSIT_SIG"'", "'"$WITHDRAWAL_SIG"'"],
    "is_active": true
  }' | jq -r '.id' > /dev/null && echo "  ✅ WETH (All Chains)" || echo "  ⚠️  WETH (may already exist)"

echo ""
echo "🌉 Creating native bridge endpoints for L2 activity..."

# Arbitrum Bridge on L2
ARBITRUM_L2_BRIDGE="0x0000000000000000000000000000000000000064"
curl -s -X POST "$API_URL/endpoints" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$BRIDGE_APP_ID"'",
    "name": "Arbitrum: L2 Withdrawals",
    "description": "Monitor withdrawal events on Arbitrum L2",
    "webhook_url": "'"$WEBHOOK_URL"'",
    "chain_ids": [42161],
    "contract_addresses": ["'"$ARBITRUM_L2_BRIDGE"'"],
    "event_signatures": ["'"$TRANSFER_SIG"'"],
    "is_active": true
  }' | jq -r '.id' > /dev/null && echo "  ✅ Arbitrum L2 Bridge" || echo "  ⚠️  Arbitrum L2 Bridge (may already exist)"

# Optimism Bridge on L2
OPTIMISM_L2_BRIDGE="0x4200000000000000000000000000000000000010"
curl -s -X POST "$API_URL/endpoints" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$BRIDGE_APP_ID"'",
    "name": "Optimism: L2 Bridge Activity",
    "description": "Monitor bridge events on Optimism L2",
    "webhook_url": "'"$WEBHOOK_URL"'",
    "chain_ids": [10],
    "contract_addresses": ["'"$OPTIMISM_L2_BRIDGE"'"],
    "event_signatures": ["'"$TRANSFER_SIG"'"],
    "is_active": true
  }' | jq -r '.id' > /dev/null && echo "  ✅ Optimism L2 Bridge" || echo "  ⚠️  Optimism L2 Bridge (may already exist)"

# Base Bridge on L2
BASE_L2_BRIDGE="0x4200000000000000000000000000000000000010"
curl -s -X POST "$API_URL/endpoints" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$BRIDGE_APP_ID"'",
    "name": "Base: L2 Bridge Activity",
    "description": "Monitor bridge events on Base L2",
    "webhook_url": "'"$WEBHOOK_URL"'",
    "chain_ids": [8453],
    "contract_addresses": ["'"$BASE_L2_BRIDGE"'"],
    "event_signatures": ["'"$TRANSFER_SIG"'"],
    "is_active": true
  }' | jq -r '.id' > /dev/null && echo "  ✅ Base L2 Bridge" || echo "  ⚠️  Base L2 Bridge (may already exist)"

echo ""
echo "✅ L2 endpoints added!"
echo ""
echo "📊 Summary: Added multi-chain support for high-volume contracts"
echo "   - USDC: Ethereum + Arbitrum + Optimism + Base"
echo "   - USDT: Ethereum + Arbitrum + Optimism"
echo "   - WETH: Ethereum + Arbitrum + Optimism + Base"
echo "   - Bridge Activity: All L2 chains"
echo ""
echo "⏱️  Wait 1-2 minutes for events to start flowing..."
echo "🌐 Then check: http://104.248.15.178:3002"
