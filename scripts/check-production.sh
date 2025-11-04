#!/usr/bin/env bash

# Production Server Diagnostic Script
# Run this to check what's running and fix common issues

set -e

HOST="104.248.15.178"
USER="root"

echo "🔍 Checking production server status..."
echo ""

# Check if we can connect
echo "1️⃣ Testing SSH connection..."
if ssh -o ConnectTimeout=5 ${USER}@${HOST} "echo 'SSH OK'" 2>/dev/null; then
    echo "✅ SSH connection successful"
else
    echo "❌ Cannot connect via SSH"
    echo "   Please check your SSH key and server access"
    exit 1
fi

echo ""
echo "2️⃣ Checking Docker containers..."
ssh ${USER}@${HOST} << 'EOSSH'
    cd ~/rust_projects/capstone0 || cd ~/ethhook || cd ~/capstone0

    echo "Running containers:"
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || docker compose ps

    echo ""
    echo "All containers (including stopped):"
    docker ps -a --format "table {{.Names}}\t{{.Status}}"
EOSSH

echo ""
echo "3️⃣ Checking firewall (ufw status)..."
ssh ${USER}@${HOST} "sudo ufw status" || echo "⚠️  ufw not configured"

echo ""
echo "4️⃣ Checking if ports are listening..."
ssh ${USER}@${HOST} << 'EOSSH'
    echo "Port 3000 (API):"
    ss -tlnp | grep :3000 || echo "❌ Port 3000 not listening"
    
    echo ""
    echo "Port 3002 (UI):"
    ss -tlnp | grep :3002 || echo "❌ Port 3002 not listening"
    
    echo ""
    echo "Port 5432 (PostgreSQL):"
    ss -tlnp | grep :5432 || echo "❌ Port 5432 not listening"
EOSSH

echo ""
echo "5️⃣ Checking recent container logs..."
ssh ${USER}@${HOST} << 'EOSSH'
    cd ~/rust_projects/capstone0 || cd ~/ethhook || cd ~/capstone0
    
    echo "Admin API logs (last 20 lines):"
    docker logs ethhook-admin-api --tail 20 2>&1 | tail -10 || echo "⚠️  admin-api not running"
    
    echo ""
    echo "UI logs (last 20 lines):"
    docker logs ethhook-ui --tail 20 2>&1 | tail -10 || echo "⚠️  ui not running"
EOSSH

echo ""
echo "📊 Summary and recommendations:"
echo ""
echo "To start services, run:"
echo "  ssh ${USER}@${HOST}"
echo "  cd ~/rust_projects/capstone0"
echo "  docker-compose -f docker-compose.prod.yml up -d"
echo ""
echo "To open firewall ports (if needed):"
echo "  sudo ufw allow 3000/tcp"
echo "  sudo ufw allow 3002/tcp"
echo ""
echo "To view live logs:"
echo "  docker logs -f ethhook-admin-api"
echo "  docker logs -f ethhook-ui"
