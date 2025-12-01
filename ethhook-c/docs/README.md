# EthHook - C Implementation

A high-performance, production-grade Ethereum webhook service written in modern C (C11/C17/C23). This is a complete reimplementation of the original Rust-based EthHook system, focusing on security, reliability, and optimal performance.

## Overview

EthHook listens to blockchain events via WebSocket connections, processes them through configurable endpoints, and delivers webhook notifications to your applications with built-in retry logic and circuit breakers.

### Key Features

- **Modern C Standards**: C11 base with C17/C23 features
- **Battle-tested Libraries**: libevent, libwebsockets, hiredis, jansson, libcurl, libmicrohttpd, libjwt
- **Production-Ready**: Circuit breakers, exponential backoff, structured logging, graceful shutdown
- **High Performance**: Async I/O, thread pools, arena allocators, lock-free atomics
- **Lightweight**: SQLite-only (no PostgreSQL/ClickHouse), Alpine-based Docker images
- **Secure**: JWT authentication, HMAC-SHA256 webhook signatures, non-root containers

## Architecture

### Services

1. **Event Ingestor** (`ethhook-ingestor`)
   - WebSocket client connecting to Ethereum nodes
   - One worker thread per blockchain chain
   - libevent async I/O event loop
   - Circuit breaker for connection resilience
   - Publishes events to Redis streams

2. **Message Processor** (`ethhook-processor`)
   - Consumes events from Redis streams
   - Matches events against endpoint filters (address, topics)
   - SQLite-based endpoint matching
   - Queues delivery requests

3. **Webhook Delivery** (`ethhook-delivery`)
   - HTTP client with libcurl
   - Retry logic with exponential backoff
   - Per-endpoint circuit breakers
   - HMAC-SHA256 webhook signatures

4. **Admin API** (`ethhook-admin-api`)
   - HTTP REST API with libmicrohttpd
   - JWT-based authentication
   - User, application, endpoint, event management
   - SQLite database queries

### Data Flow

```
Blockchain Node (WebSocket)
    ↓
Event Ingestor
    ↓
Redis Streams
    ↓
Message Processor (SQLite matching)
    ↓
Redis Delivery Queue
    ↓
Webhook Delivery (HTTP POST)
    ↓
Your Application
```

## Prerequisites

### System Dependencies (Alpine Linux)

```bash
apk add --no-cache \
    build-base \
    cmake \
    git \
    pkgconfig \
    libevent-dev \
    libwebsockets-dev \
    hiredis-dev \
    jansson-dev \
    curl-dev \
    libmicrohttpd-dev \
    libjwt-dev \
    openssl-dev \
    sqlite-dev
```

### Ubuntu/Debian

```bash
sudo apt-get install -y \
    build-essential \
    cmake \
    git \
    pkg-config \
    libevent-dev \
    libwebsockets-dev \
    libhiredis-dev \
    libjansson-dev \
    libcurl4-openssl-dev \
    libmicrohttpd-dev \
    libjwt-dev \
    libssl-dev \
    libsqlite3-dev
```

## Building

```bash
# Clone repository
cd ethhook-c

# Create build directory
mkdir build && cd build

# Configure with CMake
cmake -DCMAKE_BUILD_TYPE=Release ..

# Build all targets
make -j$(nproc)

# Run tests (optional)
make test

# Install binaries
sudo make install
```

### Build Options

- `-DBUILD_TESTS=ON`: Enable unit tests (requires Check framework)
- `-DCMAKE_BUILD_TYPE=Release`: Optimized build with LTO
- `-DCMAKE_BUILD_TYPE=Debug`: Debug build with symbols

## Configuration

Create a TOML configuration file (see `config/config.example.toml`):

```toml
# Database
database_url = "ethhook.db"

# Redis
redis_host = "localhost"
redis_port = 6379

# JWT (for Admin API)
jwt_secret = "your-secret-key-change-me"
jwt_expiry_hours = 24

# Admin API
port = 3000
```

Environment variables override config file:
- `DATABASE_URL`
- `REDIS_HOST`
- `REDIS_PORT`
- `JWT_SECRET`
- `PORT`

## Running

### Direct Execution

```bash
# Start services
./ethhook-ingestor config.toml
./ethhook-processor config.toml
./ethhook-delivery config.toml
./ethhook-admin-api config.toml
```

### Docker Compose

```bash
cd docker
docker-compose up -d
```

Services exposed:
- Admin API: http://localhost:3000
- Redis: localhost:6379

### Docker Individual Services

```bash
# Build images
docker build -f docker/Dockerfile.ingestor -t ethhook-ingestor .
docker build -f docker/Dockerfile.processor -t ethhook-processor .
docker build -f docker/Dockerfile.delivery -t ethhook-delivery .
docker build -f docker/Dockerfile.admin-api -t ethhook-admin-api .

# Run containers
docker run -d --name ethhook-admin-api \
  -p 3000:3000 \
  -e DATABASE_URL=/data/ethhook.db \
  -e JWT_SECRET=your-secret \
  -v ethhook-data:/data \
  ethhook-admin-api
```

## API Usage

### Authentication

```bash
# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@ethhook.com","password":"password123"}'

# Response
{"token":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."}
```

Use token in subsequent requests:

```bash
curl http://localhost:3000/api/users \
  -H "Authorization: Bearer <token>"
```

### Endpoints

