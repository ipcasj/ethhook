#!/usr/bin/env bash
#
# CI Pipeline Local Test Script
# Simulates GitHub Actions CI pipeline locally
#
# Usage: ./scripts/test_ci_locally.sh [job_name]
#   If no job_name provided, runs all jobs
#
# Available jobs:
#   lint, test, e2e, build, sqlx, security, all

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track results
FAILED_JOBS=()
PASSED_JOBS=()

# Helper functions
print_header() {
    echo ""
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}================================${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

check_dependencies() {
    print_header "Checking Dependencies"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker not found. Please install Docker to run tests."
        exit 1
    fi
    print_success "Docker found"
    
    # Check Docker Compose
    if ! docker compose version &> /dev/null; then
        print_error "Docker Compose not found. Please install Docker Compose."
        exit 1
    fi
    print_success "Docker Compose found"
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust."
        exit 1
    fi
    print_success "Rust $(rustc --version)"
    
    # Check sqlx-cli
    if ! command -v sqlx &> /dev/null; then
        print_warning "sqlx-cli not found. Installing..."
        cargo install sqlx-cli --no-default-features --features postgres
    else
        print_success "sqlx-cli found"
    fi
    
    # Check cargo-audit (optional)
    if ! command -v cargo-audit &> /dev/null; then
        print_warning "cargo-audit not found. Skipping security audit."
        print_warning "Install with: cargo install cargo-audit"
    else
        print_success "cargo-audit found"
    fi
}

start_services() {
    print_header "Starting Services (PostgreSQL + Redis)"
    
    # Start Docker services
    docker compose up -d postgres redis
    
    # Wait for services to be healthy
    echo "Waiting for PostgreSQL..."
    until docker compose exec -T postgres pg_isready -U ethhook &> /dev/null; do
        echo -n "."
        sleep 1
    done
    echo ""
    print_success "PostgreSQL ready"
    
    echo "Waiting for Redis..."
    until docker compose exec -T redis redis-cli ping &> /dev/null; do
        echo -n "."
        sleep 1
    done
    echo ""
    print_success "Redis ready"
}

stop_services() {
    print_header "Stopping Services"
    docker compose down -v
    print_success "Services stopped"
}

setup_database() {
    print_header "Setting Up Database"
    
    export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
    
    # Drop and recreate database to ensure clean state
    echo "Resetting database..."
    sqlx database drop -y 2>/dev/null || print_warning "Database doesn't exist yet"
    sqlx database create
    print_success "Database created"
    
    # Run migrations
    echo "Running migrations..."
    sqlx migrate run
    print_success "Database migrations complete"
}

job_lint() {
    print_header "Job 1: Lint & Format Check"
    
    echo "Checking formatting..."
    if cargo fmt --all -- --check; then
        print_success "Formatting check passed"
    else
        print_error "Formatting check failed"
        FAILED_JOBS+=("lint")
        return 1
    fi
    
    echo ""
    echo "Running clippy..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "Clippy passed"
    else
        print_error "Clippy failed"
        FAILED_JOBS+=("lint")
        return 1
    fi
    
    PASSED_JOBS+=("lint")
}

job_test() {
    print_header "Job 2: Unit & Integration Tests"
    
    export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
    export REDIS_URL="redis://localhost:6379"
    export JWT_SECRET="test-secret-for-ci"
    
    echo "Running unit tests..."
    if cargo test --workspace --lib --bins; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed"
        FAILED_JOBS+=("test")
        return 1
    fi
    
    echo ""
    echo "Running Admin API integration tests..."
    if cargo test -p ethhook-admin-api --test integration_test -- --ignored --test-threads=1; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed"
        FAILED_JOBS+=("test")
        return 1
    fi
    
    PASSED_JOBS+=("test")
}

job_e2e() {
    print_header "Job 3: End-to-End Tests"
    
    export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
    export REDIS_HOST="localhost"
    export REDIS_PORT="6379"
    export JWT_SECRET="test-secret-for-e2e-tests"
    
    echo "Running E2E tests..."
    if timeout 600 cargo test --test e2e_tests -- --ignored --test-threads=1; then
        print_success "E2E tests passed"
    else
        print_error "E2E tests failed"
        FAILED_JOBS+=("e2e-tests")
        return 1
    fi
    
    PASSED_JOBS+=("e2e-tests")
}

job_build() {
    print_header "Job 4: Build All Services"
    
    echo "Building workspace in release mode..."
    if cargo build --workspace --release; then
        print_success "Build passed"
        
        echo ""
        echo "Binaries created:"
        ls -lh target/release/event-ingestor 2>/dev/null || true
        ls -lh target/release/ethhook-message-processor 2>/dev/null || true
        ls -lh target/release/ethhook-webhook-delivery 2>/dev/null || true
        ls -lh target/release/ethhook-admin-api 2>/dev/null || true
    else
        print_error "Build failed"
        FAILED_JOBS+=("build")
        return 1
    fi
    
    PASSED_JOBS+=("build")
}

job_sqlx() {
    print_header "Job 5: SQLx Offline Mode Check"
    
    echo "Checking .sqlx directory..."
    if [ ! -d ".sqlx" ]; then
        print_error ".sqlx directory not found!"
        FAILED_JOBS+=("sqlx-check")
        return 1
    fi
    print_success ".sqlx directory exists ($(ls -1 .sqlx/*.json 2>/dev/null | wc -l | tr -d ' ') query cache files)"
    
    echo ""
    echo "Building with offline mode..."
    if SQLX_OFFLINE=true cargo build --workspace; then
        print_success "SQLx offline mode passed"
    else
        print_error "SQLx offline mode failed"
        FAILED_JOBS+=("sqlx-check")
        return 1
    fi
    
    PASSED_JOBS+=("sqlx-check")
}

job_security() {
    print_header "Job 6: Security Audit"
    
    if ! command -v cargo-audit &> /dev/null; then
        print_warning "cargo-audit not installed, skipping security audit"
        print_warning "Install with: cargo install cargo-audit"
        return 0
    fi
    
    echo "Running security audit..."
    if cargo audit --deny unsound --deny yanked \
        --ignore RUSTSEC-2024-0437 \
        --ignore RUSTSEC-2023-0071 \
        --ignore RUSTSEC-2024-0436 \
        --ignore RUSTSEC-2024-0370; then
        print_success "Security audit passed"
    else
        print_error "Security audit failed"
        FAILED_JOBS+=("security-audit")
        return 1
    fi
    
    PASSED_JOBS+=("security-audit")
}

print_summary() {
    print_header "CI Pipeline Summary"
    
    echo "Results:"
    echo ""
    
    if [ ${#PASSED_JOBS[@]} -gt 0 ]; then
        echo -e "${GREEN}Passed Jobs (${#PASSED_JOBS[@]}):${NC}"
        for job in "${PASSED_JOBS[@]}"; do
            echo -e "  ${GREEN}✓${NC} $job"
        done
        echo ""
    fi
    
    if [ ${#FAILED_JOBS[@]} -gt 0 ]; then
        echo -e "${RED}Failed Jobs (${#FAILED_JOBS[@]}):${NC}"
        for job in "${FAILED_JOBS[@]}"; do
            echo -e "  ${RED}✗${NC} $job"
        done
        echo ""
        echo -e "${RED}❌ CI Pipeline Failed${NC}"
        return 1
    else
        echo -e "${GREEN}✅ All CI Checks Passed!${NC}"
        echo ""
        echo "Your changes are ready to push! 🚀"
        return 0
    fi
}

# Main execution
main() {
    local job="${1:-all}"
    
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║   CI Pipeline Local Test Runner        ║"
    echo "║   Simulating GitHub Actions            ║"
    echo "╚════════════════════════════════════════╝"
    echo ""
    
    # Check dependencies first
    check_dependencies
    
    # Start services
    start_services
    trap stop_services EXIT
    
    # Setup database
    setup_database
    
    # Run requested jobs
    case "$job" in
        lint)
            job_lint
            ;;
        test)
            job_test
            ;;
        e2e)
            job_e2e
            ;;
        build)
            job_build
            ;;
        sqlx)
            job_sqlx
            ;;
        security)
            job_security
            ;;
        all)
            # Run all jobs in order
            job_lint || true
            job_test || true
            job_e2e || true
            job_build || true
            job_sqlx || true
            job_security || true
            ;;
        *)
            echo "Unknown job: $job"
            echo "Available jobs: lint, test, e2e, build, sqlx, security, all"
            exit 1
            ;;
    esac
    
    # Print summary
    echo ""
    print_summary
}

# Run main with all arguments
main "$@"
