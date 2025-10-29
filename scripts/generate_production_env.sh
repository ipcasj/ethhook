#!/bin/bash

# =============================================================================
# Generate Production Environment File
# =============================================================================
# This script creates .env.production with secure passwords
#
# Usage:
#   ./scripts/generate_production_env.sh
# =============================================================================

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔═══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Generate Production Environment     ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════╝${NC}"
echo ""

# Generate secure passwords
echo -e "${YELLOW}[1/4]${NC} Generating secure passwords..."
POSTGRES_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-32)
REDIS_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-32)
JWT_SECRET=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-43)

echo -e "${GREEN}✓${NC} Passwords generated"
echo ""

# Create .env.production file
echo -e "${YELLOW}[2/4]${NC} Creating .env.production file..."

cat > .env.production << EOF
# =============================================================================
# ETHHOOK DIGITALOCEAN PRODUCTION CONFIGURATION
# =============================================================================
# All-in-One Droplet: PostgreSQL + Redis + All Services in Docker
# Cost: \$24/month
#
# Auto-generated: $(date)
# =============================================================================

# =============================================================================
# DATABASE CONFIGURATION (PostgreSQL in Docker)
# =============================================================================
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
DATABASE_URL=postgresql://ethhook:${POSTGRES_PASSWORD}@postgres:5432/ethhook

DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# =============================================================================
# REDIS CONFIGURATION (Redis in Docker)
# =============================================================================
REDIS_PASSWORD=${REDIS_PASSWORD}
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_POOL_SIZE=10

# =============================================================================
# ETHEREUM RPC PROVIDER
# =============================================================================
# For DEMO/MVP: Use Sepolia testnet
ENVIRONMENT=development
ETHEREUM_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
ETHEREUM_WS_URL=wss://eth-sepolia.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW

# For PRODUCTION (uncomment when ready):
# ENVIRONMENT=production
# ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
# ETHEREUM_WS_URL=wss://eth-mainnet.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW

# =============================================================================
# SECURITY
# =============================================================================
JWT_SECRET=${JWT_SECRET}
JWT_EXPIRATION_HOURS=24

# =============================================================================
# SERVICE CONFIGURATION
# =============================================================================
WORKER_COUNT=50
RUST_LOG=info,ethhook=info,sqlx=warn

# API
API_HOST=0.0.0.0
API_PORT=3000

# Webhooks
WEBHOOK_TIMEOUT_SECONDS=30
WEBHOOK_MAX_RETRIES=5
WEBHOOK_WORKER_THREADS=50

# CORS (update with your domain)
CORS_ALLOWED_ORIGINS=*

# Metrics
PROMETHEUS_METRICS_PORT=9090

# Grafana
GRAFANA_PASSWORD=admin
EOF

echo -e "${GREEN}✓${NC} .env.production created"
echo ""

# Show what was generated
echo -e "${YELLOW}[3/4]${NC} Generated credentials (save these securely!):"
echo ""
echo -e "  ${BLUE}POSTGRES_PASSWORD:${NC}"
echo -e "    $POSTGRES_PASSWORD"
echo ""
echo -e "  ${BLUE}REDIS_PASSWORD:${NC}"
echo -e "    $REDIS_PASSWORD"
echo ""
echo -e "  ${BLUE}JWT_SECRET:${NC}"
echo -e "    $JWT_SECRET"
echo ""

# Backup existing file if it exists
if [ -f .env.production.backup ]; then
    echo -e "${YELLOW}[4/4]${NC} Backup of previous .env.production already exists"
else
    echo -e "${YELLOW}[4/4]${NC} No previous backup needed"
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  .env.production Ready!               ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
echo ""
echo "Next steps:"
echo "  1. Review .env.production"
echo "  2. Update ETHEREUM_RPC_URL if needed"
echo "  3. Deploy: ./deploy.sh YOUR_DROPLET_IP"
echo ""
echo "⚠️  Keep these passwords secure!"
echo "   Consider storing them in a password manager"
echo ""
