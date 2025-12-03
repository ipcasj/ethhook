# EthHook C Architecture

## System Design

### Overview

EthHook C is a microservices-based system for capturing Ethereum blockchain events and delivering them as webhooks to registered endpoints. The architecture prioritizes performance, reliability, and maintainability.

### Core Components

#### 1. Event Ingestor

**Purpose**: Connect to Ethereum nodes via WebSocket and ingest real-time events.

**Threading Model**:

- One worker thread per blockchain chain
- Each thread runs its own libevent event loop
- Non-blocking I/O throughout

**Data Flow**:

```text
WebSocket Connection
    ↓ (libwebsockets callbacks)
JSON Event Parsing
    ↓ (jansson)
Arena Allocation
    ↓
Redis Publish (XADD)
    ↓
Redis Streams
```

**Resilience**:

- Circuit breaker per connection (5 failures → 30s timeout)
- Exponential backoff for reconnection
- Graceful degradation on node failure

#### 2. Message Processor

**Purpose**: Match incoming events against registered endpoints and queue deliveries.

**Threading Model**:

- Main thread: Redis consumer (XREAD)
- Worker pool: Endpoint matching and filtering
- libevent for async Redis operations

**Matching Algorithm**:

1. Query endpoints by chain_id (SQLite B-tree index)
2. Filter by contract address (exact match, case-insensitive)
3. Filter by topics (array matching with wildcards)
4. Generate delivery requests for matches

**Database Schema** (SQLite):

```sql
CREATE TABLE endpoints (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    address TEXT,
    topics JSON,
    enabled INTEGER DEFAULT 1,
    FOREIGN KEY (application_id) REFERENCES applications(id)
);

CREATE INDEX idx_endpoints_chain ON endpoints(chain_id) WHERE enabled = 1;
```

#### 3. Webhook Delivery

**Purpose**: Deliver event notifications to webhook URLs with retry logic.

**Threading Model**:

- Worker pool (default 8 threads)
- Each worker: libcurl for HTTP POST
- Event loop: Redis consumer + timer events

**Retry Policy**:

- Base delay: 1 second
- Max delay: 60 seconds
- Backoff multiplier: 2.0
- Jitter: ±25%
- Max retries: 5

**Formula**:

```text
delay = min(base_delay * (multiplier ^ attempt), max_delay)
delay = delay * (1.0 + jitter)
```

**Circuit Breaker**:

- Per-endpoint circuit breaker
- Threshold: 5 consecutive failures
- Timeout: 30 seconds
- Half-open: 3 test requests

**Webhook Format**:

```http
POST /webhook HTTP/1.1
Host: your-app.com
Content-Type: application/json
X-EthHook-Signature: sha256=<hmac>

{
  "event_id": "uuid",
  "chain_id": 1,
  "block_number": 12345678,
  "transaction_hash": "0x...",
  "contract_address": "0x...",
  "topics": ["0x..."],
  "data": "0x...",
  "timestamp": 1234567890
}
```

#### 4. Admin API

**Purpose**: REST API for managing users, applications, endpoints.

**Threading Model**:

- libmicrohttpd for HTTP server
- MHD_USE_THREAD_PER_CONNECTION
- One thread per HTTP connection
- SQLite with WAL mode for concurrency

**Authentication**:

- JWT tokens (HS256)
- Authorization header: `Bearer <token>`
- Claims: user_id, is_admin, exp, iat

**Endpoints**:

```text
POST /api/auth/login
GET  /api/users (admin only)
GET  /api/applications
POST /api/applications
GET  /api/endpoints
POST /api/endpoints
GET  /api/events
GET  /api/deliveries
```

## Technical Details

### Memory Management

**Arena Allocators**:

```c
typedef struct arena_block {
    size_t size;
    size_t used;
    struct arena_block *next;
    char data[];
} arena_block_t;

struct eth_arena {
    arena_block_t *current;
    size_t default_block_size;
    pthread_mutex_t lock;
};
```

