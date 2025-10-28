#!/usr/bin/env bash
set -euo pipefail

# E2E Test Pre-Check Script
# Verifies all required services are running before executing E2E tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 E2E Test Pre-Check"
echo "====================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FAILED=0

# Function to check if a service is running
check_service() {
    local service_name="$1"
    local check_command="$2"
    local error_message="$3"
    
    echo -n "Checking $service_name... "
    if eval "$check_command" &> /dev/null; then
        echo -e "${GREEN}✓ Running${NC}"
        return 0
    else
        echo -e "${RED}✗ Not running${NC}"
        echo "  Error: $error_message"
        FAILED=1
        return 1
    fi
}

# Check PostgreSQL
check_service "PostgreSQL" \
    "psql postgres://ethhook:password@localhost:5432/ethhook -c 'SELECT 1' -q -t" \
    "PostgreSQL is not running or ethhook database not accessible. Run: docker-compose up -d postgres"

# Check Redis
check_service "Redis" \
    "redis-cli -h localhost -p 6379 PING | grep -q PONG" \
    "Redis is not running. Run: docker-compose up -d redis"

# Check if services are built
echo ""
echo "Checking service binaries..."

check_binary() {
    local binary_name="$1"
    local binary_path="$PROJECT_ROOT/target/debug/$binary_name"
    
    echo -n "  $binary_name... "
    if [[ -f "$binary_path" ]]; then
        echo -e "${GREEN}✓ Built${NC}"
        return 0
    else
        echo -e "${RED}✗ Not found${NC}"
        echo "    Error: Binary not found at $binary_path"
        echo "    Run: cargo build --bins"
        FAILED=1
        return 1
    fi
}

check_binary "event-ingestor"
check_binary "ethhook-message-processor"
check_binary "ethhook-webhook-delivery"

# Check if database has required tables
echo ""
echo -n "Checking database schema... "
TABLES=$(psql postgres://ethhook:password@localhost:5432/ethhook -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name IN ('users', 'applications', 'endpoints')" -q -t 2>/dev/null || echo "0")
TABLES=$(echo "$TABLES" | tr -d ' ')

if [[ "$TABLES" == "3" ]]; then
    echo -e "${GREEN}✓ Schema ready${NC}"
else
    echo -e "${RED}✗ Missing tables${NC}"
    echo "  Error: Database schema not initialized. Run migrations:"
    echo "    sqlx migrate run --database-url postgres://ethhook:password@localhost:5432/ethhook"
    FAILED=1
fi

# Check if ports are available
echo ""
echo "Checking port availability..."

check_port() {
    local port="$1"
    local service="$2"
    
    echo -n "  Port $port ($service)... "
    if lsof -iTCP:$port -sTCP:LISTEN -n -P &> /dev/null; then
        echo -e "${YELLOW}⚠ Already in use${NC}"
        echo "    Warning: Port $port is in use. E2E tests may fail if this is from a previous test run."
        echo "    Run: lsof -ti:$port | xargs kill -9"
    else
        echo -e "${GREEN}✓ Available${NC}"
    fi
}

check_port 8080 "webhook-delivery health"
check_port 8081 "message-processor health"
check_port 8082 "event-ingestor health"
check_port 9876 "mock webhook receiver"

# Summary
echo ""
echo "========================================"
if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}✅ All pre-checks passed!${NC}"
    echo ""
    echo "Ready to run E2E tests:"
    echo "  cargo test --test e2e_tests -- --ignored"
    exit 0
else
    echo -e "${RED}❌ Pre-checks failed!${NC}"
    echo ""
    echo "Fix the issues above before running E2E tests."
    echo ""
    echo "Quick start guide:"
    echo "  1. Start infrastructure: docker-compose up -d"
    echo "  2. Run migrations: sqlx migrate run"
    echo "  3. Build services: cargo build --bins"
    echo "  4. Run this check again: $0"
    exit 1
fi
