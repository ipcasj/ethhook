#!/bin/bash
# Stop all ETHHook services

echo "🛑 Stopping all ETHHook services..."

pkill -9 ethhook-admin-api 2>/dev/null || true
pkill -9 event-ingestor 2>/dev/null || true
pkill -9 ethhook-message-processor 2>/dev/null || true
pkill -9 ethhook-webhook-delivery 2>/dev/null || true
pkill -9 trunk 2>/dev/null || true

sleep 2

echo "✅ All services stopped"
