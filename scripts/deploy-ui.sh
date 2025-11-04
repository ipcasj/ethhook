#!/usr/bin/env bash

# Quick script to deploy/update the UI on production server

set -e

HOST="104.248.15.178"
USER="root"

echo "🚀 Deploying UI to production server..."
echo ""

ssh ${USER}@${HOST} << 'EOSSH'
    set -e
    
    # Find the project directory
    if [ -d ~/ethhook ]; then
        cd ~/ethhook
    elif [ -d ~/capstone0 ]; then
        cd ~/capstone0
    elif [ -d ~/rust_projects/capstone0 ]; then
        cd ~/rust_projects/capstone0
    else
        echo "❌ Project directory not found!"
        exit 1
    fi
    
    echo "📂 Working directory: $(pwd)"
    echo ""
    
    # Pull latest code
    echo "📥 Pulling latest code..."
    git fetch origin
    git pull origin main
    echo ""
    
    # Start/restart UI container
    echo "🔨 Building and starting UI container..."
    docker-compose -f docker-compose.prod.yml up -d --build ui
    
    echo ""
    echo "⏳ Waiting for UI to start..."
    sleep 10
    
    # Check status
    echo ""
    echo "📊 Container status:"
    docker ps | grep ethhook-ui || echo "⚠️  UI container not running"
    
    echo ""
    echo "📝 Recent logs:"
    docker logs ethhook-ui --tail 20
    
    echo ""
    echo "✅ UI deployment complete!"
    echo ""
    echo "🌐 Access your UI at: http://104.248.15.178:3002"
    echo "🔧 API endpoint: http://104.248.15.178:3000"
EOSSH

echo ""
echo "🎉 Done! Test the UI at: http://104.248.15.178:3002"
