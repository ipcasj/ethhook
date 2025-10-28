#!/bin/bash
# Run complete load test suite for EthHook
# Tests system performance under high traffic conditions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}================================================================${NC}"
echo -e "${CYAN}🚀 EthHook Load Testing Suite${NC}"
echo -e "${CYAN}================================================================${NC}"
echo ""

# Configuration
EVENTS=${1:-10000}
RATE=${2:-1000}
CONCURRENCY=${3:-10}
REDIS_URL=${REDIS_URL:-redis://localhost:6379}
METRICS_URL=${METRICS_URL:-http://localhost:8000/metrics}

echo -e "${BLUE}📋 Configuration:${NC}"
echo "  Events: $EVENTS"
echo "  Rate: $RATE events/sec"
echo "  Concurrency: $CONCURRENCY publishers"
echo ""

# Step 1: Check prerequisites
echo -e "${YELLOW}Step 1: Checking prerequisites...${NC}"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker Desktop.${NC}"
    exit 1
fi
echo "  ✅ Docker is running"

# Check if PostgreSQL container is running
if ! docker ps | grep -q ethhook-postgres; then
    echo -e "${RED}❌ PostgreSQL container is not running.${NC}"
    echo "  Run: docker compose up -d postgres"
    exit 1
fi
echo "  ✅ PostgreSQL is running"

# Check if Redis is accessible
if ! redis-cli -u "$REDIS_URL" ping > /dev/null 2>&1; then
    echo -e "${RED}❌ Redis is not accessible at $REDIS_URL${NC}"
    echo "  Run: docker compose up -d redis"
    exit 1
fi
echo "  ✅ Redis is accessible"

echo ""

# Step 2: Set up test endpoints
echo -e "${YELLOW}Step 2: Setting up test endpoints in database...${NC}"
./scripts/setup_high_traffic_endpoints.sh
echo ""

# Step 3: Start services
echo -e "${YELLOW}Step 3: Ensuring all services are running...${NC}"

# Check if services are already running
MESSAGE_PROCESSOR_RUNNING=false
WEBHOOK_DELIVERY_RUNNING=false

if lsof -Pi :8081 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "  ℹ️  Message Processor already running on port 8081"
    MESSAGE_PROCESSOR_RUNNING=true
else
    echo "  🚀 Starting Message Processor..."
    RUST_LOG=info cargo run --bin message-processor > /tmp/message-processor.log 2>&1 &
    MESSAGE_PROCESSOR_PID=$!
    sleep 3
    if ps -p $MESSAGE_PROCESSOR_PID > /dev/null; then
        echo "  ✅ Message Processor started (PID: $MESSAGE_PROCESSOR_PID)"
    else
        echo -e "${RED}  ❌ Failed to start Message Processor${NC}"
        exit 1
    fi
fi

if lsof -Pi :8082 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "  ℹ️  Webhook Delivery already running on port 8082"
    WEBHOOK_DELIVERY_RUNNING=true
else
    echo "  🚀 Starting Webhook Delivery..."
    RUST_LOG=info cargo run --bin webhook-delivery > /tmp/webhook-delivery.log 2>&1 &
    WEBHOOK_DELIVERY_PID=$!
    sleep 5
    if ps -p $WEBHOOK_DELIVERY_PID > /dev/null; then
        echo "  ✅ Webhook Delivery started (PID: $WEBHOOK_DELIVERY_PID)"
    else
        echo -e "${RED}  ❌ Failed to start Webhook Delivery${NC}"
        exit 1
    fi
fi

echo ""

# Step 4: Start webhook receiver
echo -e "${YELLOW}Step 4: Starting webhook receiver...${NC}"

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 is not installed.${NC}"
    exit 1
fi

# Kill existing receiver if running
if lsof -Pi :8000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "  🛑 Stopping existing webhook receiver..."
    pkill -f "load_test_receiver.py" || true
    sleep 1
fi

echo "  🚀 Starting load test webhook receiver..."
cd demo-webhook-receiver
python3 load_test_receiver.py > /tmp/receiver.log 2>&1 &
RECEIVER_PID=$!
cd ..

sleep 2

if ps -p $RECEIVER_PID > /dev/null; then
    echo "  ✅ Webhook receiver started (PID: $RECEIVER_PID)"
else
    echo -e "${RED}  ❌ Failed to start webhook receiver${NC}"
    exit 1
fi

echo ""

# Step 5: Wait for services to be ready
echo -e "${YELLOW}Step 5: Waiting for services to be ready...${NC}"
sleep 3

# Check Redis readiness keys
for i in {1..10}; do
    if redis-cli -u "$REDIS_URL" GET message_processor:ready | grep -q "true"; then
        echo "  ✅ Message Processor is ready"
        break
    fi
    if [ $i -eq 10 ]; then
        echo -e "${RED}  ⚠️  Message Processor readiness not confirmed (may still work)${NC}"
    fi
    sleep 1
done

for i in {1..10}; do
    if redis-cli -u "$REDIS_URL" GET webhook_delivery:ready | grep -q "true"; then
        echo "  ✅ Webhook Delivery is ready"
        break
    fi
    if [ $i -eq 10 ]; then
        echo -e "${RED}  ⚠️  Webhook Delivery readiness not confirmed (may still work)${NC}"
    fi
    sleep 1
done

# Check receiver health
if curl -s http://localhost:8000/health | grep -q "healthy"; then
    echo "  ✅ Webhook receiver is ready"
else
    echo -e "${RED}  ❌ Webhook receiver is not responding${NC}"
    exit 1
fi

echo ""

# Step 6: Build load tester
echo -e "${YELLOW}Step 6: Building load tester...${NC}"
cd tools/load-tester
cargo build --release --quiet
cd ../..
echo "  ✅ Load tester built"
echo ""

# Step 7: Run load test
echo -e "${YELLOW}Step 7: Running load test...${NC}"
echo ""

./tools/load-tester/target/release/load-tester \
    --events "$EVENTS" \
    --rate "$RATE" \
    --concurrency "$CONCURRENCY" \
    --redis-url "$REDIS_URL" \
    --metrics-url "$METRICS_URL"

echo ""

# Step 8: Cleanup
echo -e "${YELLOW}Step 8: Cleanup...${NC}"

# Only stop services we started
if [ "$MESSAGE_PROCESSOR_RUNNING" = false ] && [ ! -z "$MESSAGE_PROCESSOR_PID" ]; then
    echo "  🛑 Stopping Message Processor..."
    kill $MESSAGE_PROCESSOR_PID 2>/dev/null || true
fi

if [ "$WEBHOOK_DELIVERY_RUNNING" = false ] && [ ! -z "$WEBHOOK_DELIVERY_PID" ]; then
    echo "  🛑 Stopping Webhook Delivery..."
    kill $WEBHOOK_DELIVERY_PID 2>/dev/null || true
fi

echo "  🛑 Stopping webhook receiver..."
kill $RECEIVER_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}================================================================${NC}"
echo -e "${GREEN}✅ Load test complete!${NC}"
echo -e "${GREEN}================================================================${NC}"
echo ""
echo "📊 Logs available at:"
echo "  - Message Processor: /tmp/message-processor.log"
echo "  - Webhook Delivery: /tmp/webhook-delivery.log"
echo "  - Receiver: /tmp/receiver.log"
echo ""
