#!/usr/bin/env bash

# Production System Functionality Test
# Tests all critical endpoints and services on production server

set -e

PROD_HOST="104.248.15.178"
API_URL="http://${PROD_HOST}:3000"
UI_URL="http://${PROD_HOST}:3002"
WEBHOOK_RECEIVER_URL="http://${PROD_HOST}:8000"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   EthHook Production System Functionality Test      ║${NC}"
echo -e "${BLUE}║   Server: ${PROD_HOST}                   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

# Test function
test_endpoint() {
    local name="$1"
    local url="$2"
    local expected="$3"
    local method="${4:-GET}"
    local data="${5:-}"
    
    echo -n "Testing ${name}... "
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "$url" \
            -H "Content-Type: application/json" \
            -d "$data" 2>/dev/null || echo "000")
    else
        response=$(curl -s -w "\n%{http_code}" "$url" 2>/dev/null || echo "000")
    fi
    
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected" ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $http_code)"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} (Expected: $expected, Got: $http_code)"
        [ -n "$body" ] && echo "  Response: $body"
        ((FAILED++))
        return 1
    fi
}

# Test function with content check
test_endpoint_content() {
    local name="$1"
    local url="$2"
    local expected_http="$3"
    local expected_content="$4"
    local method="${5:-GET}"
    local data="${6:-}"
    
    echo -n "Testing ${name}... "
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "$url" \
            -H "Content-Type: application/json" \
            -d "$data" 2>/dev/null || echo "000")
    else
        response=$(curl -s -w "\n%{http_code}" "$url" 2>/dev/null || echo "000")
    fi
    
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected_http" ] && echo "$body" | grep -q "$expected_content"; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $http_code, content verified)"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        [ "$http_code" != "$expected_http" ] && echo "  Expected HTTP: $expected_http, Got: $http_code"
        ! echo "$body" | grep -q "$expected_content" && echo "  Expected content not found: $expected_content"
        echo "  Response: $body"
        ((FAILED++))
        return 1
    fi
}

echo -e "${YELLOW}━━━ Phase 1: Service Health Checks ━━━${NC}"
echo ""

test_endpoint_content "API Health" "${API_URL}/api/v1/health" "200" "OK" || true
test_endpoint_content "UI Root (redirects to login)" "${UI_URL}" "307" "EthHook" || true

# Note: Webhook receiver is internal only (not accessible from external network)
echo -n "Testing Demo Webhook Receiver (internal)... "
receiver_check=$(ssh -o ConnectTimeout=5 root@${PROD_HOST} "curl -s http://localhost:8000/health" 2>/dev/null || echo "")
if echo "$receiver_check" | grep -q "healthy"; then
    echo -e "${GREEN}✓ PASS${NC} (accessible internally)"
    ((PASSED++))
else
    echo -e "${RED}✗ FAIL${NC} (not responding)"
    ((FAILED++))
fi

echo ""
echo -e "${YELLOW}━━━ Phase 2: Authentication Tests ━━━${NC}"
echo ""

