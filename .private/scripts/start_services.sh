#!/bin/bash
# Simplified E2E Test Startup - Keeps services running in background

cd "$(dirname "$0")/.."

echo "🚀 Starting ETHHook Services"
echo "=============================="

# Load environment
export $(cat .env | grep -v '^#' | xargs)

# Clean up old processes
echo "🧹 Cleaning up old processes..."
pkill -9 ethhook-admin-api 2>/dev/null || true
pkill -9 trunk 2>/dev/null || true  
pkill -9 event-ingestor 2>/dev/null || true
sleep 2

# Start Admin API
echo "🚀 Starting Admin API on port 3000..."
nohup cargo run --bin ethhook-admin-api > /tmp/admin-api.log 2>&1 &
sleep 4

# Start Event Ingestor
echo "🚀 Starting Event Ingestor (Sepolia)..."
nohup cargo run --bin event-ingestor > /tmp/event-ingestor.log 2>&1 &
sleep 4

# Start Frontend
echo "🚀 Starting Frontend on port 3001..."
cd crates/leptos-portal
nohup trunk serve --port 3001 > /tmp/frontend.log 2>&1 &
cd ../..
sleep 6

echo ""
echo "✅ Services Started!"
echo "===================="
echo ""
echo "📊 Service URLs:"
echo "  • Frontend:  http://localhost:3001"
echo "  • Admin API: http://localhost:3000"
echo ""
echo "📝 Logs:"
echo "  • tail -f /tmp/admin-api.log"
echo "  • tail -f /tmp/event-ingestor.log"
echo "  • tail -f /tmp/frontend.log"
echo ""
echo "🧪 Quick Tests:"
echo "  • curl http://localhost:3000/api/applications"
echo "  • curl http://localhost:3001"
echo ""
echo "🛑 To stop all services:"
echo "  pkill -9 ethhook-admin-api trunk event-ingestor"
echo ""
