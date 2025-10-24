#!/bin/bash
# Start EthHook UI for Demo/Client Presentations

echo "🎬 Starting EthHook UI Demo Environment"
echo "========================================"
echo ""

# Check if services are running
echo "📊 Checking backend services..."
if ! /usr/local/bin/docker ps | grep -q ethhook-admin-api; then
    echo "❌ Backend services not running!"
    echo "   Starting services..."
    cd /Users/igor/rust_projects/capstone0
    /usr/local/bin/docker compose up -d
    echo "   Waiting for services to be healthy..."
    sleep 10
fi

echo "✅ Backend services running"
echo ""

# Clean frontend cache
echo "🧹 Cleaning frontend cache..."
cd /Users/igor/rust_projects/capstone0/crates/leptos-portal
rm -rf dist/ .parcel-cache/ 2>/dev/null
echo "✅ Cache cleaned"
echo ""

# Start frontend
echo "🚀 Starting frontend portal..."
echo "   URL: http://localhost:3002"
echo "   (Browser will open automatically)"
echo ""
echo "📝 For demo:"
echo "   1. Register new user: demo@ethhook.io"
echo "   2. Create application"
echo "   3. Add endpoint with URL: http://host.docker.internal:8000/webhook"
echo "   4. Start webhook receiver: ./test_real_webhooks.sh"
echo "   5. Wait for event and show client!"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Start trunk with auto-open
trunk serve --port 3002 --open
