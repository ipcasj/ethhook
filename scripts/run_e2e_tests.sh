#!/bin/bash
#
# Run End-to-End Integration Tests
#
# This script ensures infrastructure is running and executes E2E tests

set -e

echo "🚀 EthHook E2E Test Runner"
echo "=========================="
echo ""

# Check if docker compose is available (try both old and new syntax)
if command -v docker &> /dev/null && docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
elif command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
else
    echo "❌ Error: Docker Compose not found"
    echo "   Please install Docker and Docker Compose first"
    exit 1
fi

# Start infrastructure if not running
echo "📦 Starting infrastructure (PostgreSQL + Redis)..."
$DOCKER_COMPOSE up -d postgres redis

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
timeout=30
while [ $timeout -gt 0 ]; do
    if docker exec ethhook-postgres pg_isready -U ethhook > /dev/null 2>&1; then
        echo "✓ PostgreSQL is ready"
        break
    fi
    sleep 1
    timeout=$((timeout - 1))
done

if [ $timeout -eq 0 ]; then
    echo "❌ Timeout waiting for PostgreSQL"
    exit 1
fi

# Wait for Redis to be ready
echo "⏳ Waiting for Redis to be ready..."
timeout=30
while [ $timeout -gt 0 ]; do
    if docker exec ethhook-redis redis-cli ping > /dev/null 2>&1; then
        echo "✓ Redis is ready"
        break
    fi
    sleep 1
    timeout=$((timeout - 1))
done

if [ $timeout -eq 0 ]; then
    echo "❌ Timeout waiting for Redis"
    exit 1
fi

# Note: Migrations are assumed to be already applied during initial setup
# If you need to run migrations, execute: sqlx migrate run
echo "✓ Assuming migrations are applied (schema ready)"

# Set environment variables for tests
export DATABASE_URL="postgres://ethhook:password@localhost:5432/ethhook"
export REDIS_URL="redis://localhost:6379"

echo ""
echo "🧪 Running E2E Tests..."
echo "----------------------"

# Run the tests sequentially with --test-threads=1 to avoid test interference
cargo test --package ethhook-e2e-tests -- --test-threads=1 --ignored --nocapture

echo ""
echo "✅ E2E Tests Complete!"
