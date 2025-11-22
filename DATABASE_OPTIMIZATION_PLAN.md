# Database Performance Optimization Plan

## Executive Summary
System is experiencing severe performance degradation due to:
1. **Under-configured PostgreSQL** (128MB shared_buffers for 5.6M+ rows)
2. **Inefficient query patterns** (4-table joins on millions of rows)
3. **1GB+ of unused indexes** wasting disk I/O
4. **Missing denormalization** (no user_id in delivery_attempts)
5. **No query result caching**

## Current State Analysis

### Database Configuration Issues
```
Current:
- shared_buffers: 128MB (16384 * 8KB)
- work_mem: 4MB
- maintenance_work_mem: 64MB
- effective_cache_size: default (~4GB)

Recommended for 8GB RAM server:
- shared_buffers: 2GB
- work_mem: 32MB
- maintenance_work_mem: 512MB
- effective_cache_size: 6GB
- random_page_cost: 1.1 (SSD)
- effective_io_concurrency: 200
```

### Table Statistics
- **events**: 4.7M rows, 4.3GB total (2.8GB table + 1.5GB indexes)
- **delivery_attempts**: 5.6M rows, 2.3GB total
- **Total database size**: ~7GB

### Unused Indexes (Wasting 1GB+)
```sql
idx_events_transaction_hash (425MB) - 0 scans
delivery_attempts_pkey (263MB) - 0 scans  
idx_delivery_attempts_attempted_at (217MB) - 0 scans
idx_events_ingested_block (214MB) - 0 scans
idx_events_block_number (63MB) - 0 scans
idx_delivery_attempts_success_duration (51MB) - 0 scans
```

### Critical Query Pattern Problems

#### Problem 1: Dashboard Statistics (lines 37-110 in statistics.rs)
```rust
// BAD: Joins 4 tables for every dashboard load
SELECT COUNT(DISTINCT e.id) FROM events e
JOIN delivery_attempts da ON e.id = da.event_id  -- 5.6M × 4.7M comparison
JOIN endpoints ep ON da.endpoint_id = ep.id
JOIN applications a ON ep.application_id = a.id
WHERE a.user_id = $1 AND e.ingested_at >= NOW() - INTERVAL '24 hours'
```
**Performance**: 50-300 seconds per query
**Root cause**: No direct path from events to user_id

#### Problem 2: Event Listing (events.rs lines 158-220)
```rust
SELECT DISTINCT ON (e.id) ... FROM events e
JOIN delivery_attempts da ON e.id = da.event_id
JOIN endpoints ep ON da.endpoint_id = ep.id
JOIN applications a ON ep.application_id = a.id
WHERE a.user_id = $1
ORDER BY e.id, e.block_number DESC, e.log_index DESC
LIMIT $2 OFFSET $3
```
**Performance**: 71-306 seconds per query
**Root cause**: DISTINCT ON with multiple joins, no covering indexes

#### Problem 3: Time-Series Analytics (statistics.rs line 303)
```rust
WITH time_buckets AS (
    SELECT date_trunc($1, e.ingested_at) as time_bucket,
           COUNT(DISTINCT e.id)::bigint as event_count,
           ...
    FROM events e
    LEFT JOIN delivery_attempts da ON e.id = da.event_id
    JOIN endpoints ep ON da.endpoint_id = ep.id
    JOIN applications a ON ep.application_id = a.id
    WHERE a.user_id = $2 AND e.ingested_at >= NOW() - ($3 || ' hours')::interval
    GROUP BY time_bucket
)
```
**Performance**: 29-276 seconds per query
**Root cause**: Aggregation after massive joins

## Optimization Strategy

### Phase 1: Immediate Fixes (Can Do Now)

#### 1.1 PostgreSQL Configuration
```sql
-- /var/lib/postgresql/data/postgresql.conf or via ALTER SYSTEM
ALTER SYSTEM SET shared_buffers = '2GB';
ALTER SYSTEM SET work_mem = '32MB';
ALTER SYSTEM SET maintenance_work_mem = '512MB';
ALTER SYSTEM SET effective_cache_size = '6GB';
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
ALTER SYSTEM SET max_parallel_workers_per_gather = 2;
ALTER SYSTEM SET max_parallel_workers = 4;
-- Requires PostgreSQL restart
```

#### 1.2 Drop Unused Indexes
```sql
-- Save 1GB+ disk space and improve write performance
DROP INDEX IF EXISTS idx_events_transaction_hash;
DROP INDEX IF EXISTS idx_delivery_attempts_attempted_at;
DROP INDEX IF EXISTS idx_events_block_number;
DROP INDEX IF EXISTS idx_delivery_attempts_success_duration;
```

