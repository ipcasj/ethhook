-- SQLite-compatible schema
-- Note: SQLite stores UUIDs as TEXT, dates as TEXT in ISO8601 format

-- Users table (multi-tenant)
CREATE TABLE users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name TEXT,
    email_verified INTEGER DEFAULT 0,
    subscription_tier TEXT DEFAULT 'free',
    subscription_status TEXT DEFAULT 'active',
    stripe_customer_id TEXT,
    api_key_hash TEXT UNIQUE,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    last_login_at TEXT
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_api_key_hash ON users(api_key_hash);

-- Applications (projects/workspaces)
CREATE TABLE applications (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    webhook_secret TEXT NOT NULL,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_applications_user_id ON applications(user_id);

-- Endpoints (webhook URLs)
CREATE TABLE endpoints (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    hmac_secret TEXT NOT NULL,
    contract_address TEXT,
    event_topics TEXT,
    rate_limit_per_second INTEGER DEFAULT 10,
    max_retries INTEGER DEFAULT 5,
    timeout_seconds INTEGER DEFAULT 30,
    is_active INTEGER DEFAULT 1,
    health_status TEXT DEFAULT 'healthy',
    last_successful_delivery_at TEXT,
    consecutive_failures INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_endpoints_application_id ON endpoints(application_id);
CREATE INDEX idx_endpoints_contract_address ON endpoints(contract_address) WHERE is_active = 1;
CREATE INDEX idx_endpoints_event_topics ON endpoints(event_topics) WHERE is_active = 1;

-- Events (blockchain events)
CREATE TABLE events (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    block_number INTEGER NOT NULL,
    block_hash TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    contract_address TEXT NOT NULL,
    topics TEXT NOT NULL,
    data TEXT NOT NULL,
    ingested_at TEXT DEFAULT (datetime('now')),
    processed_at TEXT,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_events_block_number ON events(block_number DESC);
CREATE INDEX idx_events_contract_address ON events(contract_address);
CREATE INDEX idx_events_transaction_hash ON events(transaction_hash);
CREATE INDEX idx_events_ingested_at ON events(ingested_at DESC);

-- Delivery attempts
CREATE TABLE delivery_attempts (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    event_id TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    endpoint_id TEXT NOT NULL REFERENCES endpoints(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    http_status_code INTEGER,
    response_body TEXT,
    error_message TEXT,
    attempted_at TEXT DEFAULT (datetime('now')),
    completed_at TEXT,
    duration_ms INTEGER,
    success INTEGER,
    should_retry INTEGER DEFAULT 0,
    next_retry_at TEXT
);

CREATE INDEX idx_delivery_attempts_event_id ON delivery_attempts(event_id);
CREATE INDEX idx_delivery_attempts_endpoint_id ON delivery_attempts(endpoint_id);
CREATE INDEX idx_delivery_attempts_next_retry_at ON delivery_attempts(next_retry_at) WHERE should_retry = 1;
CREATE INDEX idx_delivery_attempts_attempted_at ON delivery_attempts(attempted_at DESC);

-- Usage tracking
CREATE TABLE usage_records (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    month TEXT NOT NULL,
    events_delivered INTEGER DEFAULT 0,
    webhooks_sent INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(user_id, month)
);

CREATE INDEX idx_usage_records_user_month ON usage_records(user_id, month);

-- Audit log
CREATE TABLE audit_logs (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    metadata TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Subscription limits
CREATE TABLE subscription_limits (
    tier TEXT PRIMARY KEY,
    max_events_per_month INTEGER,
    max_applications INTEGER,
    max_endpoints_per_application INTEGER,
    max_requests_per_minute INTEGER,
    support_level TEXT,
    price_usd REAL
);

INSERT INTO subscription_limits VALUES
('free', 10000, 1, 5, 60, 'community', 0.00),
('starter', 100000, 5, 20, 300, 'email', 9.00),
('pro', 1000000, 20, 100, 1000, 'priority', 49.00),
('enterprise', -1, -1, -1, -1, 'dedicated', 499.00);
