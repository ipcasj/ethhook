#!/bin/bash
#
# Comprehensive Test Runner for EthHook
#
# Runs all test types in order: unit → integration → e2e
# Use flags to run specific test suites

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_E2E=true
RUN_CLIPPY=true
RUN_FMT=true

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --unit-only)
            RUN_INTEGRATION=false
            RUN_E2E=false
            shift
            ;;
        --integration-only)
            RUN_UNIT=false
            RUN_E2E=false
            shift
            ;;
        --e2e-only)
            RUN_UNIT=false
            RUN_INTEGRATION=false
            shift
            ;;
        --no-clippy)
            RUN_CLIPPY=false
            shift
            ;;
        --no-fmt)
            RUN_FMT=false
            shift
            ;;
        --fast)
            RUN_E2E=false
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --unit-only          Run only unit tests"
            echo "  --integration-only   Run only integration tests"
            echo "  --e2e-only          Run only E2E tests"
            echo "  --fast              Skip slow E2E tests (unit + integration only)"
            echo "  --no-clippy         Skip clippy checks"
            echo "  --no-fmt            Skip format checks"
            echo "  --help              Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                  # Run all tests"
            echo "  $0 --fast           # Skip E2E tests for quick feedback"
            echo "  $0 --e2e-only       # Only run E2E tests"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run '$0 --help' for usage"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   EthHook Comprehensive Test Suite     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

FAILED_TESTS=""
START_TIME=$(date +%s)

# Function to print section header
print_section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Function to mark test as passed
mark_passed() {
    echo -e "${GREEN}✅ $1 PASSED${NC}"
}

# Function to mark test as failed
mark_failed() {
    echo -e "${RED}❌ $1 FAILED${NC}"
    FAILED_TESTS="$FAILED_TESTS\n  - $1"
}

# ============================================================================
# 1. FORMAT CHECK
# ============================================================================

if [ "$RUN_FMT" = true ]; then
    print_section "📝 Checking Code Format (cargo fmt)"
    
    if cargo fmt -- --check; then
        mark_passed "Format check"
    else
        mark_failed "Format check"
        echo ""
        echo -e "${YELLOW}Hint: Run 'cargo fmt' to fix formatting${NC}"
    fi
fi

# ============================================================================
# 2. CLIPPY LINTS
# ============================================================================

if [ "$RUN_CLIPPY" = true ]; then
    print_section "🔍 Running Clippy Lints (cargo clippy)"
    
    if cargo clippy --all-targets --all-features -- -D warnings; then
        mark_passed "Clippy"
    else
        mark_failed "Clippy"
    fi
fi

# ============================================================================
# 3. UNIT TESTS
# ============================================================================

if [ "$RUN_UNIT" = true ]; then
    print_section "🧪 Running Unit Tests (cargo test --lib)"
    
    echo "Testing workspace crates..."
    if cargo test --workspace --lib; then
        mark_passed "Unit tests"
    else
        mark_failed "Unit tests"
    fi
fi

# ============================================================================
# 4. INTEGRATION TESTS
# ============================================================================