#### 1.3 Add Critical Missing Indexes
```sql
-- For user filtering (MOST IMPORTANT)
CREATE INDEX CONCURRENTLY idx_delivery_attempts_user_id 
    ON delivery_attempts(user_id) WHERE user_id IS NOT NULL;

-- For time-range queries
CREATE INDEX CONCURRENTLY idx_events_ingested_id 
    ON events(ingested_at DESC, id);

-- Composite for common patterns
CREATE INDEX CONCURRENTLY idx_delivery_attempts_user_success_time 
    ON delivery_attempts(user_id, success, attempted_at DESC) 
    WHERE user_id IS NOT NULL;
```

### Phase 2: Schema Denormalization

#### 2.1 Add user_id to delivery_attempts (IN PROGRESS)
```sql
-- Already started but slow (5.6M rows)
-- Current UPDATE query running since 19:24
ALTER TABLE delivery_attempts ADD COLUMN user_id uuid;
UPDATE delivery_attempts da 
SET user_id = a.user_id
FROM endpoints ep
JOIN applications a ON ep.application_id = a.id
WHERE da.endpoint_id = ep.id;

-- Add NOT NULL constraint after population
ALTER TABLE delivery_attempts ALTER COLUMN user_id SET NOT NULL;

-- Add foreign key
ALTER TABLE delivery_attempts 
    ADD CONSTRAINT fk_delivery_attempts_user 
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
```

#### 2.2 Create Aggregate Tables
```sql
-- Pre-computed daily statistics per user
CREATE TABLE user_daily_stats (
    user_id uuid NOT NULL,
    date date NOT NULL,
    event_count bigint DEFAULT 0,
    delivery_count bigint DEFAULT 0,
    successful_deliveries bigint DEFAULT 0,
    failed_deliveries bigint DEFAULT 0,
    avg_duration_ms double precision,
    updated_at timestamp with time zone DEFAULT NOW(),
    PRIMARY KEY (user_id, date)
);

CREATE INDEX idx_user_daily_stats_date ON user_daily_stats(date DESC);

-- Materialized view for recent data (refresh every 5 min)
CREATE MATERIALIZED VIEW user_recent_stats AS
SELECT 
    da.user_id,
    COUNT(DISTINCT da.event_id) as event_count_24h,
    COUNT(*) as delivery_count_24h,
    COUNT(*) FILTER (WHERE da.success) as successful_24h,
    AVG(da.duration_ms) FILTER (WHERE da.duration_ms IS NOT NULL) as avg_duration_24h
FROM delivery_attempts da
WHERE da.attempted_at >= NOW() - INTERVAL '24 hours'
GROUP BY da.user_id;

CREATE UNIQUE INDEX ON user_recent_stats(user_id);
```

### Phase 3: Query Optimization

#### 3.1 Rewrite Dashboard Statistics
**Before** (statistics.rs lines 37-62):
```rust
let events_today = sqlx::query!(
    r#"SELECT COUNT(DISTINCT e.id) as "count!"
       FROM events e
       JOIN delivery_attempts da ON e.id = da.event_id
       JOIN endpoints ep ON da.endpoint_id = ep.id
       JOIN applications a ON ep.application_id = a.id
       WHERE a.user_id = $1 AND e.ingested_at >= NOW() - INTERVAL '24 hours'"#,
    auth_user.user_id
).fetch_one(&pool).await?;
```

**After** (with user_id denormalization):
```rust
let events_today = sqlx::query!(
    r#"SELECT COUNT(DISTINCT event_id) as "count!"
       FROM delivery_attempts
       WHERE user_id = $1 
       AND attempted_at >= NOW() - INTERVAL '24 hours'"#,
    auth_user.user_id
).fetch_one(&pool).await?;
```
**Expected improvement**: 50-300s → <100ms (3000x faster)

#### 3.2 Add Result Caching
```rust
// crates/admin-api/src/cache.rs
use moka::future::Cache;
use std::time::Duration;

pub struct DashboardCache {
    stats: Cache<Uuid, DashboardStatistics>,
}

impl DashboardCache {
    pub fn new() -> Self {
        Self {
            stats: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(30))
                .build(),
        }
    }
    
    pub async fn get_or_fetch<F, Fut>(
        &self,
        user_id: Uuid,
        fetcher: F,
    ) -> Result<DashboardStatistics>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<DashboardStatistics>>,
    {
        self.stats
            .try_get_with(user_id, async move { fetcher().await })
            .await
    }
}
```

#### 3.3 Implement Read Replicas Pattern
For high-traffic scenarios:
- Primary: Write operations
- Replica: Read-only dashboard queries
- Connection pooling: Separate pools for read/write

### Phase 4: Application-Level Optimizations

#### 4.1 Batch Queries Instead of N+1
**Before**:
```rust
for endpoint in endpoints {
    let stats = get_endpoint_stats(endpoint.id).await?; // N queries
}
```

**After**:
```rust
let endpoint_ids: Vec<Uuid> = endpoints.iter().map(|e| e.id).collect();
let stats_map = get_bulk_endpoint_stats(&endpoint_ids).await?; // 1 query
```

