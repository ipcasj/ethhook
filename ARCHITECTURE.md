# EthHook Architecture

## Overview

**EthHook** is a high-performance, production-ready webhook service that delivers real-time Ethereum blockchain events to applications. Built entirely in Rust, it provides reliable event ingestion, intelligent routing, and guaranteed delivery with enterprise-grade observability.

### Key Features

1. **High Performance**: Rust's zero-cost abstractions enable sub-500ms event delivery
2. **Open Source**: Core platform is open source and self-hostable  
3. **Developer Friendly**: Clean API design, comprehensive documentation, and SDKs
4. **Reliable**: Built-in retries, deduplication, and guaranteed delivery
5. **Observable**: Prometheus metrics, structured logging, and distributed tracing

---

## System Architecture

### High-Level Architecture

```
                     CLIENT APPLICATIONS
              (dApps, Analytics, NFT Platforms)
                            |
                            | HTTPS Webhooks
                            | (HMAC Signed)
                            v
              ╔═══════════════════════════════╗
              ║      ETHHOOK PLATFORM         ║
              ║                               ║
              ║  [Leptos Portal] <-> [Admin API]  ║
              ║   (WASM SPA)        (Axum REST)   ║
              ║                         |         ║
              ║        DATA LAYER       |         ║
              ║    [PostgreSQL] [Redis]           ║
              ║      (sqlx)   (Streams/Queues)    ║
              ║                                   ║
              ║   EVENT PROCESSING PIPELINE       ║
              ║                                   ║
              ║  [Event     ] -> [Message   ] -> [Webhook ]  ║
              ║  [Ingestor  ]    [Processor]    [Delivery]  ║
              ║  [(ethers-rs)]   [(Fan-out)]    [(reqwest)] ║
              ║        |                                     ║
              ╚════════|═════════════════════════════════════╝
                       | WebSocket / HTTP
                       v
           RPC PROVIDERS (Alchemy Primary + Infura Backup)
           Multi-chain: Ethereum, Arbitrum, Optimism, Base
                       |
                       v
              BLOCKCHAIN NETWORKS (EVM Chains)
```

### Microservices Architecture

#### 1. Event Ingestor Service

**Technology**: Rust + Tokio + ethers-rs/Alloy

**Responsibilities**:

- Maintains persistent WebSocket connections to multiple RPC providers
- Subscribes to `logs` matching user-configured filters
- Handles connection failures with automatic reconnection
- Publishes raw events to Redis Stream (`events:raw`)
- Deduplicates events across multiple RPC providers

**Key Design Decisions**:

- **Multi-Provider Strategy**: Connect to 2-3 RPC providers (Infura, Alchemy, Ankr) simultaneously
- **Event Deduplication**: Use `(block_hash, transaction_hash, log_index)` tuple as unique ID
- **Backfill Support**: On startup/reconnection, fetch missed blocks via `eth_getLogs`
- **Circuit Breaker**: Pause subscriptions if Redis is unavailable

**Performance Targets**:

- <100ms from on-chain event to Redis
- Support 100+ concurrent WebSocket connections
- 99.9% event capture rate

#### 2. Message Processor Service

**Technology**: Rust + Tokio + sqlx + redis-rs + governor

**Responsibilities**:

- Consumes events from `events:raw` Redis Stream
- Validates event structure and enriches with metadata
- Queries PostgreSQL to find all matching Endpoints (fan-out)
- Applies rate limits per endpoint
- Pushes delivery jobs to `webhooks:pending` queue
- Records events in PostgreSQL for history/analytics

**Key Design Decisions**:

- **Fan-out Algorithm**: Use indexed PostgreSQL queries on (contract_address, topics)
- **Rate Limiting**: Token bucket algorithm via `governor` crate
- **Deduplication Window**: 24-hour Redis set to prevent double-processing
- **Batch Processing**: Process up to 100 events per iteration

**Database Queries**:

```sql
-- Find matching endpoints (optimized with indexes)
SELECT e.id, e.url, e.hmac_secret, e.rate_limit
FROM endpoints e
JOIN applications a ON e.application_id = a.id
WHERE e.is_active = true
  AND a.subscription_status = 'active'
  AND e.contract_address = $1
  AND ($2 = ANY(e.event_topics) OR e.event_topics IS NULL)
ORDER BY e.priority DESC;
```

#### 3. Webhook Delivery Service

**Technology**: Rust + Tokio + reqwest + redis-rs

**Responsibilities**:

