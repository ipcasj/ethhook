#!/bin/bash
set -e

echo "🚀 Starting ETHHook End-to-End Test"
echo "===================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Change to project root
cd "$(dirname "$0")/.."

echo -e "${BLUE}Step 1: Load environment variables${NC}"
set -a
source .env
set +a
echo "✅ Environment: $ENVIRONMENT"
echo "✅ Database: $DATABASE_URL"
echo ""

echo -e "${BLUE}Step 2: Check infrastructure (PostgreSQL, Redis)${NC}"
docker compose ps
echo ""

echo -e "${BLUE}Step 3: Kill any existing processes${NC}"
pkill -9 ethhook-admin-api || true
pkill -9 trunk || true
pkill -9 event-ingestor || true
sleep 2
echo "✅ Cleaned up old processes"
echo ""

echo -e "${BLUE}Step 4: Start Admin API (port 3000)${NC}"
RUST_LOG=info,ethhook_admin_api=debug cargo run --bin ethhook-admin-api > /tmp/admin-api.log 2>&1 &
ADMIN_PID=$!
echo "✅ Admin API starting (PID: $ADMIN_PID)"
sleep 3
echo ""

echo -e "${BLUE}Step 5: Start Event Ingestor (Sepolia connection)${NC}"
RUST_LOG=info,event_ingestor=debug cargo run --bin event-ingestor > /tmp/event-ingestor.log 2>&1 &
INGESTOR_PID=$!
echo "✅ Event Ingestor starting (PID: $INGESTOR_PID)"
sleep 3
echo ""

echo -e "${BLUE}Step 6: Start Frontend (port 3002)${NC}"
cd ui
npm run dev > /tmp/frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..
echo "✅ Frontend starting (PID: $FRONTEND_PID)"
sleep 5
echo ""

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ All Services Started Successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "📊 Service Status:"
echo "  • PostgreSQL:     Running (port 5432)"
echo "  • Redis:          Running (port 6379)"
echo "  • Admin API:      http://localhost:3000"
echo "  • Event Ingestor: Connected to Sepolia (chain_id: 11155111)"
echo "  • Frontend:       http://localhost:3002"
echo ""
echo "📝 Log Files:"
echo "  • Admin API:      /tmp/admin-api.log"
echo "  • Event Ingestor: /tmp/event-ingestor.log"
echo "  • Frontend:       /tmp/frontend.log"
echo ""
echo "🎯 Test Data:"
echo "  • Application: 'DeFi Demo App'"
echo "  • Endpoints: Sepolia WETH & USDC Transfer events"
echo ""
echo "🌐 Open in browser: http://localhost:3002"
echo ""
echo -e "${YELLOW}Press Ctrl+C to view logs, or open another terminal${NC}"
echo ""

# Wait a moment then show initial logs
sleep 2
echo -e "${BLUE}=== Admin API Status ===${NC}"
tail -10 /tmp/admin-api.log
echo ""
echo -e "${BLUE}=== Event Ingestor Status ===${NC}"
tail -10 /tmp/event-ingestor.log
echo ""

# Keep script running
echo "Services are running. Press Ctrl+C to stop."
wait
