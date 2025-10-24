#!/bin/bash

echo "🚀 Setting up High-Traffic Sepolia Endpoints"
echo "=============================================="

# Get the application ID
APP_ID=$(/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT id FROM applications WHERE name = 'My Application' LIMIT 1;" | xargs)

if [ -z "$APP_ID" ]; then
    echo "❌ No application found. Creating one..."
    APP_ID=$(/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "INSERT INTO applications (name, description, user_id) VALUES ('My Application', 'Demo Application', (SELECT id FROM users LIMIT 1)) RETURNING id;" | xargs)
fi

echo "📋 Application ID: $APP_ID"
echo ""

# Clear old endpoints
echo "🧹 Clearing old endpoints..."
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "DELETE FROM endpoints WHERE application_id = '$APP_ID';" > /dev/null

echo "📝 Creating multiple high-traffic endpoints..."
echo ""

# 1. Sepolia USDC - Very Active ERC20
echo "1️⃣  Creating USDC endpoint..."
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description
) VALUES (
    '$APP_ID',
    '💵 USDC Transfers',
    'http://host.docker.internal:8000/webhook',
    ARRAY['0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238'],
    ARRAY['Transfer(address indexed,address indexed,uint256)'],
    ARRAY[11155111],
    true,
    'USDC token transfers on Sepolia - High volume'
);" > /dev/null
echo "   ✅ USDC endpoint created"

# 2. Sepolia WETH - Very Active
echo "2️⃣  Creating WETH endpoint..."
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description
) VALUES (
    '$APP_ID',
    '💎 WETH All Events',
    'http://host.docker.internal:8000/webhook',
    ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9'],
    ARRAY['Transfer(address indexed,address indexed,uint256)', 'Deposit(address indexed,uint256)', 'Withdrawal(address indexed,uint256)'],
    ARRAY[11155111],
    true,
    'WETH transfers, deposits, and withdrawals - High volume'
);" > /dev/null
echo "   ✅ WETH endpoint created"

# 3. Any Transfer on popular tokens (catch-all for ERC20 activity)
echo "3️⃣  Creating Multi-Token endpoint..."
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description
) VALUES (
    '$APP_ID',
    '🔥 DAI Transfers',
    'http://host.docker.internal:8000/webhook',
    ARRAY['0xFF34B3d4Aee8ddCd6F9AFFFB6Fe49bD371b8a357'],
    ARRAY['Transfer(address indexed,address indexed,uint256)'],
    ARRAY[11155111],
    true,
    'DAI token transfers on Sepolia'
);" > /dev/null
echo "   ✅ DAI endpoint created"

# 4. Uniswap V3 Swap events - Very high traffic
echo "4️⃣  Creating Uniswap V3 endpoint..."
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description
) VALUES (
    '$APP_ID',
    '🦄 Uniswap V3 Swaps',
    'http://host.docker.internal:8000/webhook',
    ARRAY['0x0227628f3F023bb0B980b67D528571c95c6DaC1c'],
    ARRAY['Swap(address indexed,address indexed,int256,int256,uint160,uint128,int24)'],
    ARRAY[11155111],
    true,
    'Uniswap V3 swap events - Very high volume'
);" > /dev/null
echo "   ✅ Uniswap endpoint created"

# 5. Generic Transfer events (will catch ANY ERC20 transfer with this signature)
echo "5️⃣  Creating Link Token endpoint..."
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description
) VALUES (
    '$APP_ID',
    '🔗 LINK Transfers',
    'http://host.docker.internal:8000/webhook',
    ARRAY['0x779877A7B0D9E8603169DdbD7836e478b4624789'],
    ARRAY['Transfer(address indexed,address indexed,uint256)'],
    ARRAY[11155111],
    true,
    'ChainLink token transfers on Sepolia'
);" > /dev/null
echo "   ✅ LINK endpoint created"

echo ""
echo "✅ All endpoints created!"
echo ""

# Show summary
echo "📊 Endpoint Summary:"
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
SELECT
    name,
    array_length(contract_addresses, 1) as contracts,
    array_length(event_signatures, 1) as events,
    is_active as active
FROM endpoints
WHERE application_id = '$APP_ID'
ORDER BY created_at DESC;
"

echo ""
echo "🎯 System is now monitoring 5 high-traffic endpoints!"
echo "   Expected event rate: 10-50 events per minute"
echo ""
echo "▶️  Next: Watch for incoming events!"
