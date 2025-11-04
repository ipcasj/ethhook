#!/bin/bash

# E2E Testing All-in-One Runner
# This script handles all the prerequisites and runs E2E tests

set -e

echo "🚀 ETHHook E2E Testing - All-in-One Runner"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
UI_DIR="$PROJECT_ROOT/ui"

echo -e "${BLUE}Project root: $PROJECT_ROOT${NC}"
echo -e "${BLUE}UI directory: $UI_DIR${NC}"
echo ""

# Function to check if a port is in use
check_port() {
    lsof -ti:$1 > /dev/null 2>&1
    return $?
}

# Function to wait for service
wait_for_service() {
    local url=$1
    local name=$2
    local max_attempts=30
    local attempt=0
    
    echo -e "${YELLOW}⏳ Waiting for $name to be ready...${NC}"
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -f -s "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $name is ready!${NC}"
            return 0
        fi
        
        attempt=$((attempt + 1))
        echo -n "."
        sleep 1
    done
    
    echo ""
    echo -e "${RED}❌ $name failed to start${NC}"
    return 1
}

# Step 1: Check Node version
echo -e "${BLUE}📦 Step 1: Checking Node.js version...${NC}"
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is not installed${NC}"
    exit 1
fi

# Load NVM and use Node 20
if [ -f "$HOME/.nvm/nvm.sh" ]; then
    source "$HOME/.nvm/nvm.sh"
    nvm use 20 > /dev/null 2>&1 || nvm install 20
fi

NODE_VERSION=$(node -v)
echo -e "${GREEN}✅ Node.js version: $NODE_VERSION${NC}"
echo ""

# Step 2: Check backend
echo -e "${BLUE}🔧 Step 2: Checking backend (port 8080)...${NC}"
if check_port 8080; then
    echo -e "${GREEN}✅ Backend is already running on port 8080${NC}"
else
    echo -e "${YELLOW}⚠️  Backend is not running. Starting it now...${NC}"
    
    # Check if .env exists
    if [ ! -f "$PROJECT_ROOT/.env" ]; then
        echo -e "${RED}❌ .env file not found at $PROJECT_ROOT/.env${NC}"
        exit 1
    fi
    
    # Start backend in background
    cd "$PROJECT_ROOT"
    (
        set -a
        source .env
        set +a
        export ADMIN_API_PORT=8080
        cargo run --bin ethhook-admin-api 2>&1 | while IFS= read -r line; do
            echo "[BACKEND] $line"
        done
    ) &
    BACKEND_PID=$!
    echo "$BACKEND_PID" > /tmp/ethhook-backend.pid
    
    # Wait for backend to be ready
    if ! wait_for_service "http://localhost:8080/health" "Backend"; then
        kill $BACKEND_PID 2>/dev/null || true
        rm -f /tmp/ethhook-backend.pid
        exit 1
    fi
fi
echo ""

# Step 3: Check frontend
echo -e "${BLUE}🎨 Step 3: Checking frontend (port 3000)...${NC}"
if check_port 3000; then
    echo -e "${GREEN}✅ Frontend is already running on port 3000${NC}"
else
    echo -e "${YELLOW}⚠️  Frontend is not running. Starting it now...${NC}"
    
    cd "$UI_DIR"
    npm run dev > /tmp/ethhook-frontend.log 2>&1 &
    FRONTEND_PID=$!
    echo "$FRONTEND_PID" > /tmp/ethhook-frontend.pid
    
    # Wait for frontend to be ready
    if ! wait_for_service "http://localhost:3000" "Frontend"; then
        kill $FRONTEND_PID 2>/dev/null || true
        rm -f /tmp/ethhook-frontend.pid
        exit 1
    fi
fi
echo ""

# Step 4: Run tests
echo -e "${BLUE}🧪 Step 4: Running E2E tests...${NC}"
echo ""

cd "$UI_DIR"

# Determine which tests to run
TEST_MODE="${1:-ui}"

case "$TEST_MODE" in
    "ui")
        echo -e "${YELLOW}Running tests in interactive UI mode...${NC}"
        npm run test:e2e:ui
        ;;
    "headless")
        echo -e "${YELLOW}Running tests in headless mode...${NC}"
        npm run test:e2e
        ;;
    "smoke")
        echo -e "${YELLOW}Running smoke tests...${NC}"
        npm run test:e2e:smoke
        ;;
    *)
        echo -e "${RED}Invalid test mode: $TEST_MODE${NC}"
        echo "Usage: $0 [ui|headless|smoke]"
        exit 1
        ;;
esac

TEST_EXIT_CODE=$?

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Tests completed successfully!${NC}"
else
    echo -e "${RED}❌ Tests failed${NC}"
fi

echo ""
echo -e "${BLUE}📊 View test report:${NC}"
echo "  npm run test:e2e:report"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    
    # Only kill services we started
    if [ -f /tmp/ethhook-backend.pid ]; then
        BACKEND_PID=$(cat /tmp/ethhook-backend.pid)
        echo -e "${YELLOW}Stopping backend (PID: $BACKEND_PID)...${NC}"
        kill $BACKEND_PID 2>/dev/null || true
        rm -f /tmp/ethhook-backend.pid
    fi
    
    if [ -f /tmp/ethhook-frontend.pid ]; then
        FRONTEND_PID=$(cat /tmp/ethhook-frontend.pid)
        echo -e "${YELLOW}Stopping frontend (PID: $FRONTEND_PID)...${NC}"
        kill $FRONTEND_PID 2>/dev/null || true
        rm -f /tmp/ethhook-frontend.pid
    fi
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Ask if user wants to keep services running
echo -e "${YELLOW}Keep services running? (y/N)${NC}"
read -t 10 -n 1 KEEP_RUNNING || KEEP_RUNNING="n"
echo ""

if [[ $KEEP_RUNNING != "y" ]] && [[ $KEEP_RUNNING != "Y" ]]; then
    cleanup
else
    echo -e "${GREEN}✅ Services are still running:${NC}"
    echo "  - Backend: http://localhost:8080"
    echo "  - Frontend: http://localhost:3000"
    echo ""
    echo -e "${YELLOW}To stop services later:${NC}"
    if [ -f /tmp/ethhook-backend.pid ]; then
        echo "  kill $(cat /tmp/ethhook-backend.pid)"
    fi
    if [ -f /tmp/ethhook-frontend.pid ]; then
        echo "  kill $(cat /tmp/ethhook-frontend.pid)"
    fi
fi

exit $TEST_EXIT_CODE
