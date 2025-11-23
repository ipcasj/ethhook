-- ClickHouse schema for blockchain events and webhook deliveries
-- Optimized for time-series data and high-volume writes

-- Events table: stores all blockchain events
CREATE TABLE IF NOT EXISTS events (
    id UUID DEFAULT generateUUIDv4(),
    endpoint_id UUID,
    application_id UUID,
    user_id UUID,
    chain_id UInt32,
    block_number UInt64,
    block_hash String,
    transaction_hash String,
    log_index UInt32,
    contract_address String,
    topics Array(String),
    data String,
    ingested_at DateTime64(3) DEFAULT now64(),
    processed_at DateTime64(3)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(ingested_at)
ORDER BY (user_id, endpoint_id, ingested_at, id)
TTL ingested_at + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- Delivery attempts table: tracks webhook delivery attempts
CREATE TABLE IF NOT EXISTS delivery_attempts (
    id UUID DEFAULT generateUUIDv4(),
    event_id UUID,
    endpoint_id UUID,
    application_id UUID,
    user_id UUID,
    attempt_number UInt8,
    status String, -- 'success', 'failed', 'pending'
    http_status UInt16,
    response_body String,
    error_message String,
    attempted_at DateTime64(3) DEFAULT now64(),
    duration_ms UInt32,
    webhook_url String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(attempted_at)
ORDER BY (user_id, endpoint_id, attempted_at, id)
TTL attempted_at + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- Materialized view for event statistics (per endpoint)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_endpoint_stats
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (endpoint_id, date)
AS SELECT
    endpoint_id,
    toDate(ingested_at) as date,
    count() as event_count,
    uniqExact(transaction_hash) as unique_transactions,
    uniqExact(contract_address) as unique_contracts
FROM events
GROUP BY endpoint_id, date;

-- Materialized view for delivery statistics (per endpoint)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_delivery_stats
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (endpoint_id, date, status)
AS SELECT
    endpoint_id,
    toDate(attempted_at) as date,
    status,
    count() as attempt_count,
    avg(duration_ms) as avg_duration_ms,
    max(duration_ms) as max_duration_ms,
    min(duration_ms) as min_duration_ms
FROM delivery_attempts
GROUP BY endpoint_id, date, status;

-- Create indexes for common queries
-- Events indexes
CREATE INDEX IF NOT EXISTS idx_events_endpoint ON events(endpoint_id) TYPE bloom_filter GRANULARITY 1;
CREATE INDEX IF NOT EXISTS idx_events_application ON events(application_id) TYPE bloom_filter GRANULARITY 1;
CREATE INDEX IF NOT EXISTS idx_events_chain ON events(chain_id) TYPE minmax GRANULARITY 4;
CREATE INDEX IF NOT EXISTS idx_events_block ON events(block_number) TYPE minmax GRANULARITY 4;
CREATE INDEX IF NOT EXISTS idx_events_tx ON events(transaction_hash) TYPE bloom_filter GRANULARITY 1;

-- Delivery attempts indexes
CREATE INDEX IF NOT EXISTS idx_delivery_endpoint ON delivery_attempts(endpoint_id) TYPE bloom_filter GRANULARITY 1;
CREATE INDEX IF NOT EXISTS idx_delivery_event ON delivery_attempts(event_id) TYPE bloom_filter GRANULARITY 1;
CREATE INDEX IF NOT EXISTS idx_delivery_status ON delivery_attempts(status) TYPE set(10) GRANULARITY 4;
