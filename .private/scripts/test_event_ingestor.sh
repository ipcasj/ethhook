#!/bin/bash
#
# Event Ingestor Test Suite
# 
# This script runs all tests for the Event Ingestor service.
# Usage: ./scripts/test_event_ingestor.sh [OPTIONS]
#
# Options:
#   --unit         Run unit tests only
#   --integration  Run integration tests only
#   --coverage     Generate code coverage report
#   --watch        Watch mode (re-run on file changes)
#   --verbose      Verbose output
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
RUN_UNIT=true
RUN_INTEGRATION=false
GENERATE_COVERAGE=false
WATCH_MODE=false
VERBOSE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --unit)
            RUN_UNIT=true
            RUN_INTEGRATION=false
            shift
            ;;
        --integration)
            RUN_UNIT=false
            RUN_INTEGRATION=true
            shift
            ;;
        --coverage)
            GENERATE_COVERAGE=true
            shift
            ;;
        --watch)
            WATCH_MODE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Event Ingestor Test Suite                   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════╝${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Must run from project root${NC}"
    exit 1
fi

# Check if event-ingestor crate exists
if [ ! -d "crates/event-ingestor" ]; then
    echo -e "${RED}❌ Error: event-ingestor crate not found${NC}"
    exit 1
fi

# Function to run unit tests
run_unit_tests() {
    echo -e "${YELLOW}📦 Running unit tests...${NC}"
    echo ""
    
    if [ "$VERBOSE" = true ]; then
        cargo test -p ethhook-event-ingestor --lib -- --nocapture
    else
        cargo test -p ethhook-event-ingestor --lib
    fi
    
    if [ $? -eq 0 ]; then
        echo ""
        echo -e "${GREEN}✅ Unit tests passed!${NC}"
    else
        echo ""
        echo -e "${RED}❌ Unit tests failed!${NC}"
        exit 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${YELLOW}🔗 Running integration tests...${NC}"
    echo ""
    
    # Check if Docker containers are running
    if ! docker ps | grep -q "ethhook-postgres\|ethhook-redis"; then
        echo -e "${YELLOW}⚠️  Warning: Docker containers not running${NC}"
        echo -e "${YELLOW}   Starting containers with docker-compose...${NC}"
        docker-compose up -d postgres redis
        
        # Wait for services to be ready
        echo -e "${YELLOW}   Waiting for services to be ready...${NC}"
        sleep 5
    fi
    
    # Set test environment variables
    export RUST_LOG=debug
    export REDIS_HOST=localhost
    export REDIS_PORT=6379
    export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/ethhook_test
    
    if [ "$VERBOSE" = true ]; then
        cargo test -p ethhook-event-ingestor --test '*' -- --nocapture
    else
        cargo test -p ethhook-event-ingestor --test '*'
    fi
    
    if [ $? -eq 0 ]; then
        echo ""
        echo -e "${GREEN}✅ Integration tests passed!${NC}"
    else
        echo ""
        echo -e "${RED}❌ Integration tests failed!${NC}"
        exit 1
    fi
}

# Function to generate coverage report
generate_coverage() {
    echo -e "${YELLOW}📊 Generating code coverage report...${NC}"
    echo ""
    
    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-tarpaulin not found, installing...${NC}"
        cargo install cargo-tarpaulin
    fi
    
    cargo tarpaulin \
        --out Html \
        --output-dir coverage \
        --packages ethhook-event-ingestor \
        --exclude-files 'target/*' \
        --verbose
    
    echo ""
    echo -e "${GREEN}✅ Coverage report generated: coverage/index.html${NC}"
    
    # Open in browser (macOS)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo -e "${BLUE}🌐 Opening coverage report in browser...${NC}"
        open coverage/index.html
    fi
}

# Function to run in watch mode
run_watch_mode() {
    echo -e "${YELLOW}👀 Running in watch mode (press Ctrl+C to exit)...${NC}"
    echo ""
    
    # Check if cargo-watch is installed
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-watch not found, installing...${NC}"
        cargo install cargo-watch
    fi
    
    cargo watch -x "test -p ethhook-event-ingestor"
}

# Main execution
echo -e "${BLUE}🔧 Configuration:${NC}"
echo "   Unit tests: $RUN_UNIT"
echo "   Integration tests: $RUN_INTEGRATION"
echo "   Coverage: $GENERATE_COVERAGE"
echo "   Watch mode: $WATCH_MODE"
echo "   Verbose: $VERBOSE"
echo ""

# Check if watch mode is enabled
if [ "$WATCH_MODE" = true ]; then
    run_watch_mode
    exit 0
fi

# Run tests based on configuration
if [ "$RUN_UNIT" = true ]; then
    run_unit_tests
fi

if [ "$RUN_INTEGRATION" = true ]; then
    run_integration_tests
fi

if [ "$GENERATE_COVERAGE" = true ]; then
    generate_coverage
fi

# Final summary
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   All tests completed successfully! 🎉        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════╝${NC}"
