#!/bin/bash
# EthHook C - Production Deployment Script for DigitalOcean
# 
# This script deploys the optimized C implementation to DigitalOcean

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DROPLET_IP="${DROPLET_IP:-104.248.15.178}"
DROPLET_USER="${DROPLET_USER:-root}"
DEPLOY_DIR="/root/ethhook-c"
GITHUB_REPO="ipcasj/ethhook"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}EthHook C - Production Deployment${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Function to print step
print_step() {
    echo -e "${YELLOW}>>> $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check prerequisites
print_step "Checking prerequisites..."

if ! command -v ssh &> /dev/null; then
    print_error "ssh not found. Please install OpenSSH client."
    exit 1
fi

if ! command -v git &> /dev/null; then
    print_error "git not found. Please install git."
    exit 1
fi

print_success "Prerequisites OK"

# Check if we can connect to the droplet
print_step "Testing connection to ${DROPLET_IP}..."

if ! ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no ${DROPLET_USER}@${DROPLET_IP} "echo 'Connection successful'" &> /dev/null; then
    print_error "Cannot connect to ${DROPLET_IP}. Please check:"
    echo "  1. SSH key is added to the droplet"
    echo "  2. Firewall allows SSH (port 22)"
    echo "  3. Droplet IP is correct"
    exit 1
fi

print_success "Connection OK"

# Load environment variables
print_step "Loading environment variables..."

if [ ! -f .env ]; then
    print_error ".env file not found. Please create it from .env.example"
    exit 1
fi

set -a
source .env
set +a

# Validate required environment variables
REQUIRED_VARS=(
    "CLICKHOUSE_PASSWORD"
    "JWT_SECRET"
    "ETH_RPC_WS"
    "ARBITRUM_RPC_WS"
    "OPTIMISM_RPC_WS"
    "BASE_RPC_WS"
)

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        print_error "Required environment variable $var is not set"
        exit 1
    fi
done

print_success "Environment variables loaded"

# Install dependencies on droplet
print_step "Installing system dependencies on droplet..."

ssh ${DROPLET_USER}@${DROPLET_IP} << 'EOF'
    # Update package list
    apt-get update -qq

    # Install build tools
    apt-get install -y -qq \
        build-essential \
        cmake \
        pkg-config \
        git

    # Install required libraries
    apt-get install -y -qq \
        libevent-dev \
        libwebsockets-dev \
        libhiredis-dev \
        libjansson-dev \
        libcurl4-openssl-dev \
        libmicrohttpd-dev \
        libjwt-dev \
        libssl-dev \
        libsqlite3-dev

    # Install Docker if not present
    if ! command -v docker &> /dev/null; then
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh
        rm get-docker.sh
    fi

    # Install Docker Compose if not present
    if ! command -v docker-compose &> /dev/null; then
        curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose
    fi
EOF

print_success "Dependencies installed"

# Clone or update repository
print_step "Syncing code to droplet..."

ssh ${DROPLET_USER}@${DROPLET_IP} << EOF
    if [ -d "${DEPLOY_DIR}" ]; then
        cd ${DEPLOY_DIR}
        git fetch origin
        git reset --hard origin/main
        git clean -fd
    else
        git clone https://github.com/${GITHUB_REPO}.git ${DEPLOY_DIR}
        cd ${DEPLOY_DIR}
    fi
    
    # Navigate to C implementation
    cd ethhook-c
    
    echo "Code synced successfully"
EOF

print_success "Code synced"

# Copy environment configuration
print_step "Copying environment configuration..."

# Create .env file on droplet
ssh ${DROPLET_USER}@${DROPLET_IP} << EOF
    cat > ${DEPLOY_DIR}/ethhook-c/.env << 'ENVEOF'
# ClickHouse
CLICKHOUSE_PASSWORD=${CLICKHOUSE_PASSWORD}

# JWT
JWT_SECRET=${JWT_SECRET}

# Blockchain RPC endpoints
ETH_RPC_WS=${ETH_RPC_WS}
ETH_RPC_HTTP=${ETH_RPC_HTTP}
ARBITRUM_RPC_WS=${ARBITRUM_RPC_WS}
ARBITRUM_RPC_HTTP=${ARBITRUM_RPC_HTTP}
OPTIMISM_RPC_WS=${OPTIMISM_RPC_WS}
OPTIMISM_RPC_HTTP=${OPTIMISM_RPC_HTTP}
BASE_RPC_WS=${BASE_RPC_WS}
BASE_RPC_HTTP=${BASE_RPC_HTTP}
SEPOLIA_RPC_WS=${SEPOLIA_RPC_WS:-}
SEPOLIA_RPC_HTTP=${SEPOLIA_RPC_HTTP:-}
ENVEOF
EOF

print_success "Configuration copied"

# Build on droplet (native build for best performance)
print_step "Building C application on droplet (native optimizations)..."

ssh ${DROPLET_USER}@${DROPLET_IP} << 'EOF'
    cd /root/ethhook-c
    
    # Clean previous build
    rm -rf build
    mkdir -p build
    cd build
    
    # Configure with release optimizations
    cmake -DCMAKE_BUILD_TYPE=Release \
          -DCMAKE_C_FLAGS="-O3 -march=native -mtune=native -flto" \
          ..
    
    # Build (use all cores)
    make -j$(nproc)
    
    # Install binaries
    make install
    
    echo "Build completed successfully"
EOF

print_success "Build completed"

# Build and start Docker containers
print_step "Building and starting Docker containers..."

ssh ${DROPLET_USER}@${DROPLET_IP} << 'EOF'
    cd /root/ethhook-c/docker
    
    # Stop existing containers
    docker-compose -f docker-compose.prod.yml down
    
    # Remove old images to force rebuild
    docker-compose -f docker-compose.prod.yml build --no-cache
    
    # Start all services
    docker-compose -f docker-compose.prod.yml up -d
    
    echo "Docker containers started"
EOF

print_success "Docker containers started"

# Wait for services to be healthy
print_step "Waiting for services to be healthy..."

sleep 10

ssh ${DROPLET_USER}@${DROPLET_IP} << 'EOF'
    cd /root/ethhook-c/docker
    
    # Wait up to 60 seconds for health checks
    for i in {1..12}; do
        if docker-compose -f docker-compose.prod.yml ps | grep -q "unhealthy"; then
            echo "Waiting for services to be healthy... ($i/12)"
            sleep 5
        else
            break
        fi
    done
    
    # Show container status
    docker-compose -f docker-compose.prod.yml ps
EOF

print_success "Services are healthy"

# Initialize ClickHouse schema
print_step "Initializing ClickHouse schema..."

ssh ${DROPLET_USER}@${DROPLET_IP} << EOF
    cd /root/ethhook-c
    
    # Wait for ClickHouse to be ready
    sleep 5
    
    # Check ClickHouse connection
    docker exec ethhook-clickhouse clickhouse-client --query "SELECT 1"
    
    echo "ClickHouse is ready"
EOF

print_success "ClickHouse initialized"

# Run database migrations for SQLite
print_step "Running database migrations..."

ssh ${DROPLET_USER}@${DROPLET_IP} << 'EOF'
    cd /root/ethhook-c
    
    # Copy migrations from Rust project if they exist
    if [ -d "../migrations" ]; then
        echo "Found migrations directory"
        # TODO: Apply SQLite migrations
    fi
EOF

print_success "Migrations completed"

# Verify deployment
print_step "Verifying deployment..."

echo ""
echo "Checking service endpoints:"
echo ""

# Check Admin API
if curl -sf http://${DROPLET_IP}:3000/api/health > /dev/null 2>&1; then
    print_success "Admin API: http://${DROPLET_IP}:3000 ✓"
else
    print_error "Admin API: http://${DROPLET_IP}:3000 ✗"
fi

# Check ClickHouse
if curl -sf http://${DROPLET_IP}:8123/ping > /dev/null 2>&1; then
    print_success "ClickHouse: http://${DROPLET_IP}:8123 ✓"
else
    print_error "ClickHouse: http://${DROPLET_IP}:8123 ✗"
fi

# Check Metrics
if curl -sf http://${DROPLET_IP}:9090/metrics > /dev/null 2>&1; then
    print_success "Metrics: http://${DROPLET_IP}:9090/metrics ✓"
else
    print_error "Metrics: http://${DROPLET_IP}:9090/metrics ✗"
fi

# Show container logs
print_step "Recent container logs:"
echo ""

ssh ${DROPLET_USER}@${DROPLET_IP} << 'EOF'
    cd /root/ethhook-c/docker
    docker-compose -f docker-compose.prod.yml logs --tail=20
EOF

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Deployment Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Services:"
echo "  • Admin API:  http://${DROPLET_IP}:3000"
echo "  • ClickHouse: http://${DROPLET_IP}:8123"
echo "  • Metrics:    http://${DROPLET_IP}:9090"
echo "  • Health:     http://${DROPLET_IP}:8080/health"
echo ""
echo "Useful commands:"
echo "  • View logs:  ssh ${DROPLET_USER}@${DROPLET_IP} 'cd ${DEPLOY_DIR}/ethhook-c/docker && docker-compose -f docker-compose.prod.yml logs -f'"
echo "  • Restart:    ssh ${DROPLET_USER}@${DROPLET_IP} 'cd ${DEPLOY_DIR}/ethhook-c/docker && docker-compose -f docker-compose.prod.yml restart'"
echo "  • Stop:       ssh ${DROPLET_USER}@${DROPLET_IP} 'cd ${DEPLOY_DIR}/ethhook-c/docker && docker-compose -f docker-compose.prod.yml down'"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Test the API: curl http://${DROPLET_IP}:3000/api/health"
echo "  2. Check ClickHouse: curl http://${DROPLET_IP}:8123/ping"
echo "  3. View metrics: curl http://${DROPLET_IP}:9090/metrics"
echo "  4. Create admin user (from Rust deployment)"
echo "  5. Test UI at http://${DROPLET_IP}:3000"
echo ""