#### 4.2 Add Connection Pool Tuning
```rust
// crates/common/src/db.rs
PgPoolOptions::new()
    .max_connections(50)  // Already done
    .min_connections(10)   // ADD: Keep warm connections
    .acquire_timeout(Duration::from_secs(3))  // ADD: Fail fast
    .idle_timeout(Duration::from_secs(600))   // ADD: Reuse connections
    .max_lifetime(Duration::from_secs(1800))  // ADD: Rotate connections
```

#### 4.3 Implement Pagination Cursor
**Before**: OFFSET pagination (slow for large offsets)
```rust
LIMIT $1 OFFSET $2  // Scans and discards OFFSET rows every time
```

**After**: Cursor-based pagination
```rust
WHERE (ingested_at, id) < ($last_time, $last_id)
ORDER BY ingested_at DESC, id DESC
LIMIT $1
```

### Phase 5: Monitoring and Maintenance

#### 5.1 Enable pg_stat_statements
```sql
-- In postgresql.conf
shared_preload_libraries = 'pg_stat_statements'
pg_stat_statements.track = all
pg_stat_statements.max = 10000

-- After restart
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Monitor slow queries
SELECT 
    query,
    calls,
    mean_exec_time,
    total_exec_time,
    stddev_exec_time
FROM pg_stat_statements
WHERE mean_exec_time > 1000  -- >1 second
ORDER BY mean_exec_time DESC
LIMIT 20;
```

#### 5.2 Automated VACUUM and ANALYZE
```sql
-- Already running via autovacuum, but tune for large tables
ALTER TABLE events SET (
    autovacuum_vacuum_scale_factor = 0.05,  -- More frequent
    autovacuum_analyze_scale_factor = 0.02
);

ALTER TABLE delivery_attempts SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02
);
```

#### 5.3 Regular Maintenance Tasks
```bash
# Add to cron (daily at 2 AM)
0 2 * * * docker exec ethhook-postgres psql -U ethhook -d ethhook -c "VACUUM ANALYZE;"

# Weekly full vacuum (Sunday 3 AM)
0 3 * * 0 docker exec ethhook-postgres psql -U ethhook -d ethhook -c "VACUUM FULL ANALYZE events, delivery_attempts;"
```

## Implementation Priority

### Critical (Do Now)
1. ✅ Denormalize user_id to delivery_attempts (IN PROGRESS)
2. ⏱️ Wait for UPDATE to complete
3. 🔴 Configure PostgreSQL parameters (requires restart)
4. 🔴 Drop unused indexes
5. 🔴 Create user_id indexes

### High (This Week)
6. Rewrite top 5 slowest queries using user_id
7. Add result caching layer
8. Enable pg_stat_statements
9. Create user_daily_stats aggregate table

### Medium (Next Week)
10. Implement cursor-based pagination
11. Add batch query operations
12. Create materialized views
13. Setup read replicas (if needed)

### Low (Ongoing)
14. Monitor query performance
15. Regular VACUUM maintenance
16. Index usage analysis
17. Query plan reviews

## Expected Performance Improvements

| Metric | Before | After Phase 1 | After Phase 2-3 |
|--------|--------|---------------|-----------------|
| Dashboard load | 50-300s | 10-30s | <1s |
| Event listing | 71-306s | 5-10s | <500ms |
| Time-series query | 29-276s | 5-15s | <2s |
| Database size | 7GB | 6GB (dropped indexes) | 7.5GB (with aggregates) |
| Memory usage | 7.4GB/7.8GB | 6.5GB/7.8GB | 6GB/7.8GB |
| Query throughput | ~2-3 qps | ~20-30 qps | ~200-500 qps |

## Monitoring Checklist

```sql
-- Daily checks
SELECT * FROM pg_stat_database WHERE datname = 'ethhook';
SELECT * FROM pg_stat_user_tables WHERE schemaname = 'public';
SELECT * FROM pg_stat_user_indexes WHERE idx_scan = 0 AND idx_tup_read > 0;

-- Weekly checks
SELECT * FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 20;
SELECT relname, n_dead_tup, last_autovacuum FROM pg_stat_user_tables;
```

## Risk Mitigation

1. **PostgreSQL restart required**: Schedule during low-traffic window
2. **Long-running UPDATE**: Already executing, monitor progress
3. **Index creation**: Use CONCURRENTLY to avoid blocking
4. **Query rewrites**: Test thoroughly, keep old queries as fallback
5. **Cache invalidation**: Implement proper TTLs and manual invalidation

## Success Metrics

- Dashboard loads in <2 seconds (currently 50-300s)
- Event listing loads in <1 second (currently 71-306s)
- Database CPU usage <50% (currently spiking to 100%)
- Memory usage <6GB (currently 7.4GB)
- Zero query timeouts
- pg_stat_statements shows no queries >5s mean execution time