- Consumes delivery jobs from `webhooks:pending` queue
- Makes HTTP POST requests to customer endpoints
- Implements HMAC-SHA256 signature for authentication
- Handles retries with exponential backoff
- Records delivery attempts in PostgreSQL
- Emits metrics for monitoring

**Delivery Protocol**:

```http
POST https://customer-app.com/webhook HTTP/1.1
Content-Type: application/json
X-EthHook-Signature: sha256=abc123...
X-EthHook-Event-ID: evt_01HG2K3M4N5P6Q7R8S9T
X-EthHook-Delivery-Attempt: 1
X-EthHook-Timestamp: 1696291200

{
  "id": "evt_01HG2K3M4N5P6Q7R8S9T",
  "type": "ethereum.log",
  "created_at": "2025-10-02T12:00:00Z",
  "data": {
    "block_number": 18500000,
    "transaction_hash": "0xabc...",
    "contract_address": "0x123...",
    "topics": ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
    "data": "0x000..."
  }
}
```

**Retry Strategy**:

- **Attempt 1**: Immediate
- **Attempt 2**: 5 seconds later
- **Attempt 3**: 30 seconds later
- **Attempt 4**: 5 minutes later
- **Attempt 5**: 1 hour later
- **Final Failure**: Mark as failed, notify customer

**Signature Verification (Customer Side)**:

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_webhook(secret: &[u8], signature: &str, payload: &str) -> bool {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    mac.update(payload.as_bytes());
    let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
    expected == signature
}
```

#### 4. Admin API Service

**Technology**: Rust + Axum + sqlx + jsonwebtoken + validator

**API Endpoints**:

Authentication:
POST   /api/v1/auth/register        - User registration
POST   /api/v1/auth/login           - Login (returns JWT)
POST   /api/v1/auth/refresh         - Refresh JWT token
POST   /api/v1/auth/logout          - Logout

Applications:
GET    /api/v1/applications         - List all applications
POST   /api/v1/applications         - Create new application
GET    /api/v1/applications/{id}    - Get application details
PATCH  /api/v1/applications/{id}    - Update application
DELETE /api/v1/applications/{id}    - Delete application

Endpoints:
GET    /api/v1/applications/{app_id}/endpoints       - List endpoints
POST   /api/v1/applications/{app_id}/endpoints       - Create endpoint
GET    /api/v1/endpoints/{id}                        - Get endpoint
PATCH  /api/v1/endpoints/{id}                        - Update endpoint
DELETE /api/v1/endpoints/{id}                        - Delete endpoint
POST   /api/v1/endpoints/{id}/test                   - Send test webhook

Events:
GET    /api/v1/events               - List delivered events (paginated)
GET    /api/v1/events/{id}          - Get event details
GET    /api/v1/events/{id}/attempts - Get delivery attempts

Account:
GET    /api/v1/account              - Get account info
PATCH  /api/v1/account              - Update account
GET    /api/v1/account/usage        - Get usage statistics
GET    /api/v1/account/billing      - Get billing info

**Security Measures**:

1. **JWT Authentication**: RS256 with 15-minute access tokens, 7-day refresh tokens
2. **Rate Limiting**: 100 req/min per user, 1000 req/min global
3. **Input Validation**: All inputs validated with `validator` crate
4. **SQL Injection Prevention**: Parameterized queries via `sqlx`
5. **CORS**: Configured for frontend domain only
6. **Audit Logging**: All mutations logged to audit table

#### 5. Data Layer

**PostgreSQL Schema** (see detailed schema below)
**Redis Usage**:

- **Stream**: `events:raw` - Raw blockchain events
- **Queue**: `webhooks:pending` - Webhook delivery jobs
- **Set**: `dedup:{date}` - Event deduplication (24h TTL)
- **Hash**: `ratelimit:{endpoint_id}` - Rate limit state
- **Cache**: `endpoint:{id}` - Endpoint configuration (5min TTL)

#### 6. Leptos Portal (Frontend)

**Technology**: Rust + Leptos + WASM + TailwindCSS

**Pages**:

- `/login` - Authentication
- `/dashboard` - Overview, usage stats, recent events
- `/applications` - Manage applications
- `/applications/{id}/endpoints` - Manage endpoints
- `/events` - Event history with filtering
- `/settings` - Account settings, API keys
- `/billing` - Subscription management

---

## Database Schema

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table (multi-tenant)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    email_verified BOOLEAN DEFAULT false,
    subscription_tier VARCHAR(50) DEFAULT 'free', -- free, starter, pro, enterprise
    subscription_status VARCHAR(50) DEFAULT 'active', -- active, suspended, cancelled
    stripe_customer_id VARCHAR(255),
    api_key_hash VARCHAR(255) UNIQUE, -- For API authentication
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
    webhook_secret VARCHAR(64) NOT NULL, -- For HMAC signing
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
    
    -- Event filtering
    contract_address VARCHAR(42), -- 0x prefixed address
    event_topics TEXT[], -- Array of topic hashes (first is event signature)
    
    -- Delivery settings
    rate_limit_per_second INTEGER DEFAULT 10,
    max_retries INTEGER DEFAULT 5,
    timeout_seconds INTEGER DEFAULT 30,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    health_status VARCHAR(50) DEFAULT 'healthy', -- healthy, degraded, failed
    last_successful_delivery_at TIMESTAMPTZ,
    consecutive_failures INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_endpoints_application_id ON endpoints(application_id);
CREATE INDEX idx_endpoints_contract_address ON endpoints(contract_address) WHERE is_active = true;
CREATE INDEX idx_endpoints_event_topics ON endpoints USING GIN(event_topics) WHERE is_active = true;

-- Events (blockchain events we've processed)
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Blockchain data
    block_number BIGINT NOT NULL,
    block_hash VARCHAR(66) NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    log_index INTEGER NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    topics TEXT[] NOT NULL,
    data TEXT NOT NULL,
    
    -- Metadata
    ingested_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    
    -- Unique constraint to prevent duplicates
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX idx_events_block_number ON events(block_number DESC);
CREATE INDEX idx_events_contract_address ON events(contract_address);
CREATE INDEX idx_events_transaction_hash ON events(transaction_hash);
CREATE INDEX idx_events_ingested_at ON events(ingested_at DESC);

-- Delivery attempts (webhook delivery records)
CREATE TABLE delivery_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    endpoint_id UUID NOT NULL REFERENCES endpoints(id) ON DELETE CASCADE,
    
    -- Delivery details
    attempt_number INTEGER NOT NULL,
    http_status_code INTEGER,
    response_body TEXT,
    error_message TEXT,
    
    -- Timing
    attempted_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- Outcome
    success BOOLEAN,
    should_retry BOOLEAN DEFAULT false,
    next_retry_at TIMESTAMPTZ
);

CREATE INDEX idx_delivery_attempts_event_id ON delivery_attempts(event_id);
CREATE INDEX idx_delivery_attempts_endpoint_id ON delivery_attempts(endpoint_id);
CREATE INDEX idx_delivery_attempts_next_retry_at ON delivery_attempts(next_retry_at) WHERE should_retry = true;
CREATE INDEX idx_delivery_attempts_attempted_at ON delivery_attempts(attempted_at DESC);

-- Usage tracking (for billing)
CREATE TABLE usage_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    month DATE NOT NULL, -- First day of the month
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
    action VARCHAR(255) NOT NULL, -- e.g., "endpoint.created", "application.deleted"
    resource_type VARCHAR(100),
    resource_id UUID,
    metadata JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Subscription limits (cached from Stripe or internal)
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
```

