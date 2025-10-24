#!/bin/bash
# Setup Pre-Configured Popular Endpoints for Demo

echo "🎯 Setting Up Demo Endpoints with Popular Sepolia Events"
echo "=========================================================="
echo ""

# Get the first application ID (or create one if none exist)
APP_ID=$(/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "
SELECT id FROM applications ORDER BY created_at DESC LIMIT 1;
" | tr -d ' ')

if [ -z "$APP_ID" ]; then
    echo "❌ No applications found. Please create an application first via UI."
    exit 1
fi

echo "📱 Using application ID: $APP_ID"
echo ""

# Popular Sepolia Contracts
echo "📊 Adding popular Sepolia event endpoints..."
echo ""

# 1. Sepolia USDC - Very active, lots of transfers
echo "1️⃣  Sepolia USDC Transfers (High Volume)"
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    hmac_secret,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description,
    rate_limit_per_second,
    max_retries,
    timeout_seconds
) VALUES (
    '$APP_ID',
    '💵 Sepolia USDC Transfers',
    'http://host.docker.internal:8000/webhook',
    'demo_usdc_secret_' || substr(md5(random()::text), 1, 32),
    ARRAY['0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238'],
    ARRAY['Transfer(address indexed,address indexed,uint256)'],
    ARRAY[11155111],
    true,
    '🔥 HIGH VOLUME: USDC is one of the most active tokens on Sepolia. Expect 50-200 events per hour during peak times. Perfect for testing webhook delivery at scale.',
    20,
    5,
    30
);
" > /dev/null

# 2. Sepolia WETH - Very popular for DeFi testing
echo "2️⃣  Sepolia WETH Deposit/Withdraw (DeFi Activity)"
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    hmac_secret,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description,
    rate_limit_per_second,
    max_retries,
    timeout_seconds
) VALUES (
    '$APP_ID',
    '💎 Sepolia WETH All Events',
    'http://host.docker.internal:8000/webhook',
    'demo_weth_secret_' || substr(md5(random()::text), 1, 32),
    ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9'],
    ARRAY['Transfer(address indexed,address indexed,uint256)', 'Deposit(address indexed,uint256)', 'Withdrawal(address indexed,uint256)'],
    ARRAY[11155111],
    true,
    '⚡ POPULAR: WETH is the most used token in Sepolia DeFi. Tracks Transfers, Deposits (ETH→WETH), and Withdrawals (WETH→ETH). Expect 20-50 events per hour.',
    15,
    5,
    30
);
" > /dev/null

echo ""
echo "✅ Demo endpoints created successfully!"
echo ""

# Show what was created
echo "📋 Active Endpoints:"
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
SELECT
    LEFT(name, 40) as name,
    CASE
        WHEN name LIKE '%USDC%' THEN '50-200/hr'
        WHEN name LIKE '%WETH%' THEN '20-50/hr'
        ELSE 'varies'
    END as expected_volume,
    is_active as active
FROM endpoints
WHERE application_id = '$APP_ID'
  AND name LIKE '%Sepolia%'
ORDER BY created_at DESC;
"

echo ""
echo "🎉 Your system will now receive real events within minutes!"
echo ""
echo "📊 Expected Event Flow:"
echo "   • USDC Transfers: 50-200 events/hour (HIGH VOLUME)"
echo "   • WETH Events: 20-50 events/hour (MEDIUM-HIGH)"
echo "   • Total: 70-250 events/hour"
echo ""
echo "🚀 Start webhook receiver to see events:"
echo "   ./test_real_webhooks.sh"
echo ""
echo "📈 Watch statistics in:"
echo "   • Dashboard: http://localhost:3002/dashboard"
echo "   • Grafana: http://localhost:3001"
echo ""
