# C vs Rust Performance Benchmark Plan

## Objective
Compare modernized C implementation vs Rust implementation using real-world metrics to make data-driven production decision.

## Benchmark Categories

### 1. Binary Size & Startup Time (5 min)
**What**: Measure executable size and cold start latency
**Why**: Important for container deployments, serverless, and scaling

**C Binaries** (from `ethhook-c/build/`):
- ethhook-admin-api
- ethhook-ingestor  
- ethhook-processor
- ethhook-delivery

**Rust Binaries** (from `target/release/`):
- ethhook-admin-api (Rust)
- pipeline (unified Rust service)

**Metrics**:
- Binary size (KB)
- Startup time (cold start to "ready")
- Memory footprint at idle
- Container image size (if Dockerized)

### 2. JWT Token Operations (10 min)
**What**: Benchmark JWT create/verify operations
**Why**: Critical path for admin API authentication

**Test**: Create and verify 100,000 JWT tokens

**C Implementation**: `src/admin-api/auth.c`
- Uses OpenSSL HMAC-SHA256 directly
- Custom base64url encoding
- ~150 lines

**Rust Implementation**: `crates/admin-api/src/auth.rs`
- Uses `jsonwebtoken` crate
- Industry-standard library

**Metrics**:
- Operations per second (create)
- Operations per second (verify)
- Memory usage during test
- CPU time

### 3. JSON Parsing Performance (10 min)
**What**: Parse blockchain event JSON payloads
**Why**: Core operation in event processing pipeline

**Test Data**: 10,000 typical Ethereum event JSONs (~500 bytes each)

**C Implementation**: `src/common/yyjson.c`
- yyjson (ultra-fast C library)
- Single-pass parser

**Rust Implementation**: `serde_json`
- Standard Rust JSON library
- Zero-copy deserialization

**Metrics**:
- Parse operations per second
- Memory allocations
- Peak memory usage
- Cache efficiency

### 4. Memory Safety Overhead (15 min)
**What**: Compare memory access patterns and safety checks
**Why**: Rust's safety guarantees might have performance cost

**Tests**:
- Array/slice access (bounds checking)
- String operations
- Buffer management
- Memory allocation patterns

**Metrics**:
- Raw pointer operations per second
- Bounds-checked access per second
- Memory allocation latency
- Deallocation latency

### 5. Concurrency Performance (20 min)
**What**: Multi-threaded workload simulation
**Why**: Services process events concurrently

**Test Scenario**:
- Spawn 100 threads
- Each processes 1000 events
- Measure completion time and contention

**C Implementation**:
- pthread.h
- Manual synchronization primitives
- Explicit locks

**Rust Implementation**:
- tokio async runtime
- async/await
- Lock-free data structures where possible

**Metrics**:
- Total throughput (events/sec)
- Latency (P50, P95, P99)
- Thread creation overhead
- Lock contention time
- Context switch overhead

### 6. Database Operations (15 min)
**What**: SQLite query performance
**Why**: Admin API uses SQLite extensively

**Test Queries**:
- SELECT (1000 queries)
- INSERT (1000 inserts)
- UPDATE (1000 updates)
- Complex JOIN (100 queries)

**C Implementation**: Direct SQLite3 C API
**Rust Implementation**: sqlx with compile-time query checking

**Metrics**:
- Queries per second
- Connection pool overhead
- Prepared statement caching benefit
- Transaction throughput

### 7. HTTP Request Handling (20 min)
**What**: Admin API endpoint performance
**Why**: Real-world user-facing performance

**Test Endpoints**:
- GET /api/health (10,000 requests)
- GET /api/users (1,000 requests)
- POST /api/login (1,000 requests)
- GET /api/events?limit=100 (100 requests)

**C Implementation**: libmicrohttpd
**Rust Implementation**: axum + tokio

**Load Test Tool**: `wrk` or `ab` (Apache Bench)

**Metrics**:
- Requests per second
- Latency distribution
- Memory per connection
- Max concurrent connections
- Error rate under load

### 8. End-to-End Event Processing (30 min)
**What**: Simulate complete blockchain event → webhook delivery pipeline
**Why**: Most realistic production scenario

