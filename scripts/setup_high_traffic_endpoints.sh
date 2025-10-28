#!/bin/bash
# Setup high-traffic test endpoints for load testing
# This creates multiple endpoints to simulate realistic load

set -e

echo "🚀 Setting up high-traffic test endpoints..."

# Docker exec command for PostgreSQL
DOCKER_EXEC="/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c"

# First, get or create test application (using test@ethhook.com user)
echo "📦 Creating test application..."
$DOCKER_EXEC "
INSERT INTO applications (id, user_id, name, description, webhook_secret, api_key)
VALUES (
    '5e81c0a4-ab19-49ea-a079-9aae99ddbcb1',
    '00000000-0000-0000-0000-000000000001',
    'Load Test App',
    'High-traffic load testing',
    'load_test_webhook_secret_12345',
    'test_key_12345'
)
ON CONFLICT (id) DO NOTHING;
"

# Create multiple endpoints for different contract patterns
echo "🎯 Creating test endpoints..."

# First, delete existing load test endpoints to avoid duplicates
$DOCKER_EXEC "
DELETE FROM endpoints
WHERE application_id = '5e81c0a4-ab19-49ea-a079-9aae99ddbcb1';
" 2>/dev/null || true

# Insert new endpoints
$DOCKER_EXEC "
INSERT INTO endpoints (application_id, name, webhook_url, hmac_secret, contract_addresses, event_signatures, chain_ids, is_active, description) VALUES
('5e81c0a4-ab19-49ea-a079-9aae99ddbcb1', '💵 USDC Transfers', 'http://host.docker.internal:8000/webhook', 'usdc_secret_abc123def456ghi789jkl012mno345pqr678stu901vwx234yz', ARRAY['0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238'], ARRAY['Transfer(address indexed,address indexed,uint256)'], ARRAY[11155111], true, 'USDC transfers'),
('5e81c0a4-ab19-49ea-a079-9aae99ddbcb1', '💎 WETH All Events', 'http://host.docker.internal:8000/webhook', 'weth_secret_abc123def456ghi789jkl012mno345pqr678stu901vwx234yz', ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9'], ARRAY['Transfer(address indexed,address indexed,uint256)', 'Deposit(address indexed,uint256)', 'Withdrawal(address indexed,uint256)'], ARRAY[11155111], true, 'WETH events'),
('5e81c0a4-ab19-49ea-a079-9aae99ddbcb1', '🔥 DAI Transfers', 'http://host.docker.internal:8000/webhook', 'dai_secret_abc123def456ghi789jkl012mno345pqr678stu901vwx234yza', ARRAY['0xFF34B3d4Aee8ddCd6F9AFFFB6Fe49bD371b8a357'], ARRAY['Transfer(address indexed,address indexed,uint256)'], ARRAY[11155111], true, 'DAI transfers'),
('5e81c0a4-ab19-49ea-a079-9aae99ddbcb1', '🔗 LINK Transfers', 'http://host.docker.internal:8000/webhook', 'link_secret_abc123def456ghi789jkl012mno345pqr678stu901vwx234yz', ARRAY['0x779877A7B0D9E8603169DdbD7836e478b4624789'], ARRAY['Transfer(address indexed,address indexed,uint256)'], ARRAY[11155111], true, 'LINK transfers'),
('5e81c0a4-ab19-49ea-a079-9aae99ddbcb1', '🦄 Uniswap Swaps', 'http://host.docker.internal:8000/webhook', 'swap_secret_abc123def456ghi789jkl012mno345pqr678stu901vwx234yz', ARRAY['0x0227628f3F023bb0B980b67D528571c95c6DaC1c'], ARRAY['Swap(address indexed,address indexed,int256,int256,uint160,uint128,int24)'], ARRAY[11155111], true, 'Uniswap swaps');
"

echo "✅ High-traffic endpoints created!"
echo ""
echo "📊 Endpoints configured:"
echo "  - USDC Transfers (0x1c7D...)"
echo "  - WETH All Events (0x7b79...)"
echo "  - DAI Transfers (0xFF34...)"
echo "  - LINK Transfers (0x7798...)"
echo "  - Uniswap Swaps (0x0227...)"
echo ""
echo "🎯 Ready for load testing!"
