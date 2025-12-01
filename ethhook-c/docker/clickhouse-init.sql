-- ClickHouse Schema for EthHook
-- Optimized for time-series event storage and analytics

-- Events table (main event storage)
CREATE TABLE IF NOT EXISTS events (
    id UUID,
    endpoint_id UUID,
    application_id UUID,
    chain_id UInt64,
    block_number UInt64,
    block_hash String,
    transaction_hash String,
    log_index UInt32,
    contract_address String,
    topics Array(String),
    data String,
    ingested_at DateTime64(3),
    processed_at DateTime64(3)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(ingested_at)
ORDER BY (chain_id, block_number, log_index)
TTL ingested_at + INTERVAL 90 DAY
SETTINGS 
    index_granularity = 8192,
    compress_marks = 1,
    compress_primary_key = 1;

-- Deliveries table (webhook delivery tracking)
CREATE TABLE IF NOT EXISTS deliveries (
    id UUID,
    event_id UUID,
    endpoint_id UUID,
    url String,
    status String,
    attempt_count UInt32,
    http_status_code Int32,
    error_message Nullable(String),
    delivered_at DateTime64(3),
    next_retry_at DateTime64(3)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(delivered_at)
ORDER BY (endpoint_id, delivered_at)
TTL delivered_at + INTERVAL 90 DAY
SETTINGS 
    index_granularity = 8192,
    compress_marks = 1,
    compress_primary_key = 1;

-- Indexes for common queries
-- These are implicit in the ORDER BY, but documented here for clarity:
-- events: Fast queries by (chain_id, block_number, log_index)
-- deliveries: Fast queries by (endpoint_id, delivered_at)

-- Create user and grant permissions
CREATE USER IF NOT EXISTS ethhook IDENTIFIED WITH sha256_password BY '${CLICKHOUSE_PASSWORD}';
GRANT SELECT, INSERT, CREATE TABLE ON ethhook.* TO ethhook;

-- Materialized views for analytics (optional, can be added later)
-- Example: Event count by chain
-- CREATE MATERIALIZED VIEW IF NOT EXISTS events_by_chain
-- ENGINE = SummingMergeTree()
-- PARTITION BY toYYYYMM(ingested_at)
-- ORDER BY (chain_id, ingested_at)
-- AS SELECT
--     chain_id,
--     toStartOfHour(ingested_at) as ingested_at,
--     count() as event_count
-- FROM events
-- GROUP BY chain_id, toStartOfHour(ingested_at);
