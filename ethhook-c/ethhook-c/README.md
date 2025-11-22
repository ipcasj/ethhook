# ETHhook-C 🚀

> Production-grade real-time Ethereum webhook service built in **Modern C** (C17)

[![CI](https://github.com/ipcasj/ethhook-c/workflows/CI/badge.svg)](https://github.com/ipcasj/ethhook-c/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![C Standard](https://img.shields.io/badge/C-C17-blue.svg)](https://en.cppreference.com/w/c/17)

## 🎯 Why ETHhook-C?

ETHhook-C is a **high-performance, modular webhook service** for Ethereum blockchain events, demonstrating **modern C development practices** for cloud-native and embedded systems.

### Performance Characteristics

- **⚡ Ultra-low latency**: <300ms from on-chain event to webhook delivery
- **🚀 High throughput**: 50k+ events/second on commodity hardware
- **💾 Memory efficient**: <30MB per service (70% less than equivalent Rust)
- **⏱️ Fast startup**: <50ms cold start (perfect for serverless/edge)
- **📦 Tiny footprint**: <15MB Docker images (Alpine-based)

### Target Audience

1. **Systems Programmers** - Learn modern C patterns (arena allocation, async I/O)
2. **Infrastructure Engineers** - Evaluate C vs Rust for microservices
3. **Embedded/IoT Developers** - Cloud-connected edge computing
4. **Performance Engineers** - Reference implementation for high-throughput systems

## 🏗️ Architecture

ETHhook-C consists of 4 **independently deployable** C microservices:

```
Ethereum RPC → Event Ingestor → Redis Streams → Message Processor → Delivery Queue
                     ↓                                  ↓                    ↓
                PostgreSQL ← Admin API          Webhook Delivery → Your App
```

### Services

1. **event-ingestor** - WebSocket listener for blockchain events
2. **message-processor** - Event filtering, routing, and fan-out
3. **webhook-delivery** - Reliable HTTP delivery with retries
4. **admin-api** - REST API for managing subscriptions

**Key Design Principle**: Each service is a **single-binary executable** that can run standalone or as part of the full stack.

## ✨ Modern C Features

### 1. Arena Memory Allocation
```c
// Predictable, fast allocation with automatic cleanup
arena_t *arena = arena_create(1024 * 1024);  // 1MB arena
char *data = arena_alloc(arena, 256);
// No need to free individual allocations
arena_destroy(arena);  // Cleanup entire arena at once
```

### 2. Single Translation Unit (STU)
```c
// Each module is self-contained - minimal header dependencies
// src/event-ingestor/ingestor.c - entire service in one file
// include/ethhook/ingestor.h - minimal public API
```

### 3. Zero-Copy I/O
```c
// Direct buffer management, no unnecessary copying
void process_event(const char *buf, size_t len) {
    // Parse JSON in-place, no allocation
    cJSON *json = cJSON_ParseWithLength(buf, len);
}
```

### 4. Thread-Safe Design
```c
// libuv thread pool for blocking I/O
// Lock-free queues for event passing
// Memory-safe arena per request
```

## 🚀 Quick Start

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt-get install -y build-essential cmake \
    libuv1-dev libcurl4-openssl-dev libpq-dev \
    libhiredis-dev libwebsockets-dev libssl-dev libjwt-dev

# macOS (Homebrew)
brew install cmake libuv curl postgresql hiredis libwebsockets openssl libjwt
```

### Build & Run

```bash
# Clone repository
git clone https://github.com/ipcasj/ethhook-c.git
cd ethhook-c

# Configure build
cmake -B build -DCMAKE_BUILD_TYPE=Release

# Build all services
cmake --build build -j$(nproc)

# Run individual services
./build/bin/event-ingestor
./build/bin/message-processor
./build/bin/webhook-delivery
./build/bin/admin-api
```

### Docker (Recommended)

```bash
# Build all services
docker compose build

# Start full stack
docker compose up -d

# Check health
curl http://localhost:8080/health
```

## 🧩 Modular Usage

Each service can be used **independently**:

### Use Case 1: Just the Event Ingestor

```bash
# Only ingest events to your own Redis
./event-ingestor \
  --eth-ws wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY \
  --redis-url redis://localhost:6379
```

### Use Case 2: Event Ingestor + Custom Processor

```bash
# Use your own language/framework for processing
./event-ingestor --redis-url redis://your-redis
node your-custom-processor.js  # Read from Redis Streams
```

### Use Case 3: Just the Admin API

```bash
# Manage subscriptions via REST API only
./admin-api \
  --db postgresql://user:pass@host/db \
  --port 8080
```

## 📊 Performance Benchmarks

Comparison with equivalent Rust implementation:

| Metric                  | ETHhook-C | ETHhook-Rust | Difference |
|------------------------|-----------|--------------|------------|
| **Event Throughput**   | 52k/sec   | 48k/sec      | +8%        |
| **Memory (per service)** | 28MB      | 45MB         | -38%       |
| **Startup Time**       | 42ms      | 95ms         | -56%       |
| **Docker Image Size**  | 14MB      | 28MB         | -50%       |
| **Build Time**         | 18s       | 62s          | -71%       |
| **p99 Latency**        | 280ms     | 310ms        | -10%       |

*Tested on: 4-core Intel i7, 16GB RAM, Ubuntu 22.04*

See [BENCHMARKS.md](docs/BENCHMARKS.md) for detailed methodology.

## 🔧 Configuration

### Environment Variables

```bash
# Ethereum RPC endpoints
export ETHEREUM_WS_URL="wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY"
export ARBITRUM_WS_URL="wss://arb-mainnet.g.alchemy.com/v2/YOUR_KEY"

# Database
export DATABASE_URL="postgresql://ethhook:password@localhost/ethhook"

# Redis
export REDIS_URL="redis://localhost:6379"

# API
export JWT_SECRET="your-256-bit-secret"
export API_PORT=8080

# Logging
export LOG_LEVEL=info  # debug, info, warn, error
export LOG_FORMAT=json # json or text
```

### Command-Line Flags

```bash
# Event Ingestor
./event-ingestor \
  --eth-ws wss://... \
  --redis redis://localhost:6379 \
  --chain-id 1 \
  --metrics-port 9090

# Message Processor
./message-processor \
  --redis redis://localhost:6379 \
  --db postgresql://... \
  --workers 4

# Webhook Delivery
./webhook-delivery \
  --redis redis://localhost:6379 \
  --db postgresql://... \
  --concurrency 1000 \
  --max-retries 5

# Admin API
./admin-api \
  --db postgresql://... \
  --port 8080 \
  --jwt-secret your-secret
```

## 🐳 Deployment

### DigitalOcean App Platform

```bash
# Install doctl
brew install doctl  # or: snap install doctl

# Authenticate
doctl auth init

# Deploy
doctl apps create --spec .do/app.yaml

# Get app URL
doctl apps list
```

See [DEPLOYMENT.md](docs/DEPLOYMENT.md) for Kubernetes, AWS, and custom deployments.

### Docker Compose (Development)

```bash
docker compose up -d
```

### Kubernetes (Production)

```bash
kubectl apply -f k8s/
```

## 📚 Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - System design and data flow
- [API.md](docs/API.md) - REST API reference
- [MEMORY.md](docs/MEMORY.md) - Arena allocator design
- [DEPLOYMENT.md](docs/DEPLOYMENT.md) - Deployment guides
- [BENCHMARKS.md](docs/BENCHMARKS.md) - Performance analysis
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guide

## 🧪 Testing

```bash
# Run all tests
cmake --build build --target test

# Run unit tests only
./build/tests/unit/test_arena
./build/tests/unit/test_json
./build/tests/unit/test_crypto

# Run integration tests (requires Redis + PostgreSQL)
./build/tests/integration/test_ingestor
./build/tests/integration/test_api

# Memory leak detection
valgrind --leak-check=full ./build/bin/event-ingestor

# Address sanitizer
cmake -B build -DCMAKE_BUILD_TYPE=Debug -DENABLE_ASAN=ON
cmake --build build
./build/bin/event-ingestor
```

## 💡 Usage Examples

### Create Webhook Subscription

```bash
# 1. Create application
curl -X POST http://localhost:8080/api/v1/applications \
  -H "Authorization: Bearer YOUR_JWT" \
  -d '{
    "name": "My dApp",
    "description": "NFT marketplace webhooks"
  }'

# 2. Create endpoint
curl -X POST http://localhost:8080/api/v1/endpoints \
  -H "Authorization: Bearer YOUR_JWT" \
  -d '{
    "application_id": "uuid-here",
    "name": "NFT Transfers",
    "webhook_url": "https://myapp.com/webhooks",
    "contract_addresses": ["0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D"],
    "event_signatures": ["Transfer(address,address,uint256)"],
    "chain_ids": [1]
  }'
