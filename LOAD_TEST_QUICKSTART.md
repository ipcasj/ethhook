# Load Test Quick Start

## Simple 3-Step Load Test

### Step 1: Start the webhook receiver

```bash
cd demo-webhook-receiver
python3 load_test_receiver.py
```

Leave this running in a terminal.

### Step 2: Start the services (in separate terminals)

Terminal 2:
```bash
RUST_LOG=info cargo run --bin message-processor
```

Terminal 3:
```bash
RUST_LOG=info cargo run --bin webhook-delivery
```

Wait ~5 seconds for both to start.

### Step 3: Run the load test

Terminal 4:
```bash
./target/release/load-tester \
  --events 1000 \
  --rate 500 \
  --concurrency 5
```

## What You'll See

The load tester will show:
- Progress bar of event publishing
- Final performance report with:
  - Publishing throughput
  - Webhook delivery latency (min/avg/p95/p99/max)
  - Overall system throughput

Example output:
```
📊 Load Test Results
====================================================================

Event Publishing:
  Total Events: 1000
  Duration: 2.15s
  Throughput: 465.12 events/sec

Webhook Delivery:
  Total Delivered: 1000
  Success Rate: 100.00%
  End-to-End Latency:
    Avg: 123ms
    P95: 234ms
    P99: 345ms

🎯 Performance Analysis:
  ✅ Latency: PASS (Target: <500ms)
  ✅ Throughput: EXCELLENT (465 webhooks/sec)
```

## Troubleshooting

If no webhooks are delivered:
1. Make sure endpoints are set up: `./scripts/setup_high_traffic_endpoints.sh`
2. Check services are running: `lsof -i :8081` and `lsof -i :8082`
3. Check receiver is running: `curl http://localhost:8000/health`

## Automated Test

For a fully automated test (sets up everything):
```bash
./scripts/run_load_test.sh 1000 500 5
```

See [docs/LOAD_TESTING.md](docs/LOAD_TESTING.md) for full documentation.
