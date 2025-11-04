#!/bin/bash
# Start all ETHHook services for real network data testing
# This includes Event Ingestor, Message Processor, Webhook Delivery, Admin API, and Frontend

set -e  # Exit on error

cd "$(dirname "$0")/.."

echo "🚀 Starting ETHHook Full Stack with Real Network Data"
echo "======================================================"
echo ""

# Load environment variables
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found"
    exit 1
fi

export $(cat .env | grep -v '^#' | xargs)

# Clean up old processes
echo "🧹 Cleaning up old processes..."
pkill -9 ethhook-admin-api 2>/dev/null || true
pkill -9 event-ingestor 2>/dev/null || true
pkill -9 ethhook-message-processor 2>/dev/null || true
pkill -9 ethhook-webhook-delivery 2>/dev/null || true
pkill -9 trunk 2>/dev/null || true
sleep 2

# Ensure log directory exists
mkdir -p /tmp/ethhook-logs

echo ""
echo "📦 Starting Backend Services..."
echo "================================"

# 1. Start Event Ingestor (listens to Sepolia blockchain)
echo "🔗 Starting Event Ingestor (Sepolia testnet)..."
nohup cargo run --bin event-ingestor > /tmp/ethhook-logs/event-ingestor.log 2>&1 &
EVENT_INGESTOR_PID=$!
echo "   ✓ PID: $EVENT_INGESTOR_PID"
sleep 3

# 2. Start Message Processor (matches events to endpoints)
echo "🔍 Starting Message Processor (event matching)..."
nohup cargo run --bin ethhook-message-processor > /tmp/ethhook-logs/message-processor.log 2>&1 &
MESSAGE_PROCESSOR_PID=$!
echo "   ✓ PID: $MESSAGE_PROCESSOR_PID"
sleep 3

# 3. Start Webhook Delivery (delivers webhooks)
echo "📤 Starting Webhook Delivery (HTTP delivery)..."
nohup cargo run --bin ethhook-webhook-delivery > /tmp/ethhook-logs/webhook-delivery.log 2>&1 &
WEBHOOK_DELIVERY_PID=$!
echo "   ✓ PID: $WEBHOOK_DELIVERY_PID"
sleep 3

# 4. Start Admin API (REST API)
echo "🌐 Starting Admin API (port 3000)..."
nohup cargo run --bin ethhook-admin-api > /tmp/ethhook-logs/admin-api.log 2>&1 &
ADMIN_API_PID=$!
echo "   ✓ PID: $ADMIN_API_PID"
sleep 4

echo ""
echo "🎨 Starting Frontend..."
echo "======================="

# 5. Start Frontend (Next.js UI)
echo "🖥️  Starting Frontend (port 3002)..."
cd ui
nohup npm run dev > /tmp/ethhook-logs/frontend.log 2>&1 &
FRONTEND_PID=$!
echo "   ✓ PID: $FRONTEND_PID"
cd ..
sleep 6

echo ""
echo "✅ All Services Started!"
echo "========================"
echo ""
echo "📊 Service URLs:"
echo "  • Frontend:  http://localhost:3002"
echo "  • Admin API: http://localhost:3000"
echo ""
echo "🔧 Backend Services (running in background):"
echo "  • Event Ingestor:    PID $EVENT_INGESTOR_PID (Sepolia blockchain → Redis Streams)"
echo "  • Message Processor: PID $MESSAGE_PROCESSOR_PID (Redis Streams → Event Matching → Webhook Jobs)"
echo "  • Webhook Delivery:  PID $WEBHOOK_DELIVERY_PID (Webhook Jobs → HTTP Delivery)"
echo "  • Admin API:         PID $ADMIN_API_PID (REST API)"
echo "  • Frontend:          PID $FRONTEND_PID (Next.js UI)"
echo ""
echo "📝 Logs (real-time monitoring):"
echo "  • Event Ingestor:    tail -f /tmp/ethhook-logs/event-ingestor.log"
echo "  • Message Processor: tail -f /tmp/ethhook-logs/message-processor.log"
echo "  • Webhook Delivery:  tail -f /tmp/ethhook-logs/webhook-delivery.log"
echo "  • Admin API:         tail -f /tmp/ethhook-logs/admin-api.log"
echo "  • Frontend:          tail -f /tmp/ethhook-logs/frontend.log"
echo ""
echo "🧪 Quick Health Checks:"
echo "  • Admin API:        curl http://localhost:3000/api/v1/applications"
echo "  • Event Stats:      curl http://localhost:3000/api/v1/statistics/dashboard"
echo "  • Process Status:   ps aux | grep ethhook"
echo ""
echo "🛑 To stop all services:"
echo "  • ./scripts/stop_all_services.sh"
echo "  • OR: pkill -9 ethhook; pkill -9 trunk"
echo ""
echo "📈 Real Network Data:"
echo "   Your Event Ingestor is now connected to Sepolia testnet."
echo "   It will automatically detect Transfer events on your monitored contracts:"
echo "   - WETH: 0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9"
echo "   - USDC: 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
echo ""
echo "   Check the dashboard for live statistics as events arrive!"
echo ""