- `POST /api/auth/login` - Authenticate user
- `GET /api/users` - List users (admin only)
- `GET /api/applications` - List applications
- `POST /api/applications` - Create application
- `GET /api/endpoints` - List endpoints
- `POST /api/endpoints` - Create endpoint
- `GET /api/events` - List events
- `GET /api/deliveries` - List delivery attempts

See `docs/API.md` for complete documentation.

## Development

### Code Structure

```
ethhook-c/
├── CMakeLists.txt           # Build configuration
├── include/ethhook/         # Public headers
│   ├── common.h             # Core types, logging, errors
│   ├── ingestor.h           # Event ingestor API
│   ├── processor.h          # Message processor API
│   ├── delivery.h           # Webhook delivery API
│   └── admin_api.h          # Admin API
├── src/
│   ├── common/              # Shared implementations
│   │   ├── logging.c        # Structured logging
│   │   ├── error.c          # Error handling
│   │   ├── arena.c          # Arena allocator
│   │   ├── circuit_breaker.c # Circuit breaker
│   │   ├── config.c         # TOML config parser
│   │   └── database.c       # SQLite wrapper
│   ├── ingestor/            # Event ingestor
│   │   ├── main.c
│   │   ├── websocket.c
│   │   ├── worker.c
│   │   └── redis_publisher.c
│   ├── processor/           # Message processor
│   │   ├── main.c
│   │   ├── redis_consumer.c
│   │   ├── matcher.c
│   │   └── filter.c
│   ├── delivery/            # Webhook delivery
│   │   ├── main.c
│   │   ├── http_client.c
│   │   ├── retry.c
│   │   └── worker.c
│   └── admin-api/           # Admin API
│       ├── main.c
│       ├── handlers.c
│       ├── auth.c
│       ├── routes.c
│       └── json_response.c
├── tests/                   # Unit tests
├── docker/                  # Docker configurations
└── docs/                    # Documentation
```

### Memory Management

- **Arena Allocators**: Per-thread arenas for fast allocation
- **Lock-Protected**: Thread-safe with mutexes
- **Automatic Cleanup**: Arena reset/destroy
- **No Memory Leaks**: Valgrind-tested

### Threading Model

- **One Thread Per Chain**: Event ingestor
- **Worker Pool**: Message processor and delivery
- **Event Loop**: libevent per thread
- **Structured Concurrency**: pthread with proper joins
- **Atomic Operations**: C11 stdatomic.h for metrics

### Error Handling

- **Error Codes**: Enum with descriptive values
- **Error Context**: File, line, message
- **Structured Logging**: Syslog + stderr
- **Circuit Breakers**: Automatic failure handling

## Performance

### Benchmarks (Preliminary)

- Event ingestion: ~10,000 events/sec
- Event processing: ~5,000 matches/sec
- Webhook delivery: ~1,000 deliveries/sec

### Optimization

- Compile with `-O3 -march=native`
- Enable LTO (Link Time Optimization)
- Use arena allocators for hot paths
- Minimize lock contention with atomics
- Connection pooling for HTTP

## Security

- JWT authentication with HS256
- HMAC-SHA256 webhook signatures
- Non-root Docker containers
- No hardcoded secrets
- Input validation
- SQL injection prevention (parameterized queries)

## Production Deployment

1. **Configure secrets**: Set `JWT_SECRET` via environment
2. **Database backups**: Regular SQLite backup with `.backup` command
3. **Monitoring**: Structured logs to syslog, metrics via atomics
4. **Resource limits**: Docker memory/CPU limits
5. **Health checks**: HTTP health endpoints
6. **Graceful shutdown**: SIGTERM/SIGINT handlers
7. **Circuit breakers**: Automatic failure handling

## Comparison with Rust Version

| Feature | Rust Version | C Version |
|---------|--------------|-----------|
| Database | PostgreSQL + SQLite + ClickHouse | SQLite only |
| Language | Rust (tokio, axum) | C (libevent, libcurl) |
| Binary Size | ~20MB | ~2MB |
| Memory Usage | ~50MB base | ~5MB base |
| Startup Time | ~200ms | ~50ms |
| Dependencies | Cargo crates | System libraries |
| Safety | Memory-safe | Manual management |

## Troubleshooting

### Build Issues

**Missing libraries**:
```bash
# Check installed packages
pkg-config --list-all | grep -E 'libevent|websockets|hiredis'

# Install missing dependencies
apk add <package>-dev
```

**CMake errors**:
```bash
# Clear build directory
rm -rf build && mkdir build && cd build
cmake ..
```

### Runtime Issues

**Connection refused**:
- Check Redis is running: `redis-cli ping`
- Check ports: `netstat -tlnp | grep 6379`

**Database locked**:
- Enable WAL mode (done automatically)
- Check file permissions
- Verify only one writer

**WebSocket errors**:
- Validate WebSocket URL
- Check API key/credentials
- Monitor circuit breaker state

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Run tests: `make test`
5. Push to branch: `git push origin feature/amazing-feature`
6. Open Pull Request

## License

See `../LICENSE` for details.

## Acknowledgments

- Original Rust implementation: https://github.com/ethhook/ethhook
- libevent: https://libevent.org/
- libwebsockets: https://libwebsockets.org/
- hiredis: https://github.com/redis/hiredis
- jansson: https://digip.org/jansson/

## Support

- GitHub Issues: https://github.com/ethhook/ethhook-c/issues
- Documentation: https://docs.ethhook.com
- Email: support@ethhook.com
