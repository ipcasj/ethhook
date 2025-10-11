# End-to-End Integration Tests

Comprehensive integration tests for the EthHook pipeline.

## 🎯 What is Tested

These tests validate the **complete data flow** through all 4 services:

```text
┌─────────────────┐
│  Event Ingestor │  ← Simulated: Publishes to raw-events stream
└────────┬────────┘
         │ Redis Stream: raw-events
         ↓
┌─────────────────┐
│ Message         │  ← Simulated: Reads from raw-events
│ Processor       │     Queries PostgreSQL for matching endpoints
└────────┬────────┘     Publishes to delivery-queue
         │ Redis Stream: delivery-queue
         ↓
┌─────────────────┐
│ Webhook         │  ← Simulated: Reads from delivery-queue
│ Delivery        │     Sends HTTP POST with HMAC signature
└────────┬────────┘
         │ HTTP POST + HMAC
         ↓
┌─────────────────┐
│ Mock Webhook    │  ← WireMock: Receives and validates webhook
│ Endpoint        │
└─────────────────┘
```

## 🧪 Test Scenarios

### 1. **Full Pipeline Test** (`test_end_to_end_pipeline`)

- ✅ Event published to raw-events stream
- ✅ Endpoint matching query (< 100ms latency)
- ✅ Job published to delivery-queue
- ✅ Webhook HTTP delivery with HMAC signature
- ✅ Mock server receives and validates request

### 2. **No Matching Endpoint** (`test_end_to_end_with_no_matching_endpoint`)

- ✅ Event for wrong contract address
- ✅ Matcher returns 0 results
- ✅ No jobs published to delivery-queue

### 3. **Wildcard Endpoint** (`test_end_to_end_with_wildcard_endpoint`)

- ✅ Endpoint with NULL contract + NULL topics (matches all)
- ✅ Random event matches wildcard endpoint
- ✅ Webhook delivered successfully

## 📋 Prerequisites

### Required Services

- **PostgreSQL 15**: `localhost:5432`
- **Redis 7**: `localhost:6379`

### Environment Variables

```bash
DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
REDIS_URL="redis://localhost:6379"
```

### Database Migrations

Run migrations before tests:

```bash
sqlx migrate run
```

## 🚀 Running Tests

### Automated (Recommended)

Use the helper script that handles infrastructure:

```bash
./scripts/run_e2e_tests.sh
```

This script:

1. Starts PostgreSQL + Redis via docker-compose
2. Waits for services to be ready
3. Runs database migrations
4. Executes all E2E tests
5. Shows test output with colors

### Manual

If infrastructure is already running:

```bash
# Set environment variables
export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
export REDIS_URL="redis://localhost:6379"

# Run tests (--ignored flag because they require infrastructure)
cargo test --package ethhook-e2e-tests -- --ignored --nocapture
```

### Run Specific Test

```bash
cargo test --package ethhook-e2e-tests test_end_to_end_pipeline -- --ignored --nocapture
```

## 📊 Expected Output

```text
🚀 Starting E2E Pipeline Test
✓ Connected to PostgreSQL and Redis
✓ Mock webhook server started at: http://127.0.0.1:54321/webhook
✓ Created test user, app, and endpoint: 3f8b4d2a-...
✓ Mock webhook configured to accept requests
✓ Cleared Redis streams

📥 STEP 1: Publishing event to raw-events stream...
✓ Published event with ID: 1696800000000-0

🔍 STEP 2: Simulating Message Processor matching...
✓ Found 1 matching endpoint(s) in 23ms

📤 STEP 3: Publishing job to delivery-queue...
✓ Published job with ID: 1696800000001-0

📋 STEP 4: Reading job from delivery-queue...
✓ Job retrieved from delivery-queue: 3f8b4d2a-...

🌐 STEP 5: Simulating webhook delivery...
✓ Webhook delivered with status: 200 OK

✅ STEP 6: Verifying webhook reception...
✓ Webhook confirmed received: Webhook processed successfully

🧹 Cleaning up test data...
✓ Test data cleaned up

✅ E2E PIPELINE TEST PASSED!
   Total latency: 127ms
```

## 🔍 What Gets Validated

### Data Integrity

- ✅ Event data flows correctly through all stages
- ✅ Contract address matching (exact, case-insensitive, NULL)
- ✅ Event topic matching (exact, subset, NULL)
- ✅ HMAC signature calculation and verification
- ✅ Job payload structure and serialization

### Performance

- ✅ Endpoint matching < 100ms (measured)
- ✅ Total pipeline latency reported
- ✅ No memory leaks or resource exhaustion

### Error Handling

- ✅ No matches returns empty result (not error)
- ✅ Invalid data doesn't crash pipeline
- ✅ Database queries handle NULLs correctly

## 🛠️ Test Infrastructure

### WireMock

Mock HTTP server for webhook endpoints:

- Validates POST requests received
- Checks headers (Content-Type, X-EthHook-Signature)
- Returns configurable responses
- Expectations verified automatically

### Test Helpers

- `create_test_user()`: Creates isolated test user
- `create_test_application()`: Creates test app
- `create_test_endpoint()`: Creates endpoint with specific rules
- `publish_raw_event()`: Simulates Event Ingestor
- `read_delivery_jobs()`: Simulates Webhook Delivery consumer
- `cleanup_test_data()`: Removes all test data

## 🏗️ Architecture Notes

### Why These Tests Are Important

1. **Validates Integration**: Unit tests can't catch integration issues
2. **Performance Baseline**: Measures actual latency through pipeline
3. **Regression Prevention**: Catches breaking changes immediately
4. **Documentation**: Shows how the system works end-to-end

## 🐛 Troubleshooting

### "Failed to connect to database"

- Ensure PostgreSQL is running: `docker-compose ps postgres`
- Check connection: `psql postgresql://ethhook:password@localhost:5432/ethhook`

### "Failed to connect to Redis"

- Ensure Redis is running: `docker-compose ps redis`
- Check connection: `redis-cli -h localhost -p 6379 ping`

### "Migration failed"

- Run migrations manually: `sqlx migrate run`
- Check migration status: `sqlx migrate info`

### "Timeout waiting for webhook"

- Ensure WireMock server is running and accessible
- Check mock server is accessible
- Increase timeout in test code
- Check network/firewall settings

## 📚 Related Documentation

- [Testing Strategy](../docs/TESTING_STRATEGY.md)
- [Message Processor Implementation](../docs/MESSAGE_PROCESSOR_IMPLEMENTATION.md)
- [Webhook Delivery Implementation](../docs/WEBHOOK_DELIVERY_IMPLEMENTATION.md)
- [Architecture](../ARCHITECTURE.md)
