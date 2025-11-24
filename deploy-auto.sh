#!/bin/bash
# =============================================================================
# EthHook - Automated Deployment Script
# =============================================================================
# This script automates the complete deployment process:
# - Builds all services
# - Runs tests
# - Builds Docker images
# - Pushes to registry
# - Deploys to server
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
REGISTRY="ghcr.io"
IMAGE_PREFIX="ghcr.io/ipcasj/ethhook"
VERSION=$(git describe --tags --always --dirty 2>/dev/null || echo "dev")
SERVICES=("admin-api" "pipeline" "ui" "demo-webhook-receiver")

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

step_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Parse arguments
MODE="${1:-full}"
DO_HOST="${2:-}"

show_help() {
    echo "EthHook Automated Deployment"
    echo ""
    echo "Usage: $0 [MODE] [DO_HOST]"
    echo ""
    echo "Modes:"
    echo "  full         - Complete pipeline (build + test + docker + deploy)"
    echo "  build        - Build Rust binaries only"
    echo "  test         - Run tests only"
    echo "  docker       - Build Docker images only"
    echo "  push         - Push Docker images to registry"
    echo "  deploy       - Deploy to server (requires DO_HOST)"
    echo "  quick-deploy - Pull and restart on server (no rebuild)"
    echo ""
    echo "Examples:"
    echo "  $0 full 147.182.123.456          # Full deployment"
    echo "  $0 build                         # Build only"
    echo "  $0 quick-deploy 147.182.123.456  # Quick update"
    echo ""
    exit 0
}

[[ "$MODE" == "help" ]] || [[ "$MODE" == "-h" ]] || [[ "$MODE" == "--help" ]] && show_help

# =============================================================================
# Step 1: Environment Check
# =============================================================================
step_header "Step 1: Environment Check"

log_info "Checking required tools..."
command -v cargo >/dev/null 2>&1 || { log_error "cargo not found"; exit 1; }
command -v docker >/dev/null 2>&1 || { log_error "docker not found"; exit 1; }
command -v git >/dev/null 2>&1 || { log_error "git not found"; exit 1; }

log_success "All required tools found"
log_info "Version: $VERSION"
log_success "Environment prepared"

# =============================================================================
# Step 2: Build Rust Binaries
# =============================================================================
if [[ "$MODE" == "full" ]] || [[ "$MODE" == "build" ]]; then
    step_header "Step 2: Building Rust Workspace"
    
    log_info "Compiling with SQLX offline mode..."
    SQLX_OFFLINE=true cargo build --workspace --release
    
    log_success "Build complete"
    
    [[ "$MODE" == "build" ]] && exit 0
fi

# =============================================================================
# Step 3: Run Tests
# =============================================================================
if [[ "$MODE" == "full" ]] || [[ "$MODE" == "test" ]]; then
    step_header "Step 3: Running Tests"
    
    log_info "Running unit tests..."
    DATABASE_URL=sqlite:test.db cargo test --workspace --lib --bins
    
    log_info "Running integration tests..."
    DATABASE_URL=sqlite:test.db cargo test --test '*' -- --test-threads=1
    
    log_success "All tests passed"
    
    [[ "$MODE" == "test" ]] && exit 0
fi

# =============================================================================
# Step 4: Build Docker Images
# =============================================================================
if [[ "$MODE" == "full" ]] || [[ "$MODE" == "docker" ]]; then
    step_header "Step 4: Building Docker Images"
    
    for service in "${SERVICES[@]}"; do
        log_info "Building $service..."
        
        if [[ -f "crates/$service/Dockerfile" ]]; then
            docker build \
                -f "crates/$service/Dockerfile" \
                -t "$IMAGE_PREFIX-$service:$VERSION" \
                -t "$IMAGE_PREFIX-$service:latest" \
                .
        elif [[ -f "$service/Dockerfile" ]]; then
            docker build \
                -f "$service/Dockerfile" \
                -t "$IMAGE_PREFIX-$service:$VERSION" \
                -t "$IMAGE_PREFIX-$service:latest" \
                "$service/"
        else
            log_warning "Dockerfile not found for $service"
            continue
        fi
        
        log_success "$service image built"
    done
    
    [[ "$MODE" == "docker" ]] && exit 0
fi

# =============================================================================
# Step 5: Push Docker Images
# =============================================================================
if [[ "$MODE" == "full" ]] || [[ "$MODE" == "push" ]]; then
    step_header "Step 5: Pushing Docker Images"
    
    log_info "Checking Docker registry authentication..."
    
    # Try to push (will fail if not authenticated)
    for service in "${SERVICES[@]}"; do
        log_info "Pushing $service:$VERSION..."
        docker push "$IMAGE_PREFIX-$service:$VERSION" || {
            log_error "Failed to push $service. Are you logged in to $REGISTRY?"
            log_info "Run: echo \$GITHUB_TOKEN | docker login ghcr.io -u \$GITHUB_ACTOR --password-stdin"
            exit 1
        }
        
        log_info "Pushing $service:latest..."
        docker push "$IMAGE_PREFIX-$service:latest"
        
        log_success "$service pushed"
    done
    
    log_success "All images pushed to registry"
    
    [[ "$MODE" == "push" ]] && exit 0
fi

# =============================================================================
# Step 6: Deploy to Server
# =============================================================================
if [[ "$MODE" == "full" ]] || [[ "$MODE" == "deploy" ]]; then
    step_header "Step 6: Deploying to Server"
    
    if [[ -z "$DO_HOST" ]]; then
        log_error "Server host required for deployment"
        echo "Usage: $0 deploy <server-ip>"
        exit 1
    fi
    
    log_info "Deploying to $DO_HOST..."
    
    # Test SSH connection
    log_info "Testing SSH connection..."
    ssh -o ConnectTimeout=10 -o BatchMode=yes root@"$DO_HOST" exit 2>/dev/null || {
        log_error "Cannot connect to $DO_HOST"
        exit 1
    }
    log_success "SSH connection verified"
    
    # Copy production config
    log_info "Copying configuration files..."
    scp .env.production root@"$DO_HOST":~/ethhook/.env
    scp docker-compose.prod.yml root@"$DO_HOST":~/ethhook/docker-compose.yml
    
    # Pull and restart services
    log_info "Pulling latest images on server..."
    ssh root@"$DO_HOST" << 'ENDSSH'
        cd ~/ethhook
        echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$GITHUB_ACTOR" --password-stdin 2>/dev/null || true
        docker-compose pull
        docker-compose up -d
        echo "Waiting for services to start..."
        sleep 15
        docker-compose ps
ENDSSH
    
    log_success "Deployment complete!"
    
    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ EthHook deployed successfully!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Service URLs:"
    echo "  - Frontend:   http://$DO_HOST:3002"
    echo "  - Admin API:  http://$DO_HOST:3000"
    echo "  - Grafana:    http://$DO_HOST:3001"
    echo ""
fi

# =============================================================================
# Quick Deploy Mode
# =============================================================================
if [[ "$MODE" == "quick-deploy" ]]; then
    step_header "Quick Deploy (Pull + Restart)"
    
    if [[ -z "$DO_HOST" ]]; then
        log_error "Server host required"
        echo "Usage: $0 quick-deploy <server-ip>"
        exit 1
    fi
    
    log_info "Quick deploying to $DO_HOST..."
    
    ssh root@"$DO_HOST" << 'ENDSSH'
        cd ~/ethhook
        docker-compose pull
        docker-compose up -d
        docker-compose ps
ENDSSH
    
    log_success "Quick deploy complete!"
fi

log_success "All operations completed successfully!"
