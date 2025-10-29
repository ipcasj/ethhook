#!/bin/bash

# =============================================================================
# EthHook DigitalOcean Deployment Script
# =============================================================================
# This script deploys EthHook to a DigitalOcean Droplet
#
# Prerequisites:
# - DigitalOcean Droplet with Docker installed
# - Managed PostgreSQL and Redis created
# - .env.production file with all variables set
#
# Usage:
#   ./deploy.sh <droplet-ip>
#
# Example:
#   ./deploy.sh 147.182.123.456
# =============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check arguments
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Droplet IP address required${NC}"
    echo "Usage: ./deploy.sh <droplet-ip>"
    echo "Example: ./deploy.sh 147.182.123.456"
    exit 1
fi

DROPLET_IP=$1
SSH_USER="root"  # Change if using different user
PROJECT_DIR="/root/ethhook"

echo -e "${BLUE}╔═══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   EthHook DigitalOcean Deployment    ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════╝${NC}"
echo ""

# =============================================================================
# Step 1: Check if .env.production exists
# =============================================================================
echo -e "${YELLOW}[1/7]${NC} Checking environment configuration..."

if [ ! -f ".env.production" ]; then
    echo -e "${RED}Error: .env.production not found${NC}"
    echo "Please create .env.production from .env.digitalocean.example"
    echo "  1. cp .env.digitalocean.example .env.production"
    echo "  2. Edit .env.production and fill in your values"
    exit 1
fi

echo -e "${GREEN}✓${NC} Environment file found"
echo ""

# =============================================================================
# Step 2: Test SSH connection
# =============================================================================
echo -e "${YELLOW}[2/7]${NC} Testing SSH connection to ${DROPLET_IP}..."

if ssh -o ConnectTimeout=10 -o BatchMode=yes ${SSH_USER}@${DROPLET_IP} exit 2>/dev/null; then
    echo -e "${GREEN}✓${NC} SSH connection successful"
else
    echo -e "${RED}Error: Cannot connect to ${DROPLET_IP}${NC}"
    echo "Make sure:"
    echo "  1. Droplet IP is correct"
    echo "  2. SSH key is added to the Droplet"
    echo "  3. Firewall allows SSH (port 22)"
    exit 1
fi
echo ""

# =============================================================================
# Step 3: Create project directory on Droplet
# =============================================================================
echo -e "${YELLOW}[3/7]${NC} Setting up project directory..."

ssh ${SSH_USER}@${DROPLET_IP} << 'EOF'
    mkdir -p /root/ethhook
    echo "✓ Created /root/ethhook"
EOF

echo -e "${GREEN}✓${NC} Project directory ready"
echo ""

# =============================================================================
# Step 4: Copy files to Droplet
# =============================================================================
echo -e "${YELLOW}[4/7]${NC} Copying project files..."

# Create tar archive excluding unnecessary files
tar czf /tmp/ethhook-deploy.tar.gz \
    --exclude='target' \
    --exclude='node_modules' \
    --exclude='.git' \
    --exclude='*.log' \
    --exclude='demo-webhook-receiver' \
    .

# Copy to Droplet
scp /tmp/ethhook-deploy.tar.gz ${SSH_USER}@${DROPLET_IP}:${PROJECT_DIR}/

# Extract on Droplet
ssh ${SSH_USER}@${DROPLET_IP} << EOF
    cd ${PROJECT_DIR}
    tar xzf ethhook-deploy.tar.gz
    rm ethhook-deploy.tar.gz
    echo "✓ Extracted project files"
EOF

# Copy environment file
scp .env.production ${SSH_USER}@${DROPLET_IP}:${PROJECT_DIR}/.env.production

echo -e "${GREEN}✓${NC} Files copied successfully"
echo ""

# =============================================================================
# Step 5: Install Docker if not present
# =============================================================================
echo -e "${YELLOW}[5/7]${NC} Checking Docker installation..."

ssh ${SSH_USER}@${DROPLET_IP} << 'EOF'
    if ! command -v docker &> /dev/null; then
        echo "Installing Docker..."
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh
        systemctl enable docker
        systemctl start docker
        echo "✓ Docker installed"
    else
        echo "✓ Docker already installed"
    fi

    if ! command -v docker-compose &> /dev/null; then
        echo "Installing Docker Compose..."
        curl -L "https://github.com/docker/compose/releases/download/v2.24.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose
        echo "✓ Docker Compose installed"
    else
        echo "✓ Docker Compose already installed"
    fi
EOF

echo -e "${GREEN}✓${NC} Docker ready"
echo ""

# =============================================================================
# Step 6: Build and start services
# =============================================================================
echo -e "${YELLOW}[6/7]${NC} Building and starting services..."
echo -e "${BLUE}This may take 10-15 minutes for the first build...${NC}"

ssh ${SSH_USER}@${DROPLET_IP} << 'EOF'
    cd /root/ethhook

    # Stop any existing containers
    docker-compose -f docker-compose.prod.yml down 2>/dev/null || true

    # Build images
    echo "Building Docker images..."
    docker-compose -f docker-compose.prod.yml build --no-cache

    # Start services
    echo "Starting services..."
    docker-compose -f docker-compose.prod.yml up -d

    echo "✓ Services started"
EOF

echo -e "${GREEN}✓${NC} Services deployed"
echo ""

# =============================================================================
# Step 7: Verify deployment
# =============================================================================
echo -e "${YELLOW}[7/7]${NC} Verifying deployment..."

echo "Waiting 30 seconds for services to start..."
sleep 30

ssh ${SSH_USER}@${DROPLET_IP} << 'EOF'
    cd /root/ethhook
    echo ""
    echo "Service Status:"
    echo "─────────────────────────────────────────"
    docker-compose -f docker-compose.prod.yml ps
    echo ""
    echo "Health Checks:"
    echo "─────────────────────────────────────────"

    # Check each service
    for service in event-ingestor message-processor webhook-delivery admin-api; do
        port=$(docker-compose -f docker-compose.prod.yml port $service 808 2>/dev/null | cut -d: -f2 | head -1)
        if [ -n "$port" ]; then
            if curl -f -s http://localhost:$port/health > /dev/null 2>&1; then
                echo "✓ $service: healthy"
            else
                echo "✗ $service: unhealthy"
            fi
        fi
    done
EOF

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     Deployment Complete! 🎉           ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
echo ""
echo "Your EthHook instance is running at: http://${DROPLET_IP}"
echo ""
echo "Service URLs:"
echo "  - Admin API:         http://${DROPLET_IP}:3000"
echo "  - Event Ingestor:    http://${DROPLET_IP}:8080/health"
echo "  - Message Processor: http://${DROPLET_IP}:8081/health"
echo "  - Webhook Delivery:  http://${DROPLET_IP}:8082/health"
echo "  - Grafana:           http://${DROPLET_IP}:3001 (admin/admin)"
echo ""
echo "Next steps:"
echo "  1. Set up DNS (point your domain to ${DROPLET_IP})"
echo "  2. Configure SSL/TLS (use Caddy or nginx)"
echo "  3. Run migrations: ssh ${SSH_USER}@${DROPLET_IP} 'cd ${PROJECT_DIR} && ./scripts/run_migrations.sh'"
echo "  4. Create test endpoint via Admin API"
echo "  5. Test webhook delivery"
echo ""
echo "To view logs:"
echo "  ssh ${SSH_USER}@${DROPLET_IP}"
echo "  cd ${PROJECT_DIR}"
echo "  docker-compose -f docker-compose.prod.yml logs -f"
echo ""