if [ "$RUN_INTEGRATION" = true ]; then
    print_section "🔧 Running Integration Tests"
    
    # Check if Docker Compose is available
    if command -v docker &> /dev/null && docker compose version &> /dev/null; then
        DOCKER_COMPOSE="docker compose"
    elif command -v docker-compose &> /dev/null; then
        DOCKER_COMPOSE="docker-compose"
    else
        echo -e "${YELLOW}⚠️  Docker Compose not found, skipping integration tests${NC}"
        RUN_INTEGRATION=false
    fi
    
    if [ "$RUN_INTEGRATION" = true ]; then
        # Start infrastructure
        echo "📦 Starting infrastructure (PostgreSQL + Redis)..."
        $DOCKER_COMPOSE up -d postgres redis
        
        # Wait for PostgreSQL
        echo "⏳ Waiting for PostgreSQL..."
        timeout=30
        while [ $timeout -gt 0 ]; do
            if docker exec ethhook-postgres pg_isready -U ethhook > /dev/null 2>&1; then
                echo "✓ PostgreSQL ready"
                break
            fi
            sleep 1
            timeout=$((timeout - 1))
        done
        
        # Wait for Redis
        echo "⏳ Waiting for Redis..."
        timeout=30
        while [ $timeout -gt 0 ]; do
            if docker exec ethhook-redis redis-cli ping > /dev/null 2>&1; then
                echo "✓ Redis ready"
                break
            fi
            sleep 1
            timeout=$((timeout - 1))
        done
        
        # Set environment variables
        export DATABASE_URL="postgres://ethhook:password@localhost:5432/ethhook"
        export REDIS_URL="redis://localhost:6379"
        
        # Run database migrations
        echo "🔄 Running database migrations..."
        MIGRATION_SUCCESS=false
        
        # Try sqlx-cli first
        if command -v sqlx &> /dev/null; then
            if sqlx migrate run --source migrations 2>/dev/null; then
                MIGRATION_SUCCESS=true
                echo "✓ Migrations applied via sqlx-cli"
            fi
        fi
        
        # Fallback: run migrations directly via docker
        if [ "$MIGRATION_SUCCESS" = false ]; then
            echo "  Using docker exec fallback..."
            for migration in migrations/*.sql; do
                if [ -f "$migration" ]; then
                    echo "  Applying $(basename "$migration")..."
                    if docker exec -i ethhook-postgres psql -U ethhook -d ethhook < "$migration" > /dev/null 2>&1; then
                        MIGRATION_SUCCESS=true
                    else
                        echo -e "${RED}  Failed to apply $(basename "$migration")${NC}"
                    fi
                fi
            done
            
            if [ "$MIGRATION_SUCCESS" = true ]; then
                echo "✓ Migrations applied via docker"
            else
                echo -e "${RED}❌ Migration failed!${NC}"
                mark_failed "Integration tests (migration failed)"
                RUN_INTEGRATION=false
            fi
        fi
        
        echo ""
        echo "Running integration tests..."
        if cargo test --test integration_tests -- --test-threads=1 --ignored --nocapture; then
            mark_passed "Integration tests"
        else
            mark_failed "Integration tests"
        fi
    fi
fi

# ============================================================================
# 5. END-TO-END TESTS
# ============================================================================

if [ "$RUN_E2E" = true ]; then
    print_section "🚀 Running Real E2E Tests (Full Service Pipeline)"
    
    # Ensure infrastructure is running
    if [ "$RUN_INTEGRATION" = false ]; then
        if command -v docker &> /dev/null && docker compose version &> /dev/null; then
            DOCKER_COMPOSE="docker compose"
        elif command -v docker-compose &> /dev/null; then
            DOCKER_COMPOSE="docker-compose"
        else
            echo -e "${YELLOW}⚠️  Docker Compose not found, skipping E2E tests${NC}"
            RUN_E2E=false
        fi
        
        if [ "$RUN_E2E" = true ]; then
            echo "📦 Starting infrastructure..."
            $DOCKER_COMPOSE up -d postgres redis
            
            # Wait for services
            echo "⏳ Waiting for services..."
            sleep 3
            
            # Ensure migrations are applied
            export DATABASE_URL="postgres://ethhook:password@localhost:5432/ethhook"
            export REDIS_URL="redis://localhost:6379"
            
            echo "🔄 Ensuring database migrations..."
            for migration in migrations/*.sql; do
                if [ -f "$migration" ]; then
                    docker exec -i ethhook-postgres psql -U ethhook -d ethhook < "$migration" > /dev/null 2>&1 || true
                fi
            done
            echo "✓ Database ready"
        fi
    fi
    
    if [ "$RUN_E2E" = true ]; then
        # Build all services first
        echo "🔨 Building all services..."
        if ! cargo build --release; then
            echo -e "${RED}❌ Failed to build services${NC}"
            mark_failed "E2E tests (build failed)"
            RUN_E2E=false
        fi
    fi
    
    if [ "$RUN_E2E" = true ]; then
        echo ""
        echo "Running E2E tests (this may take a while)..."
        if cargo test --test e2e_tests -- --test-threads=1 --ignored --nocapture; then
            mark_passed "E2E tests"
        else
            mark_failed "E2E tests"
        fi
    fi
fi

# ============================================================================
# SUMMARY
# ============================================================================

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Total time: ${DURATION}s"
echo ""

if [ -z "$FAILED_TESTS" ]; then
    echo -e "${GREEN}✅ ALL TESTS PASSED!${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED:${NC}"
    echo -e "${FAILED_TESTS}"
    echo ""
    exit 1
fi
