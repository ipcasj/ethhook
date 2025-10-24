# Testing Strategy & Coverage Analysis

## Current Test Coverage Status

### Summary


### Test Coverage by Service

#### ✅ Event Ingestor (19 tests)

**Strong Coverage:**


**Integration Tests:** 1 file (Redis-dependent, marked as `#[ignore]`)

#### ✅ Webhook Delivery (10 tests)

**Strong Coverage:**


#### ⚠️ Message Processor (2 tests)

**Weak Coverage:**


#### ❌ Admin API (0 tests) - **CRITICAL GAP**

**No Tests:**



## Production-Ready Testing Best Practices

### 1. **Unit Test Coverage (Target: 70-80%)**

#### What to Test

```rust
// ✅ Business logic
#[test]
fn test_calculate_backoff() {
    assert_eq!(calculate_backoff(1, 1000), 1000);
    assert_eq!(calculate_backoff(2, 1000), 2000);
}

// ✅ Edge cases
#[test]
fn test_empty_input() {
    assert!(parse_events(&[]).is_empty());
}

// ✅ Error conditions
#[test]
fn test_invalid_signature() {
    let result = verify_hmac("invalid", "secret", "data");
    assert!(result.is_err());
}

// ✅ State transitions
#[test]
fn test_circuit_breaker_state_machine() {
    let mut cb = CircuitBreaker::new(5);
    assert_eq!(cb.state(), State::Closed);
    // ... test all transitions
}
```

#### Current Gaps to Fill

**Admin API:**

```rust
// crates/admin-api/src/handlers/users.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_register_user_success() {
        // Mock database, test registration
    }
    
    #[tokio::test]
    async fn test_register_duplicate_email() {
        // Should return 409 Conflict
    }
    
    #[tokio::test]
    async fn test_login_invalid_password() {
        // Should return 401 Unauthorized
    }
    
    #[test]
    fn test_jwt_token_generation() {
        // Test token creation and validation
    }
}
```

**Message Processor:**

```rust
// crates/message-processor/src/matcher.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_match_single_contract() {
        // Test endpoint matching logic
    }
    
    #[tokio::test]
    async fn test_match_multiple_endpoints() {
        // Test fan-out to multiple webhooks
    }
    
    #[test]
    fn test_no_match_returns_empty() {
        // Test when no endpoints match
    }
}
```

---

### 2. **Integration Tests (Target: Key User Journeys)**

#### Current Status

- ✅ Event Ingestor: 1 integration test file
- ❌ Message Processor: 0 integration tests
- ❌ Webhook Delivery: 0 integration tests
- ❌ Admin API: 0 integration tests

#### What to Add

**End-to-End Flow Tests:**

```rust
// crates/admin-api/tests/integration_test.rs
#[tokio::test]
#[ignore] // Requires PostgreSQL + Redis
async fn test_complete_user_flow() {
    // 1. Register user
    // 2. Login and get JWT
    // 3. Create application (get API key)
    // 4. Create endpoint
    // 5. Use API key to authenticate
}

// crates/event-ingestor/tests/full_pipeline_test.rs
#[tokio::test]
#[ignore] // Requires Redis + Ethereum node
async fn test_event_ingestion_to_delivery() {
    // 1. Ingest event from blockchain
    // 2. Deduplicate in Redis
    // 3. Publish to stream
    // 4. Message processor picks up
    // 5. Webhook delivery sends HTTP
    // 6. Verify delivery attempt logged
}
```

---

### 3. **Load/Performance Tests**

#### Tools

- **k6** (for HTTP load testing)
- **Rust criterion** (for microbenchmarks)

```javascript
// scripts/load_test.js (k6)
import http from 'k6/http';
import { check } from 'k6';

export let options = {
    stages: [
        { duration: '30s', target: 100 },  // Ramp up to 100 users
        { duration: '1m', target: 1000 },  // Spike to 1000 users
        { duration: '30s', target: 0 },    // Ramp down
    ],
    thresholds: {
        http_req_duration: ['p(95)<500'], // 95% under 500ms
    },
};

export default function() {
    let res = http.post('http://localhost:8080/api/v1/ingest', JSON.stringify({
        chain_id: 1,
        events: [/* ... */]
    }));
    
    check(res, {
        'status is 200': (r) => r.status === 200,
        'response time OK': (r) => r.timings.duration < 500,
    });
}
```

**Run:**

```bash
k6 run scripts/load_test.js
```

---

### 4. **CI/CD Pipeline Tests**

#### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15-alpine
        env:
          POSTGRES_USER: ethhook
          POSTGRES_PASSWORD: password
          POSTGRES_DB: ethhook
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache Cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run Migrations
        env:
          DATABASE_URL: postgresql://ethhook:password@localhost:5432/ethhook
        run: |
          cargo install sqlx-cli --no-default-features --features postgres
          sqlx migrate run
      
      - name: Check Formatting
        run: cargo fmt --all -- --check
      
      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Run Unit Tests
        run: cargo test --workspace --lib
      
      - name: Run Integration Tests
        env:
          DATABASE_URL: postgresql://ethhook:password@localhost:5432/ethhook
          REDIS_URL: redis://localhost:6379
        run: cargo test --workspace --test "*" -- --ignored
      
      - name: Check SQLx Offline Cache
        run: |
          cargo sqlx prepare --check --workspace
      
      - name: Security Audit
        run: |
          cargo install cargo-audit
          cargo audit
      
      - name: Code Coverage
        uses: actions-rs/tarpaulin@v0.1
        with:
          args: '--workspace --out Xml'
      
      - name: Upload Coverage
        uses: codecov/codecov-action@v3

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Release
        run: cargo build --release --workspace
      
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: binaries
          path: |
            target/release/event-ingestor
            target/release/ethhook-message-processor
            target/release/ethhook-webhook-delivery
            target/release/ethhook-admin-api
```

---

### 5. **Contract Testing (API)**

For microservices, use contract tests to ensure API compatibility:

```rust
// crates/admin-api/tests/contract_test.rs
#[tokio::test]
async fn test_register_endpoint_contract() {
    // Given: Valid request payload
    let payload = json!({
        "email": "test@example.com",
        "password": "SecurePass123!",
        "name": "Test User"
    });
    
    // When: POST to /api/v1/auth/register
    let response = client.post("/api/v1/auth/register")
        .json(&payload)
        .send()
        .await
        .unwrap();
    
    // Then: Contract validation
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["user"]["id"].is_string());
    assert_eq!(body["user"]["email"], "test@example.com");
    assert!(body["token"].is_string());
    // ... validate full response schema
}
```

---

### 6. **Database Migration Tests**

```rust
// migrations/tests/migration_test.rs
#[tokio::test]
async fn test_migrations_up_and_down() {
    let pool = PgPool::connect(&database_url).await.unwrap();
    
    // Apply all migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();
    
    // Verify schema
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert!(tables.contains(&"users".to_string()));
    assert!(tables.contains(&"applications".to_string()));
    assert!(tables.contains(&"endpoints".to_string()));
}
```

---

### 7. **Chaos Engineering Tests**

```rust
// tests/chaos_test.rs
#[tokio::test]
#[ignore]
async fn test_webhook_delivery_with_intermittent_failures() {
    // Simulate network failures, timeouts, etc.
    // Verify circuit breaker opens/closes correctly
    // Verify retries work as expected
}

#[tokio::test]
#[ignore]
async fn test_redis_connection_loss_recovery() {
    // Kill Redis mid-test
    // Verify graceful degradation
    // Verify reconnection logic
}
```

---

## Recommended Testing Priorities

### 🔴 **Critical (Do Now)**

1. **Admin API Integration Tests** - User flows, authentication
2. **Message Processor Unit Tests** - Endpoint matching logic
3. **End-to-End Pipeline Test** - Event → Webhook delivery
4. **CI/CD Pipeline Setup** - GitHub Actions with PostgreSQL/Redis

### 🟡 **High Priority (Next Sprint)**

1. **Load Tests** - Verify 10,000 events/sec target
2. **Security Tests** - SQL injection, JWT tampering, API key validation
3. **Contract Tests** - API schema validation
4. **Database Migration Tests** - Up/down migrations

### 🟢 **Medium Priority (Future)**

1. **Chaos Tests** - Network failures, service crashes
2. **Performance Benchmarks** - Criterion microbenchmarks
3. **Property-Based Tests** - Quickcheck/proptest
4. **Fuzz Tests** - cargo-fuzz for parser code

---

## Test Coverage Tools

### Measure Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Open coverage/index.html in browser
open coverage/index.html
```

### Continuous Coverage

```bash
# Install cargo-watch for live testing
cargo install cargo-watch

# Auto-run tests on file changes
cargo watch -x test
```

---

## Test Organization Best Practices

### Directory Structure

```text
crates/
  admin-api/
    src/
      handlers/
        users.rs          # Business logic
    tests/
      integration_test.rs  # Full API tests
      contract_test.rs     # API contract tests
```

### Naming Conventions

- **Unit tests**: `test_<function>_<scenario>()`
- **Integration tests**: `test_<feature>_<flow>()`
- **Benchmark tests**: `bench_<operation>()`

### Test Annotations

```rust
#[test]              // Unit test (no async)
#[tokio::test]       // Async unit test
#[ignore]            // Skip by default (needs external service)
#[should_panic]      // Expects panic
```

### Current State: ⚠️ **Incomplete**

- ✅ Good unit test coverage for core logic
- ❌ Missing Admin API tests (0 tests)
- ❌ Missing integration tests (except event-ingestor)
- ❌ No CI/CD pipeline
- ❌ No load/performance tests

### Target State: ✅ **Production-Ready**

- 70-80% unit test coverage
- Integration tests for all services
- CI/CD with automated testing
- Load tests validating performance targets
- Security/chaos tests
- Code coverage tracking

### Effort Estimate

- **Admin API Tests**: 2-3 days
- **CI/CD Setup**: 1 day
- **Integration Tests**: 3-4 days
- **Load Tests**: 1-2 days
- **Total**: ~1-2 weeks for comprehensive test suite

Would you like me to start implementing tests for a specific area (e.g., Admin API integration tests)?
