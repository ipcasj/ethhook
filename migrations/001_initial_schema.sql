-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table (multi-tenant)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    email_verified BOOLEAN DEFAULT false,
    subscription_tier VARCHAR(50) DEFAULT 'free',
    subscription_status VARCHAR(50) DEFAULT 'active',
    stripe_customer_id VARCHAR(255),
    api_key_hash VARCHAR(255) UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_api_key_hash ON users(api_key_hash);

-- Applications (projects/workspaces)
CREATE TABLE applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    webhook_secret VARCHAR(64) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_applications_user_id ON applications(user_id);

-- Endpoints (webhook URLs)
CREATE TABLE endpoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    hmac_secret VARCHAR(64) NOT NULL,
    contract_address VARCHAR(42),
    event_topics TEXT[],
    rate_limit_per_second INTEGER DEFAULT 10,
    max_retries INTEGER DEFAULT 5,
    timeout_seconds INTEGER DEFAULT 30,
    is_active BOOLEAN DEFAULT true,
    health_status VARCHAR(50) DEFAULT 'healthy',
    last_successful_delivery_at TIMESTAMPTZ,
    consecutive_failures INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_endpoints_application_id ON endpoints(application_id);
CREATE INDEX idx_endpoints_contract_address ON endpoints(contract_address) WHERE is_active = true;
CREATE INDEX idx_endpoints_event_topics ON endpoints USING GIN(event_topics) WHERE is_active = true;

-- Events (blockchain events)
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    block_number BIGINT NOT NULL,
    block_hash VARCHAR(66) NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    log_index INTEGER NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    topics TEXT[] NOT NULL,
    data TEXT NOT NULL,
    ingested_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_events_block_number ON events(block_number DESC);
CREATE INDEX idx_events_contract_address ON events(contract_address);
CREATE INDEX idx_events_transaction_hash ON events(transaction_hash);
CREATE INDEX idx_events_ingested_at ON events(ingested_at DESC);

-- Delivery attempts
CREATE TABLE delivery_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    endpoint_id UUID NOT NULL REFERENCES endpoints(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    http_status_code INTEGER,
    response_body TEXT,
    error_message TEXT,
    attempted_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    success BOOLEAN,
    should_retry BOOLEAN DEFAULT false,
    next_retry_at TIMESTAMPTZ
);

CREATE INDEX idx_delivery_attempts_event_id ON delivery_attempts(event_id);
CREATE INDEX idx_delivery_attempts_endpoint_id ON delivery_attempts(endpoint_id);
CREATE INDEX idx_delivery_attempts_next_retry_at ON delivery_attempts(next_retry_at) WHERE should_retry = true;
CREATE INDEX idx_delivery_attempts_attempted_at ON delivery_attempts(attempted_at DESC);

-- Usage tracking
CREATE TABLE usage_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    month DATE NOT NULL,
    events_delivered INTEGER DEFAULT 0,
    webhooks_sent INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, month)
);

CREATE INDEX idx_usage_records_user_month ON usage_records(user_id, month);

-- Audit log
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    metadata JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Subscription limits
CREATE TABLE subscription_limits (
    tier VARCHAR(50) PRIMARY KEY,
    max_events_per_month INTEGER,
    max_applications INTEGER,
    max_endpoints_per_application INTEGER,
    max_requests_per_minute INTEGER,
    support_level VARCHAR(50),
    price_usd DECIMAL(10, 2)
);

INSERT INTO subscription_limits VALUES
('free', 10000, 1, 5, 60, 'community', 0.00),
('starter', 100000, 5, 20, 300, 'email', 9.00),
('pro', 1000000, 20, 100, 1000, 'priority', 49.00),
('enterprise', -1, -1, -1, -1, 'dedicated', 499.00);
