#!/bin/bash
# Quick Start Script for Real Webhook Testing
# Starts webhook receiver and monitors all services

set -e

echo "🚀 EthHook Real Webhook Testing - Quick Start"
echo "=============================================="
echo ""

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: Python 3 is required"
    echo "   Please install Python 3 and try again"
    exit 1
fi

# Check if services are running
echo "📊 Checking services..."
if ! /usr/local/bin/docker ps | grep -q ethhook-postgres; then
    echo "❌ Error: PostgreSQL not running"
    echo "   Please start services with: docker compose up -d"
    exit 1
fi

if ! /usr/local/bin/docker ps | grep -q ethhook-event-ingestor; then
    echo "❌ Error: Event Ingestor not running"
    echo "   Please start services with: docker compose up -d"
    exit 1
fi

echo "✅ All services running!"
echo ""

# Show endpoint info
echo "📍 Webhook Endpoint Configuration:"
echo "   ID: aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
echo "   URL: http://host.docker.internal:8000/webhook"
echo "   Contract: 0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9 (Sepolia WETH)"
echo "   Event: Transfer(address,address,uint256)"
echo "   Chain: Sepolia Testnet (11155111)"
echo ""

# Show monitoring commands
echo "📺 Open these in separate terminals to monitor:"
echo ""
echo "   Terminal 1 - Event Ingestor:"
echo "   /usr/local/bin/docker logs -f ethhook-event-ingestor | grep -E 'Processing block|Transfer'"
echo ""
echo "   Terminal 2 - Message Processor:"
echo "   /usr/local/bin/docker logs -f ethhook-message-processor | grep -E 'Matched|endpoint'"
echo ""
echo "   Terminal 3 - Webhook Delivery:"
echo "   /usr/local/bin/docker logs -f ethhook-webhook-delivery | grep -E 'Sending|delivered'"
echo ""
echo "   Terminal 4 - Grafana:"
echo "   open http://localhost:3001"
echo ""

# Start webhook receiver
echo "🎯 Starting Real Webhook Receiver on port 8000..."
echo "   (Press Ctrl+C to stop)"
echo ""
echo "=============================================="
echo ""

# Run webhook receiver
exec python3 webhook_receiver.py 8000