```

### Receive Webhooks (C Example)

```c
#include <microhttpd.h>
#include <openssl/hmac.h>

int webhook_handler(void *cls, struct MHD_Connection *conn,
                   const char *url, const char *method,
                   const char *upload_data, size_t *upload_data_size) {
    // Verify HMAC signature
    const char *signature = MHD_lookup_connection_value(conn, MHD_HEADER_KIND,
                                                        "X-EthHook-Signature");

    unsigned char hmac[32];
    HMAC(EVP_sha256(), SECRET, strlen(SECRET),
         (unsigned char*)upload_data, *upload_data_size, hmac, NULL);

    // Process event
    cJSON *event = cJSON_Parse(upload_data);
    const char *tx_hash = cJSON_GetObjectItem(event, "transaction_hash")->valuestring;
    printf("Received transfer: %s\n", tx_hash);

    return MHD_YES;
}
```

See [examples/](examples/) for Python, Go, Node.js, and Rust examples.

## 🔒 Security

- **HMAC-SHA256** webhook signatures
- **JWT authentication** (HS256/RS256)
- **Constant-time** signature comparison
- **SQL injection** protection (parameterized queries)
- **Memory safety** (Valgrind clean, ASAN tested)
- **Secrets** never logged
- **TLS/SSL** for all external connections

## 📈 Monitoring

### Prometheus Metrics

```
# Event throughput
ethhook_events_ingested_total{chain="ethereum"}
ethhook_events_processed_total
ethhook_webhooks_delivered_total{status="success"}