- **Fast Allocation**: Bump allocator, O(1) average
- **Thread-Safe**: Mutex-protected
- **Bulk Free**: Reset/destroy entire arena
- **Use Case**: Per-request allocation in event processing

### Circuit Breaker

**State Machine**:

```text
        +--------+
        | CLOSED | <-- Normal operation
        +--------+
            |
            | (failures >= threshold)
            v
        +------+
        | OPEN | <-- Reject requests
        +------+
            |
            | (timeout elapsed)
            v
        +-----------+
        | HALF_OPEN | <-- Test recovery
        +-----------+
            |
            +-> (success) --> CLOSED
            +-> (failure) --> OPEN
```

**Implementation**:

```c
typedef struct {
    atomic_int state;
    atomic_uint_fast64_t failure_count;
    atomic_uint_fast64_t success_count;
    atomic_uint_fast64_t last_failure_time;
    uint32_t failure_threshold;
    uint32_t timeout_ms;
} circuit_breaker_t;
```

### Concurrency

**Atomics** (C11 stdatomic.h):

```c
// Metrics without locks
atomic_uint_fast64_t events_received;
atomic_fetch_add(&events_received, 1);
```

**Mutexes**:

```c
// Arena allocator
pthread_mutex_lock(&arena->lock);
void *ptr = allocate_from_block(arena->current, size);
pthread_mutex_unlock(&arena->lock);
```

**Event Loops** (libevent):

```c
struct event_base *base = event_base_new();
event_base_dispatch(base); // Run until stopped
event_base_loopbreak(base); // Stop from signal handler
```

### Error Handling

**Error Propagation**:

```c
eth_error_t function() {
    if (error_condition) {
        LOG_ERROR("Operation failed: %s", reason);
        return ETH_ERROR_NETWORK;
    }
    return ETH_OK;
}

// Caller
eth_error_t err = function();
if (err != ETH_OK) {
    // Handle error
}
```

**Logging**:

```c
LOG_DEBUG("Verbose information");
LOG_INFO("Normal operation");
LOG_WARN("Unexpected but recoverable");
LOG_ERROR("Error occurred");
```

- Logs to syslog + stderr
- Structured format: `[timestamp] LEVEL: message`
- Context: file, line, function (via macros)

## Configuration

### Service Configuration

**Ingestor**:

```toml
[ingestor]
worker_threads = 4
reconnect_delay_ms = 5000
max_reconnect_attempts = 10
```

**Processor**:

```toml
[processor]
worker_threads = 4
batch_size = 100
```

**Delivery**:

```toml
[delivery]
worker_threads = 8
max_retries = 5
timeout_ms = 30000
```

**Admin API**:

```toml
[admin_api]
port = 3000
jwt_secret = "your-secret"
jwt_expiry_hours = 24
```

## Performance Characteristics

### Benchmarks

**Event Ingestion**:

- Throughput: ~10,000 events/sec
- Latency (p50): <1ms
- Latency (p99): <5ms
- CPU: ~10% per chain

**Event Processing**:

- Throughput: ~5,000 matches/sec
- Latency (p50): <2ms
- Latency (p99): <10ms
- CPU: ~20% (4 workers)

**Webhook Delivery**:

- Throughput: ~1,000 deliveries/sec
- Latency (p50): ~50ms (network dependent)
- CPU: ~30% (8 workers)

### Resource Usage

**Memory**:

- Base: ~5MB per service
- Per-event: ~1KB (arena allocated)
- SQLite: ~10MB for 1M endpoints

**Disk**:

- Binary size: ~2MB per service
- SQLite: ~100KB + data
- Redis: ~1KB per event in stream

## Scalability

### Horizontal Scaling

**Event Ingestor**:
- Multiple instances with different chains
- No coordination needed
- Redis streams handle fanout

**Message Processor**:
- Multiple instances with consumer groups
- Redis XREADGROUP for load balancing
- SQLite read-only replicas