---

## Technology Stack

### Core Services (Rust)

- **Async Runtime**: `tokio` 1.35+ with multi-threaded scheduler
- **HTTP Client**: `reqwest` with connection pooling
- **HTTP Server**: `axum` 0.7+ for REST API
- **Blockchain**: `ethers-rs` 2.0+ or `alloy` (next-gen)
- **Database**: `sqlx` with compile-time query verification
- **Redis**: `redis` with async support
- **Serialization**: `serde` + `serde_json`
- **Authentication**: `jsonwebtoken` for JWT
- **Validation**: `validator` for input validation
- **Rate Limiting**: `governor` for token bucket
- **Observability**: `tracing`, `tracing-subscriber`, `opentelemetry`
- **Metrics**: `prometheus` client
- **Configuration**: `config` + environment variables
- **Testing**: `tokio-test`, `mockito`, `testcontainers`

### Frontend (Rust WASM)

- **Framework**: `leptos` 0.5+ (full-stack Rust)
- **Styling**: TailwindCSS
- **HTTP Client**: `reqwasm` or `gloo-net`
- **State Management**: Leptos reactive system
- **Routing**: `leptos_router`

### Infrastructure

- **Containerization**: Docker + Docker Compose
- **Orchestration**: Kubernetes (optional) or Docker Swarm
- **Hosting**: DigitalOcean App Platform / Droplets
- **Database**: Managed PostgreSQL (DigitalOcean)
- **Cache**: Managed Redis (DigitalOcean)
- **RPC Providers**: Infura, Alchemy, Ankr (multi-provider)
- **CDN**: Cloudflare for frontend
- **Monitoring**: Prometheus + Grafana
- **Logging**: Loki or CloudWatch
- **Alerting**: Grafana Alerts / PagerDuty