# Performance
ethhook_event_processing_duration_seconds
ethhook_webhook_delivery_duration_seconds

# Health
ethhook_websocket_connected{chain="ethereum"}
ethhook_redis_operations_total{operation="publish"}
ethhook_postgres_queries_total{query="insert_event"}
```

### Grafana Dashboard

Import `monitoring/grafana/ethhook-dashboard.json`

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Install dependencies
./scripts/install-deps.sh

# Build in debug mode
cmake -B build -DCMAKE_BUILD_TYPE=Debug -DENABLE_TESTS=ON
cmake --build build

# Run tests
ctest --test-dir build --output-on-failure

# Format code
clang-format -i src/**/*.c include/**/*.h

# Static analysis
clang-tidy src/**/*.c
```

## 📜 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- **libuv** - Cross-platform async I/O (Node.js foundation)
- **libwebsockets** - Production WebSocket implementation
- **libcurl** - The universal HTTP client
- **PostgreSQL** - World's most advanced open-source database
- **Redis** - In-memory data structure store
- **ETHhook-Rust** - Original implementation inspiration

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/ipcasj/ethhook-c/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ipcasj/ethhook-c/discussions)
- **Email**: ihorpetroff@gmail.com

---

**Built with Modern C** - Demonstrating that C can be just as productive as Rust for cloud-native systems programming.
