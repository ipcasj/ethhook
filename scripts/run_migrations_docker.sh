#!/bin/bash

# =============================================================================
# Run Database Migrations (Docker-based)
# =============================================================================
# This script runs migrations using a temporary Docker container
# Useful when sqlx-cli is not installed on the host
#
# Usage (on Droplet):
#   cd /root/ethhook
#   ./scripts/run_migrations_docker.sh
# =============================================================================

set -e

echo "🔄 Running database migrations..."

# Get database URL from .env.production
if [ -f ".env.production" ]; then
    export $(grep DATABASE_URL .env.production | xargs)
else
    echo "Error: .env.production not found"
    exit 1
fi

# Run migrations using temporary Rust container
docker run --rm \
    --network ethhook_ethhook-network \
    -v $(pwd)/migrations:/migrations \
    -e DATABASE_URL="${DATABASE_URL}" \
    rust:1.83-slim \
    bash -c "
        apt-get update && apt-get install -y libpq-dev > /dev/null 2>&1
        cargo install sqlx-cli --no-default-features --features postgres --quiet
        cd /migrations && sqlx migrate run
    "

echo "✅ Migrations complete!"