# Test admin login
ADMIN_LOGIN='{"email":"admin@ethhook.io","password":"SecureAdmin123!"}'
admin_response=$(curl -s -w "\n%{http_code}" -X POST "${API_URL}/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d "$ADMIN_LOGIN" 2>/dev/null)
admin_http_code=$(echo "$admin_response" | tail -1)
admin_body=$(echo "$admin_response" | sed '$d')

echo -n "Testing Admin Login... "
if [ "$admin_http_code" = "200" ] && echo "$admin_body" | grep -q "token"; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((PASSED++))
    ADMIN_TOKEN=$(echo "$admin_body" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    echo "  Admin token obtained: ${ADMIN_TOKEN:0:20}..."
else
    echo -e "${RED}✗ FAIL${NC} (HTTP $admin_http_code)"
    echo "  Response: $admin_body"
    ((FAILED++))
    ADMIN_TOKEN=""
fi

# Test demo user login
DEMO_LOGIN='{"email":"demo@ethhook.com","password":"Demo1234!"}'
demo_response=$(curl -s -w "\n%{http_code}" -X POST "${API_URL}/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d "$DEMO_LOGIN" 2>/dev/null)
demo_http_code=$(echo "$demo_response" | tail -1)
demo_body=$(echo "$demo_response" | sed '$d')

echo -n "Testing Demo User Login... "
if [ "$demo_http_code" = "200" ] && echo "$demo_body" | grep -q "token"; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((PASSED++))
    DEMO_TOKEN=$(echo "$demo_body" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    echo "  Demo token obtained: ${DEMO_TOKEN:0:20}..."
else
    echo -e "${RED}✗ FAIL${NC} (HTTP $demo_http_code)"
    echo "  Response: $demo_body"
    ((FAILED++))
    DEMO_TOKEN=""
fi

echo ""
echo -e "${YELLOW}━━━ Phase 3: API Endpoints (Authenticated) ━━━${NC}"
echo ""

if [ -n "$DEMO_TOKEN" ]; then
    # Test applications list
    echo -n "Testing List Applications... "
    apps_response=$(curl -s -w "\n%{http_code}" "${API_URL}/api/v1/applications" \
        -H "Authorization: Bearer $DEMO_TOKEN" 2>/dev/null)
    apps_http_code=$(echo "$apps_response" | tail -1)
    apps_body=$(echo "$apps_response" | sed '$d')
    
    if [ "$apps_http_code" = "200" ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
        app_count=$(echo "$apps_body" | grep -o '"id"' | wc -l)
        echo "  Found $app_count applications"
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $apps_http_code)"
        ((FAILED++))
    fi
    
    # Test endpoints list
    echo -n "Testing List Endpoints... "
    endpoints_response=$(curl -s -w "\n%{http_code}" "${API_URL}/api/v1/endpoints" \
        -H "Authorization: Bearer $DEMO_TOKEN" 2>/dev/null)
    endpoints_http_code=$(echo "$endpoints_response" | tail -1)
    endpoints_body=$(echo "$endpoints_response" | sed '$d')
    
    if [ "$endpoints_http_code" = "200" ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
        endpoint_count=$(echo "$endpoints_body" | grep -o '"id"' | wc -l)
        echo "  Found $endpoint_count endpoints"
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $endpoints_http_code)"
        ((FAILED++))
    fi
    
    # Test events list
    echo -n "Testing List Events... "
    events_response=$(curl -s -w "\n%{http_code}" "${API_URL}/api/v1/events?limit=10" \
        -H "Authorization: Bearer $DEMO_TOKEN" 2>/dev/null)
    events_http_code=$(echo "$events_response" | tail -1)
    events_body=$(echo "$events_response" | sed '$d')
    
    if [ "$events_http_code" = "200" ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
        event_count=$(echo "$events_body" | grep -o '"event_id"' | wc -l)
        echo "  Found $event_count recent events"
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $events_http_code)"
        ((FAILED++))
    fi
    
    # Test dashboard stats
    echo -n "Testing Dashboard Stats... "
    stats_response=$(curl -s -w "\n%{http_code}" "${API_URL}/api/v1/dashboard/stats" \
        -H "Authorization: Bearer $DEMO_TOKEN" 2>/dev/null)
    stats_http_code=$(echo "$stats_response" | tail -1)
    stats_body=$(echo "$stats_response" | sed '$d')
    
    if [ "$stats_http_code" = "200" ] && echo "$stats_body" | grep -q "total_events"; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
        total_events=$(echo "$stats_body" | grep -o '"total_events":[0-9]*' | cut -d':' -f2)
        echo "  Total events in system: $total_events"
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $stats_http_code)"
        ((FAILED++))
    fi
else
    echo -e "${YELLOW}⚠ Skipping authenticated tests (no demo token)${NC}"
    ((FAILED+=4))
fi

echo ""
echo -e "${YELLOW}━━━ Phase 4: Demo Webhook Receiver ━━━${NC}"
echo ""

# Test webhook history
echo -n "Testing Webhook History... "
history_response=$(curl -s -w "\n%{http_code}" "${WEBHOOK_RECEIVER_URL}/history" 2>/dev/null)
history_http_code=$(echo "$history_response" | tail -1)
history_body=$(echo "$history_response" | sed '$d')

if [ "$history_http_code" = "200" ]; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((PASSED++))
    webhook_count=$(echo "$history_body" | grep -o '"webhook_id"' | wc -l)
    echo "  Webhooks received: $webhook_count"
else
    echo -e "${RED}✗ FAIL${NC} (HTTP $history_http_code)"
    ((FAILED++))
fi

echo ""
echo -e "${YELLOW}━━━ Phase 5: Docker Container Status ━━━${NC}"
echo ""

echo "Checking running containers on production server..."
ssh root@${PROD_HOST} << 'EOSSH'
    cd ~/ethhook || cd ~/rust_projects/capstone0 || cd ~/capstone0
    
    echo ""
    docker ps --format "{{.Names}}: {{.Status}}" | grep ethhook || echo "No containers running"
    echo ""
EOSSH

echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Test Summary                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Total Tests: $((PASSED + FAILED))"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! Production system is healthy.${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed. Please review the output above.${NC}"
    echo ""
    echo "Troubleshooting tips:"
    echo "  1. Check container status: ./scripts/check-production.sh"
    echo "  2. View logs: ssh root@${PROD_HOST} 'docker logs ethhook-admin-api --tail 50'"
    echo "  3. Restart services: ssh root@${PROD_HOST} 'cd ~/ethhook && docker compose -f docker-compose.prod.yml restart'"
    exit 1
fi
