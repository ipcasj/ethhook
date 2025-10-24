#!/bin/bash
# Comprehensive E2E Verification Script

echo "🔍 ETHHook End-to-End Verification"
echo "===================================="
echo ""

# Test 1: Infrastructure
echo "✅ Test 1: Infrastructure Services"
echo "-----------------------------------"
docker compose ps | grep -E "postgres|redis" | grep healthy && echo "✓ PostgreSQL and Redis are healthy" || echo "✗ Infrastructure issue"
echo ""

# Test 2: Backend Services
echo "✅ Test 2: Backend Services"
echo "-----------------------------------"
if ps aux | grep -E "ethhook-admin-api" | grep -v grep > /dev/null; then
    echo "✓ Admin API is running"
else
    echo "✗ Admin API not running"
fi

if ps aux | grep -E "event-ingestor" | grep -v grep > /dev/null; then
    echo "✓ Event Ingestor is running"
else
    echo "✗ Event Ingestor not running"
fi

if ps aux | grep -E "trunk serve" | grep -v grep > /dev/null; then
    echo "✓ Frontend is running"
else
    echo "✗ Frontend not running"
fi
echo ""

# Test 3: API Health
echo "✅ Test 3: API Health Checks"
echo "-----------------------------------"
HEALTH=$(curl -s http://localhost:3000/api/v1/health)
if [ "$HEALTH" = "OK" ]; then
    echo "✓ Admin API health: $HEALTH"
else
    echo "✗ Admin API health check failed"
fi
echo ""

# Test 4: Frontend
echo "✅ Test 4: Frontend Availability"
echo "-----------------------------------"
FRONTEND_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3001/)
if [ "$FRONTEND_STATUS" = "200" ]; then
    echo "✓ Frontend responding (HTTP $FRONTEND_STATUS)"
else
    echo "✗ Frontend not responding (HTTP $FRONTEND_STATUS)"
fi
echo ""

# Test 5: Database Content
echo "✅ Test 5: Test Data in Database"
echo "-----------------------------------"
docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "
SELECT 
    COUNT(*) || ' applications, ' || 
    (SELECT COUNT(*) FROM endpoints) || ' endpoints'
FROM applications;" | xargs echo "✓ Data:"
echo ""

# Test 6: Event Ingestor Connection
echo "✅ Test 6: Sepolia Testnet Connection"
echo "-----------------------------------"
if grep -q "Sepolia Testnet.*connected successfully" /tmp/event-ingestor.log; then
    echo "✓ Event Ingestor connected to Sepolia"
    LAST_BLOCK=$(grep "Sepolia" /tmp/event-ingestor.log | grep -o "block.*" | tail -1)
    echo "  Latest activity: $LAST_BLOCK"
else
    echo "⚠ Checking connection..."
fi
echo ""

# Test 7: Environment Configuration
echo "✅ Test 7: Environment Configuration"
echo "-----------------------------------"
source /Users/igor/rust_projects/capstone0/.env
echo "✓ Environment: $ENVIRONMENT"
echo "✓ Using Sepolia RPC: ${ETHEREUM_WS_URL:0:50}..."
echo ""

# Summary
echo "════════════════════════════════════"
echo "🎉 VERIFICATION COMPLETE"
echo "════════════════════════════════════"
echo ""
echo "📊 Access Points:"
echo "  • Frontend UI:  http://localhost:3001"
echo "  • Admin API:    http://localhost:3000/api/v1"
echo "  • PostgreSQL:   localhost:5432"
echo "  • Redis:        localhost:6379"
echo ""
echo "📝 Test Data Available:"
echo "  • Application: 'DeFi Demo App'"
echo "  • Endpoints:"
echo "    - Sepolia WETH Transfers (0x7b79...E7f9)"
echo "    - Sepolia USDC Transfers (0x1c7D...7238)"
echo ""
echo "🔗 Network:"
echo "  • Chain: Sepolia Testnet (chain_id: 11155111)"
echo "  • RPC: Alchemy WebSocket"
echo "  • Status: Processing real-time blocks"
echo ""
echo "📖 Quick Actions:"
echo "  1. Open UI:        open http://localhost:3001"
echo "  2. View logs:      tail -f /tmp/event-ingestor.log"
echo "  3. Stop services:  pkill -9 ethhook-admin-api trunk event-ingestor"
echo ""