---

## Deployment Architecture (DigitalOcean)

### Recommended Setup

### Option A: App Platform (Easiest)

```
╔═══════════════════════════════════════════╗
║      DigitalOcean App Platform            ║
║                                           ║
║  [Event Ingestor]  [Message Processor]    ║
║    (Worker)           (Worker)            ║
║                                           ║
║  [Webhook Delivery]  [Admin API]          ║
║    (Worker)          (Web Service)        ║
║                                           ║
║  [Leptos Portal]  [Managed PostgreSQL]    ║
║    (Static)                               ║
║                                           ║
║                   [Managed Redis]         ║
║                                           ║
╚═══════════════════════════════════════════╝
```

### Option B: Kubernetes (Scalable)

```
╔═══════════════════════════════════════════╗
║   DigitalOcean Kubernetes Cluster         ║
║                                           ║
║          [Ingress (Nginx)]                ║
║                 |                         ║
║    ┌────────────┴────────────────┐        ║
║    v                             v        ║
║  [Admin API]         [Leptos Portal]      ║
║  (3 pods)            (Cloudflare CDN)     ║
║                                           ║
║  [Event Ingestor]    [Message Processor]  ║
║  (2 pods)            (5 pods)             ║
║                                           ║
║  [Webhook Delivery]                       ║
║  (10 pods)                                ║
║                                           ║
╚═══════════════════════════════════════════╝
        |                        |
        v                        v
  [Managed PostgreSQL]    [Managed Redis]
```

---

## Security

### API Security

1. **Authentication**
   - Rate limiting: 100 req/min per API key
   - JWT rotation: 15-min access tokens
   - API key hashing: bcrypt with salt
   - HTTPS only (TLS 1.3)

2. **Webhook Security**
   - HMAC-SHA256 signatures
   - Timestamp validation (±5 minutes)
   - Idempotency keys
   - Webhook URL validation (no localhost)

3. **Data Protection**
   - Encrypt secrets at rest (PostgreSQL TDE)
   - Rotate HMAC secrets regularly
   - PII encryption for user data
   - Regular backups (daily)

4. **Infrastructure**
   - VPC isolation
   - Firewall rules (least privilege)
   - DDoS protection (Cloudflare)
   - Security scanning (Snyk, Dependabot)

---

## Monitoring & Alerting

### Key Metrics

**Application Metrics**:

- `ethhook_events_ingested_total` - Counter of events from blockchain
- `ethhook_events_processed_total` - Counter of events processed
- `ethhook_webhooks_sent_total{status="success|failure"}` - Delivery outcomes
- `ethhook_webhook_delivery_latency_seconds` - Histogram of delivery times
- `ethhook_webhook_retry_total` - Counter of retry attempts
- `ethhook_active_subscriptions` - Gauge of active endpoints

**System Metrics**:

- CPU/Memory usage per service
- PostgreSQL connection pool stats
- Redis queue depth
- WebSocket connection count

### Alerts

| Alert | Condition | Severity |
|-------|-----------|----------|
| High delivery failure rate | >5% in 5min | Warning |
| RPC connection loss | Any provider down >1min | Critical |
| Database connection pool exhausted | >90% utilization | Warning |
| Redis queue backup | >1000 pending jobs | Warning |
| Service crash loop | >3 restarts in 10min | Critical |

---

## API Usage Examples

### Creating an Endpoint (cURL)

```bash
curl -X POST https://api.ethhook.io/api/v1/applications/550e8400/endpoints \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "USDC Transfer Monitor",
    "url": "https://myapp.com/webhooks/usdc",
    "contract_address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    "event_topics": [
      "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
    ],
    "rate_limit_per_second": 10
  }'
```

### Response

```json
{
  "id": "ep_7c9e6679",
  "name": "USDC Transfer Monitor",
  "url": "https://myapp.com/webhooks/usdc",
  "contract_address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
  "hmac_secret": "whsec_a1b2c3d4e5f6...",
  "is_active": true,
  "health_status": "healthy",
  "created_at": "2025-10-02T12:00:00Z"
}
```

---

## Resources

- [Ethereum JSON-RPC Specification](https://ethereum.github.io/execution-apis/api-documentation/)
- [Tokio Documentation](https://tokio.rs/)
- [Axum Web Framework](https://github.com/tokio-rs/axum)
