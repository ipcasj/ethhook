#!/bin/bash
# Database Immediate Optimizations
# Run this script to apply critical performance fixes

set -e

echo "==================================="
echo "Database Performance Optimization"
echo "==================================="
echo ""

# Check if running on production
read -p "This will modify production database. Continue? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "Phase 1: Dropping unused indexes..."
echo "-----------------------------------"

ssh root@104.248.15.178 "docker exec ethhook-postgres psql -U ethhook -d ethhook" <<'EOF'
-- Drop unused indexes to save 1GB+ space and improve write performance
BEGIN;

DROP INDEX IF EXISTS idx_events_transaction_hash;          -- 425MB, 0 scans
DROP INDEX IF EXISTS idx_delivery_attempts_attempted_at;   -- 217MB, 0 scans
DROP INDEX IF EXISTS idx_events_block_number;              -- 63MB, 0 scans

-- Keep idx_events_ingested_block for now (may be used after optimization)
-- DROP INDEX IF EXISTS idx_events_ingested_block;         -- 214MB, 0 scans

COMMIT;

SELECT 'Unused indexes dropped' as status;
EOF

echo ""
echo "Phase 2: Creating optimized indexes..."
echo "---------------------------------------"

# Create indexes CONCURRENTLY (won't block)
ssh root@104.248.15.178 "docker exec ethhook-postgres psql -U ethhook -d ethhook -c \"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_delivery_attempts_user_id ON delivery_attempts(user_id) WHERE user_id IS NOT NULL;\""

echo "Created: idx_delivery_attempts_user_id"

ssh root@104.248.15.178 "docker exec ethhook-postgres psql -U ethhook -d ethhook -c \"CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_delivery_attempts_user_success ON delivery_attempts(user_id, success) WHERE user_id IS NOT NULL;\""

echo "Created: idx_delivery_attempts_user_success"

echo ""
echo "Phase 3: Optimizing PostgreSQL configuration..."
echo "------------------------------------------------"

ssh root@104.248.15.178 "docker exec ethhook-postgres psql -U ethhook -d ethhook" <<'EOF'
-- Optimize for 8GB RAM server with SSD
ALTER SYSTEM SET shared_buffers = '2GB';
ALTER SYSTEM SET work_mem = '32MB';
ALTER SYSTEM SET maintenance_work_mem = '512MB';
ALTER SYSTEM SET effective_cache_size = '6GB';
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
ALTER SYSTEM SET max_parallel_workers_per_gather = 2;
ALTER SYSTEM SET max_parallel_workers = 4;

SELECT 'Configuration updated - requires PostgreSQL restart' as status;
EOF

echo ""
echo "Phase 4: Enabling query performance monitoring..."
echo "--------------------------------------------------"

ssh root@104.248.15.178 "docker exec ethhook-postgres psql -U ethhook -d ethhook -c \"CREATE EXTENSION IF NOT EXISTS pg_stat_statements;\""

echo ""
echo "Phase 5: Checking current state..."
echo "-----------------------------------"

ssh root@104.248.15.178 "docker exec ethhook-postgres psql -U ethhook -d ethhook" <<'EOF'
SELECT 
    'Delivery Attempts' as table_name,
    COUNT(*) as total_rows,
    COUNT(user_id) as rows_with_user_id,
    ROUND(100.0 * COUNT(user_id) / COUNT(*), 2) as pct_populated
FROM delivery_attempts
UNION ALL
SELECT 
    'Events' as table_name,
    COUNT(*) as total_rows,
    0 as rows_with_user_id,
    0.0 as pct_populated
FROM events;

-- Check if UPDATE is still running
SELECT pid, state, query_start, age(now(), query_start) as duration
FROM pg_stat_activity 
WHERE query LIKE '%UPDATE delivery_attempts%' AND state = 'active';
EOF

echo ""
echo "====================================="
echo "Phase 1-5 Complete!"
echo "====================================="
echo ""
echo "NEXT STEPS:"
echo ""
echo "1. Wait for UPDATE delivery_attempts to complete (check progress above)"
echo "2. Restart PostgreSQL to apply configuration:"
echo "   ssh root@104.248.15.178 'docker restart ethhook-postgres'"
echo "3. Restart dependent services after PostgreSQL is ready:"
echo "   ssh root@104.248.15.178 'docker restart ethhook-admin-api ethhook-message-processor ethhook-webhook-delivery'"
echo ""
echo "Expected improvements after restart:"
echo "- Dashboard queries: 50-300s → 5-15s"
echo "- Memory usage: 7.4GB → 6GB"
echo "- Disk space saved: ~700MB"
echo ""
