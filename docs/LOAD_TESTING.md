# Load Testing Guide

This guide explains how to run load tests on EthHook to validate performance claims and identify bottlenecks.

## Overview

The load testing suite includes:

- **Load Tester**: Rust-based event generator that publishes events to Redis streams
- **Performance Receiver**: Python webhook receiver with latency tracking
- **Automation Scripts**: Shell scripts to orchestrate tests
- **Metrics Collection**: Real-time performance statistics

## Performance Targets

According to the README, EthHook claims:

- **Latency**: <500ms from on-chain event to webhook delivery
- **Throughput**: 50k+ events/second

These tests validate those claims.

## Quick Start

### 1. Run a Quick Test (1,000 events)

```bash
./scripts/quick_load_test.sh
```

This runs a small test:
- 1,000 events
- 500 events/sec
- 5 concurrent publishers
- ~2 seconds duration

### 2. Run a Medium Test (10,000 events)

```bash
./scripts/run_load_test.sh 10000 1000 10
```

This runs a medium test:
- 10,000 events
- 1,000 events/sec
- 10 concurrent publishers
- ~10 seconds duration

### 3. Run a Stress Test (100,000 events)

```bash
./scripts/run_load_test.sh 100000 5000 20
```

This runs a stress test:
- 100,000 events
- 5,000 events/sec
- 20 concurrent publishers
- ~20 seconds duration

## Prerequisites

The load test script automatically checks for:

- ✅ Docker running (for PostgreSQL)
- ✅ Redis accessible
- ✅ Python 3 installed (for receiver)
- ✅ Rust toolchain (to build load tester)

## What the Load Test Does

### 1. Setup Phase

1. **Database Setup**: Creates test endpoints in PostgreSQL
   - USDC Transfer events
   - WETH events (Transfer, Deposit, Withdrawal)
   - DAI Transfer events
   - LINK Transfer events
   - Uniswap Swap events

2. **Service Startup**: Ensures services are running
   - Message Processor (port 8081)
   - Webhook Delivery (port 8082)
   - Webhook Receiver (port 8000)

3. **Readiness Check**: Waits for services to signal ready
   - Uses Redis readiness keys (production pattern)
   - Validates receiver health endpoint

### 2. Load Generation Phase

The load tester:

1. **Generates Events**: Creates realistic blockchain events with:
   - Block numbers, hashes
   - Transaction hashes
   - Contract addresses matching test endpoints
   - Event topics (Transfer, Swap, etc.)
   - Timestamps for latency measurement

2. **Publishes to Redis**: Writes events to Redis streams
   - Uses `XADD` commands
   - Distributes across concurrent publishers
   - Rate-limits to target throughput

3. **Tracks Metrics**: Records:
   - Events published
   - Publishing latency
   - Error rate

### 3. Measurement Phase

The webhook receiver:

1. **Receives Webhooks**: Accepts POST requests at `/webhook`

2. **Calculates Latency**: Measures end-to-end time
   - Extracts `created_at` timestamp from payload
   - Calculates delta: `now - created_at`
   - Records in histogram for percentile calculation

3. **Tracks Throughput**: Counts requests per second
   - Overall throughput
   - Recent throughput (sliding window)

4. **Exposes Metrics**: Provides `/metrics` endpoint with:
   ```json
   {
     "total_requests": 10000,
     "successful": 10000,
     "failed": 0,
     "uptime_seconds": 12.34,
     "throughput_rps": 810.37,
     "recent_throughput_rps": 850.21,
     "latency_ms": {
       "min": 12.45,
       "max": 487.32,
       "avg": 125.67,
       "median": 98.23,
       "p95": 234.56,
       "p99": 345.67
     }
   }
   ```

### 4. Results Phase

The load tester:

1. **Waits for Completion**: Gives webhooks time to be delivered (5s)

2. **Fetches Metrics**: Calls receiver's `/metrics` endpoint

3. **Generates Report**: Displays:
   - Event publishing stats
   - Webhook delivery stats
   - End-to-end latency percentiles
   - System throughput
   - Performance verdict (PASS/FAIL)

## Understanding the Results

### Sample Output

```
📊 Load Test Results
====================================================================

Event Publishing:
  Total Events: 10000
  Errors: 0
  Duration: 10.23s
  Throughput: 977.65 events/sec
  Publish Latency: min: 1ms, p50: 2ms, p95: 5ms, p99: 8ms, max: 15ms

Webhook Delivery:
  Total Delivered: 10000
  Success Rate: 100.00%
  End-to-End Latency:
    Min: 45ms
    Avg: 123ms
    Median: 98ms
    P95: 234ms
    P99: 345ms
    Max: 487ms

System Throughput:
  Overall: 810.37 webhooks/sec

🎯 Performance Analysis:
  ✅ Latency: PASS (Target: <500ms)
  ✅ Throughput: EXCELLENT (810 webhooks/sec)
```

### Interpreting Results

#### Latency Metrics

- **Min**: Best-case latency (usually network + processing overhead)
- **Median (P50)**: Typical latency for most webhooks
- **P95**: 95% of webhooks delivered within this time
- **P99**: 99% of webhooks delivered within this time (tail latency)
- **Max**: Worst-case latency (may include retries)

**Targets**:
- P50 < 200ms: Excellent
- P95 < 500ms: Good (meets target)
- P99 < 1000ms: Acceptable
- Max > 5000ms: Investigate (may indicate issues)

#### Throughput Metrics

