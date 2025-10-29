#!/bin/bash

# =============================================================================
# EthHook Backup Script
# =============================================================================
# Backs up PostgreSQL database and Redis data
#
# Usage:
#   ./scripts/backup.sh
#
# Backups are stored in: /root/ethhook-backups/
#
# Setup automatic daily backups:
#   crontab -e
#   # Add this line to backup daily at 3 AM:
#   0 3 * * * /root/ethhook/scripts/backup.sh >> /var/log/ethhook-backup.log 2>&1
# =============================================================================

set -e

# Configuration
BACKUP_DIR="/root/ethhook-backups"
DATE=$(date +%Y%m%d_%H%M%S)
KEEP_DAYS=7  # Keep backups for 7 days

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "================================================================="
echo "  EthHook Backup - $(date)"
echo "================================================================="

# Create backup directory
mkdir -p "$BACKUP_DIR"

# =============================================================================
# Backup PostgreSQL
# =============================================================================
echo -e "${YELLOW}[1/3]${NC} Backing up PostgreSQL database..."

PG_BACKUP_FILE="$BACKUP_DIR/postgres_${DATE}.sql.gz"

docker exec ethhook-postgres pg_dump -U ethhook ethhook | gzip > "$PG_BACKUP_FILE"

PG_SIZE=$(du -h "$PG_BACKUP_FILE" | cut -f1)
echo -e "${GREEN}✓${NC} PostgreSQL backup created: $PG_BACKUP_FILE ($PG_SIZE)"

# =============================================================================
# Backup Redis
# =============================================================================
echo -e "${YELLOW}[2/3]${NC} Backing up Redis data..."

# Trigger Redis to save
docker exec ethhook-redis redis-cli -a "${REDIS_PASSWORD}" --no-auth-warning SAVE

# Copy Redis dump file
REDIS_BACKUP_FILE="$BACKUP_DIR/redis_${DATE}.rdb"
docker cp ethhook-redis:/data/dump.rdb "$REDIS_BACKUP_FILE"

REDIS_SIZE=$(du -h "$REDIS_BACKUP_FILE" | cut -f1)
echo -e "${GREEN}✓${NC} Redis backup created: $REDIS_BACKUP_FILE ($REDIS_SIZE)"

# =============================================================================
# Clean up old backups
# =============================================================================
echo -e "${YELLOW}[3/3]${NC} Cleaning up old backups (older than $KEEP_DAYS days)..."

find "$BACKUP_DIR" -name "postgres_*.sql.gz" -mtime +$KEEP_DAYS -delete
find "$BACKUP_DIR" -name "redis_*.rdb" -mtime +$KEEP_DAYS -delete

REMAINING=$(ls -1 "$BACKUP_DIR" | wc -l)
echo -e "${GREEN}✓${NC} Cleanup complete. $REMAINING backup files remaining."

# =============================================================================
# Summary
# =============================================================================
echo ""
echo "================================================================="
echo "  Backup Complete!"
echo "================================================================="
echo "  Location: $BACKUP_DIR"
echo "  PostgreSQL: $PG_BACKUP_FILE ($PG_SIZE)"
echo "  Redis:      $REDIS_BACKUP_FILE ($REDIS_SIZE)"
echo ""
echo "To restore:"
echo "  PostgreSQL: gunzip < $PG_BACKUP_FILE | docker exec -i ethhook-postgres psql -U ethhook ethhook"
echo "  Redis:      docker cp $REDIS_BACKUP_FILE ethhook-redis:/data/dump.rdb && docker restart ethhook-redis"
echo "================================================================="