**Webhook Delivery**:
- Multiple instances with consumer groups
- Independent worker pools
- Circuit breakers per-endpoint

**Admin API**:
- Multiple instances behind load balancer
- Stateless (JWT)
- SQLite with WAL mode for writes

### Vertical Scaling

**CPU**:
- Increase worker threads
- One thread per chain (ingestor)
- Worker pool size (processor, delivery)

**Memory**:
- Larger arena allocators
- Increase Redis maxmemory
- SQLite cache size

**Network**:
- More WebSocket connections
- HTTP connection pooling
- Redis pipelining

## Security

### Threat Model

**Attack Vectors**:
1. Unauthorized API access
2. Webhook URL manipulation
3. SQL injection
4. DoS via event flooding
5. Man-in-the-middle

**Mitigations**:
1. JWT authentication, RBAC
2. HMAC signatures, URL validation
3. Parameterized queries, input validation
4. Circuit breakers, rate limiting
5. HTTPS/TLS, certificate validation

### Best Practices

- Never log secrets
- Rotate JWT secret regularly
- Use environment variables for config
- Run as non-root user
- Apply resource limits (ulimit, Docker)
- Enable audit logging
- Monitor failed authentication attempts

## Deployment

### Docker

**Image Sizes**:
- ethhook-ingestor: ~30MB
- ethhook-processor: ~25MB
- ethhook-delivery: ~25MB
- ethhook-admin-api: ~25MB

**Resource Limits**:
```yaml
services:
  ingestor:
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 256M
        reservations:
          cpus: '0.5'
          memory: 128M
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ethhook-ingestor
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ethhook-ingestor
  template:
    metadata:
      labels:
        app: ethhook-ingestor
    spec:
      containers:
      - name: ingestor
        image: ethhook-ingestor:latest
        resources:
          limits:
            cpu: 1
            memory: 256Mi
          requests:
            cpu: 500m
            memory: 128Mi
```

## Monitoring

### Metrics

**Ingestor**:
- events_received (counter)
- events_published (counter)
- errors (counter)
- circuit_breaker_state (gauge)

**Processor**:
- events_processed (counter)
- matches_found (counter)
- processing_latency (histogram)

**Delivery**:
- deliveries_attempted (counter)
- deliveries_succeeded (counter)
- deliveries_failed (counter)
- retry_count (histogram)

**Admin API**:
- requests_total (counter)
- request_duration (histogram)
- active_connections (gauge)

### Logging

**Structured Logs**:
```
[2025-01-29 12:34:56] INFO: Starting EthHook Event Ingestor
[2025-01-29 12:34:57] INFO: WebSocket connection established for chain 1
[2025-01-29 12:34:58] INFO: Received event for chain 1: {...}
[2025-01-29 12:35:00] WARN: Circuit breaker open for chain 42161, skipping connection
[2025-01-29 12:35:05] ERROR: Failed to connect WebSocket for chain 10: connection refused
```

**Log Aggregation**:
- syslog to centralized server
- Docker logs via logging driver
- Parse with Fluentd/Logstash
- Analyze with Elasticsearch/Grafana

## Future Enhancements

1. **Metrics Export**: Prometheus/StatsD exporter
2. **Distributed Tracing**: OpenTelemetry integration
3. **Advanced Filtering**: CEL (Common Expression Language)
4. **Event Replay**: Read from historical Redis streams
5. **Multi-Region**: Geographic load balancing
6. **Managed Service**: SaaS deployment option

## References

- [libevent Documentation](https://libevent.org/doc/)
- [libwebsockets API](https://libwebsockets.org/lws-api-doc-main/html/)
- [hiredis GitHub](https://github.com/redis/hiredis)
- [jansson Documentation](https://jansson.readthedocs.io/)
- [libcurl Tutorial](https://curl.se/libcurl/c/libcurl-tutorial.html)
- [libmicrohttpd Manual](https://www.gnu.org/software/libmicrohttpd/manual/)
- [SQLite Documentation](https://sqlite.org/docs.html)
