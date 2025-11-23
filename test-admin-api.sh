#!/bin/bash
# Test script for running admin-API locally

# Set required environment variables
export JWT_SECRET="test-jwt-secret-for-local-development"
export DATABASE_URL="sqlite:config.db"
export ADMIN_API_HOST="127.0.0.1"
export ADMIN_API_PORT="3000"

# ClickHouse (will use defaults if not running)
export CLICKHOUSE_URL="http://localhost:8123"
export CLICKHOUSE_USER="default"
export CLICKHOUSE_PASSWORD=""
export CLICKHOUSE_DATABASE="ethhook"

echo "Starting admin-API server..."
echo "- Host: $ADMIN_API_HOST"
echo "- Port: $ADMIN_API_PORT"
echo "- Database: $DATABASE_URL"
echo "- ClickHouse: $CLICKHOUSE_URL"
echo ""
echo "Note: ClickHouse queries will fail if ClickHouse is not running"
echo "      But config management (users/apps/endpoints) will work fine"
echo ""

cargo run --bin ethhook-admin-api
