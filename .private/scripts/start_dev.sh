#!/bin/bash

# EthHook Development Server Startup Script
# This script starts all required services for local development

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🚀 Starting EthHook Development Environment..."
echo ""

# Check if Docker services are running
echo "📦 Checking Docker services..."
if ! docker ps | grep -q ethhook-postgres; then
    echo "⚠️  PostgreSQL not running. Starting Docker services..."
    docker-compose up -d
    echo "⏳ Waiting for services to be healthy..."
    sleep 5
fi

# Load environment variables
export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
export JWT_SECRET="test-secret-for-ci"
export ADMIN_API_PORT="8080"

# Start Admin API
echo ""
echo "🔧 Starting Admin API on port 8080..."
cargo run --bin ethhook-admin-api > /tmp/ethhook-admin-api.log 2>&1 &
ADMIN_PID=$!
echo "   Admin API PID: $ADMIN_PID"

# Wait for Admin API to be ready
echo "⏳ Waiting for Admin API to start..."
for i in {1..30}; do
    if curl -s http://localhost:8080/api/v1/health > /dev/null 2>&1; then
        echo "✅ Admin API is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Admin API failed to start. Check logs at /tmp/ethhook-admin-api.log"
        exit 1
    fi
    sleep 1
done

# Start Frontend
echo ""
echo "🎨 Starting Leptos Portal on port 3002..."
cd crates/leptos-portal
trunk serve --port 3002 > /tmp/ethhook-frontend.log 2>&1 &
FRONTEND_PID=$!
echo "   Frontend PID: $FRONTEND_PID"
cd "$PROJECT_ROOT"

# Wait for Frontend to be ready
echo "⏳ Waiting for Frontend to start..."
for i in {1..30}; do
    if curl -s http://localhost:3002/ > /dev/null 2>&1; then
        echo "✅ Frontend is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Frontend failed to start. Check logs at /tmp/ethhook-frontend.log"
        kill $ADMIN_PID 2>/dev/null || true
        exit 1
    fi
    sleep 1
done

echo ""
echo "🎉 Development environment is ready!"
echo ""
echo "📍 Services:"
echo "   Admin API:  http://localhost:8080/api/v1/"
echo "   Frontend:   http://localhost:3002/"
echo "   PostgreSQL: localhost:5432"
echo "   Redis:      localhost:6379"
echo ""
echo "📝 Logs:"
echo "   Admin API:  tail -f /tmp/ethhook-admin-api.log"
echo "   Frontend:   tail -f /tmp/ethhook-frontend.log"
echo ""
echo "🛑 To stop services:"
echo "   kill $ADMIN_PID $FRONTEND_PID"
echo "   Or run: pkill -f 'ethhook-admin-api|trunk serve'"
echo ""
echo "💡 Opening browser..."
sleep 2
open http://localhost:3002/

# Keep script running
echo "Press Ctrl+C to stop all services..."
trap "echo ''; echo '🛑 Stopping services...'; kill $ADMIN_PID $FRONTEND_PID 2>/dev/null || true; echo '✅ Done!'; exit 0" INT
wait