**Scenario**:
1. Receive 10,000 mock blockchain events
2. Parse and validate
3. Match against endpoints
4. Publish to delivery queue
5. Deliver webhooks

**Metrics**:
- Total pipeline latency (P50, P95, P99)
- Events processed per second
- Memory usage over time
- CPU utilization
- Failure handling overhead

## Benchmark Implementation

### Tools Needed

```bash
# Install benchmarking tools
brew install hyperfine  # For command benchmarking
brew install wrk        # For HTTP load testing
brew install valgrind   # For memory profiling (if needed)

# Rust benchmark framework (already available)
cargo install cargo-criterion
```

### Test Harness Structure

```
benchmarks/
├── 1_binary_size.sh          # Binary size comparison
├── 2_jwt_bench.c              # C JWT benchmark
├── 2_jwt_bench.rs             # Rust JWT benchmark
├── 3_json_parse.c             # C JSON parsing
├── 3_json_parse.rs            # Rust JSON parsing
├── 4_memory_safety.c          # C memory operations
├── 4_memory_safety.rs         # Rust memory operations
├── 5_concurrency.c            # C threading test
├── 5_concurrency.rs           # Rust async test
├── 6_database.c               # C SQLite benchmark
├── 6_database.rs              # Rust sqlx benchmark
├── 7_http_load_test.sh        # wrk load test script
├── 8_e2e_pipeline.sh          # End-to-end test
├── run_all.sh                 # Master script
├── test_data/
│   ├── events.json            # Sample blockchain events
│   ├── users.sql              # Test database data
│   └── endpoints.sql          # Test endpoint configs
└── results/
    └── benchmark_results.md   # Generated report
```

### Execution Plan

```bash
# 1. Build both implementations (optimized)
cd ethhook-c && cmake --build build --config Release
cd .. && cargo build --release

# 2. Run benchmarks
cd benchmarks
./run_all.sh

# 3. Generate report
./generate_report.sh > results/benchmark_results.md
```

### Expected Results Format

```markdown
# C vs Rust Benchmark Results

## Summary

| Metric | C | Rust | Winner | Margin |
|--------|---|------|--------|--------|
| Binary Size | 323 KB | 5.2 MB | C | 16x smaller |
| Startup Time | 2 ms | 8 ms | C | 4x faster |
| JWT Create | 50K ops/s | 45K ops/s | C | 11% faster |
| JSON Parse | 180K ops/s | 150K ops/s | C | 20% faster |
| Memory Safety | N/A | 0 bugs | Rust | ∞ safer |
| Concurrency | 8K events/s | 12K events/s | Rust | 50% faster |
| Database Ops | 15K qps | 13K qps | C | 15% faster |
| HTTP RPS | 25K rps | 35K rps | Rust | 40% faster |
| E2E Latency (P95) | 8 ms | 5 ms | Rust | 37% faster |

## Detailed Analysis
[Charts, graphs, flame graphs]

## Recommendations
Based on data...
```

## Next Steps

1. **Create benchmark harness** (30 min)
2. **Generate test data** (15 min)
3. **Run benchmarks** (2 hours)
4. **Analyze results** (30 min)
5. **Make recommendation** (data-driven)

## Critical Questions to Answer

1. **Is C actually faster?** (Hypothesis: Yes for single-threaded, No for concurrent)
2. **How much does memory safety cost?** (Hypothesis: <5% in real workloads)
3. **Which is more predictable?** (Latency jitter, tail latency)
4. **Which is easier to optimize?** (Profiling, tuning)
5. **Which is more maintainable long-term?** (Not just performance)

## Decision Framework

After benchmarks, score each dimension:

| Criterion | Weight | C Score | Rust Score |
|-----------|--------|---------|------------|
| Raw Speed | 20% | ? | ? |
| Memory Efficiency | 15% | ? | ? |
| Concurrency | 25% | ? | ? |
| Safety/Reliability | 25% | ? | ? |
| Maintainability | 15% | ? | ? |

**Final Score = Σ(Weight × Score)**

Winner: Data-driven decision based on YOUR production priorities.

---

Ready to implement? I can create the benchmark harness now.