- **Events/sec**: Rate of event publishing to Redis
- **Webhooks/sec**: Rate of webhook delivery to receiver

**Targets**:
- 100-500/sec: Good for single instance
- 500-1000/sec: Very good
- 1000-5000/sec: Excellent
- 5000+/sec: Outstanding (approaches 50k target with scaling)

#### Success Rate

- **100%**: Perfect (no failures)
- **99.9%**: Excellent
- **99%**: Good
- **<99%**: Investigate errors

## Troubleshooting

### Test Fails to Start

**Error**: `PostgreSQL container is not running`
```bash
docker compose up -d postgres redis
```

**Error**: `Failed to start Message Processor`
```bash
# Check logs
tail -f /tmp/message-processor.log

# Ensure DATABASE_URL is set
echo $DATABASE_URL
```

### Low Throughput

**Symptom**: <100 webhooks/sec

**Possible Causes**:
1. **Limited workers**: Increase `WORKER_COUNT` in webhook-delivery
2. **Database bottleneck**: Check PostgreSQL connections
3. **Network latency**: Receiver on different host
4. **CPU throttling**: Check system resources

**Solutions**:
```bash
# Increase workers (in .env)
WORKER_COUNT=100

# Check system resources
htop
docker stats
```

### High Latency

**Symptom**: P95 > 1000ms

**Possible Causes**:
1. **Slow webhook receiver**: Not responding fast enough
2. **Retry delays**: Webhooks failing and retrying
3. **Database queries**: Endpoint matching slow
4. **Redis congestion**: Too many streams/keys

**Solutions**:
```bash
# Check receiver logs
tail -f /tmp/receiver.log

# Check delivery logs for retries
tail -f /tmp/webhook-delivery.log | grep "Retry"

# Check Redis memory
redis-cli INFO memory
```

### Events Not Delivered

**Symptom**: `Total Delivered: 0`

**Possible Causes**:
1. **Endpoints not configured**: No matching endpoints in DB
2. **Services not running**: Message processor or delivery stopped
3. **Redis streams not consumed**: Consumer groups not active

**Solutions**:
```bash
# Check endpoints exist
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "SELECT name FROM endpoints WHERE is_active = true;"

# Check Redis streams
redis-cli XINFO GROUPS events:11155111

# Check service logs
tail -f /tmp/message-processor.log
tail -f /tmp/webhook-delivery.log
```

## Advanced Usage

### Custom Configuration

Set environment variables before running:

```bash
# Use different Redis
export REDIS_URL=redis://custom-redis:6379

# Use different receiver
export METRICS_URL=http://custom-receiver:8000/metrics

# Run test
./scripts/run_load_test.sh 10000 1000 10
```

### Manual Load Tester

Run the load tester directly for more control:

```bash
cd tools/load-tester
cargo run --release -- \
  --events 50000 \
  --rate 5000 \
  --concurrency 20 \
  --chain-id 11155111 \
  --redis-url redis://localhost:6379 \
  --metrics-url http://localhost:8000/metrics
```

### Continuous Load

Run for a specific duration instead of event count:

```bash
cd tools/load-tester
cargo run --release -- \
  --duration 60 \
  --rate 1000 \
  --concurrency 10
```

This runs for 60 seconds at 1000 events/sec.

## Performance Tuning

### Optimize for Latency

Focus on reducing end-to-end time:

1. **Reduce worker count**: Less contention
2. **Increase database connections**: Faster endpoint lookups
3. **Use connection pooling**: Reuse HTTP connections
4. **Enable keep-alive**: Reduce TCP handshakes

### Optimize for Throughput

Focus on maximizing events/sec:

1. **Increase worker count**: More parallel processing
2. **Batch Redis operations**: Fewer round trips
3. **Pipeline webhooks**: Send multiple in parallel
4. **Increase Redis memory**: Avoid evictions

### Optimize for Reliability

Focus on 100% delivery:

1. **Tune retry logic**: Balance speed vs retries
2. **Increase timeouts**: Allow slow receivers
3. **Monitor Redis memory**: Prevent OOM
4. **Scale workers**: Distribute load

## Benchmarking Different Scenarios

### Scenario 1: Burst Traffic

Simulate sudden spike (ICO launch, NFT drop):

```bash
./scripts/run_load_test.sh 10000 10000 20
```

- 10,000 events in 1 second
- Tests burst handling

### Scenario 2: Sustained Load

Simulate steady mainnet traffic:

```bash
./scripts/run_load_test.sh 100000 1000 10
```

- 100,000 events over 100 seconds
- Tests stability

### Scenario 3: Ultra High Traffic

Simulate peak mainnet (DeFi summer):

```bash
./scripts/run_load_test.sh 500000 10000 50
```

- 500,000 events at 10k/sec
- Tests scaling limits

## Next Steps

After load testing:

1. **Document Results**: Save metrics for baseline
2. **Identify Bottlenecks**: Use profiling tools
3. **Optimize Code**: Focus on hot paths
4. **Scale Infrastructure**: Add more workers/instances
5. **Repeat Tests**: Validate improvements

## Related Documentation

- [Architecture](../ARCHITECTURE.md)
- [E2E Tests Fixed](../E2E_TESTS_FIXED.md)
- [Deployment Guide](../DEPLOYMENT_QUICKSTART.md)
- [Setup Guide](../SETUP_GUIDE.md)

---

**Built with 🦀 Rust** - For maximum performance under load
