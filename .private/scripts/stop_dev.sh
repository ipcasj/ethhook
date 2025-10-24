#!/bin/bash

# EthHook - Stop Development Services

echo "🛑 Stopping EthHook development services..."

# Kill Admin API
pkill -f ethhook-admin-api && echo "✅ Admin API stopped" || echo "ℹ️  Admin API not running"

# Kill Trunk/Frontend
pkill -f "trunk serve" && echo "✅ Frontend stopped" || echo "ℹ️  Frontend not running"

echo ""
echo "✅ All services stopped!"
echo "💡 Docker services (PostgreSQL, Redis) are still running."
echo "   To stop them: docker-compose down"
