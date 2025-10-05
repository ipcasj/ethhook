# Quick Setup Guide - Getting Started with EthHook

**Last Updated**: October 3, 2025  
**Time to complete**: 30 minutes

This guide gets you from zero to running the EthHook development environment.

---

## Prerequisites

Before starting, ensure you have:

- ✅ **Rust 1.75+** installed ([rustup.rs](https://rustup.rs))
- ✅ **Docker Desktop** installed (for PostgreSQL, Redis)
- ✅ **Git** installed
- ✅ **Text editor** (VS Code recommended)

Check versions:

```bash
rust --version    # Should show 1.75.0 or higher
cargo --version   # Should show 1.75.0 or higher
docker --version  # Should show 20.0+ or higher
```

---

## Step 1: Clone Repository & Setup

```bash
# Navigate to your projects directory
cd ~/rust_projects/capstone0

# Verify files are present
ls -la
# You should see: Cargo.toml, docker-compose.yml, migrations/, crates/, etc.
```

---

## Step 2: Get Free RPC Provider API Keys

### Alchemy (Primary Provider) - RECOMMENDED

1. **Visit**: [https://www.alchemy.com/](https://www.alchemy.com/)
2. **Sign up**: Use GitHub or Google account (free)
3. **Create Apps**: Click "Create App" 4 times for:
   - **App 1**: Ethereum Mainnet
   - **App 2**: Arbitrum One
   - **App 3**: Optimism Mainnet
   - **App 4**: Base Mainnet
4. **Copy API Keys**: Click each app → "View Key" → Copy the API key
5. **Save them**: You'll need these in Step 4

**Free tier**: 300M compute units/month (enough for 100k+ events)

### Infura (Backup Provider) - Optional but Recommended

1. **Visit**: [https://www.infura.io/](https://www.infura.io/)
2. **Sign up**: Use email (free)
3. **Create Project**: Click "Create New Project"
4. **Enable Networks**: In project settings, enable:
   - Ethereum Mainnet
   - Arbitrum One
   - Optimism Mainnet
   - Base Mainnet
5. **Copy Project ID**: You'll use same project ID for all chains
6. **Save it**: You'll need this in Step 4

**Free tier**: 100k requests/day per endpoint

---

## Step 3: Start Infrastructure Services

**IMPORTANT**: Make sure Docker Desktop is running first!

```bash
# On macOS: Start Docker Desktop if not running
open -a Docker
# Wait 30 seconds for Docker to fully start (watch for whale icon in menu bar)

# Verify Docker is running
docker info
# Should show system info, not "Cannot connect to daemon" error

# Start PostgreSQL, Redis, Prometheus, Grafana
docker compose up -d

# Verify services are running
docker compose ps
# You should see 4 services: postgres, redis, prometheus, grafana

# Check logs if needed
docker compose logs -f postgres  # Ctrl+C to exit
docker compose logs -f redis
```

**Services now running**:

- PostgreSQL: `localhost:5432`
- Redis: `localhost:6379`
- Prometheus: `localhost:9090`
- Grafana: `localhost:3001`

---

## Step 4: Configure Environment Variables

```bash
# Copy example environment file
cp .env.example .env

# Open in your text editor
code .env  # or nano .env, vim .env, etc.
```

**Replace these values** (from Step 2):

```bash
# Ethereum Mainnet
ETH_RPC_WS=wss://eth-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_API_KEY_HERE
ETH_RPC_HTTP=https://eth-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_API_KEY_HERE
ETH_RPC_WS_BACKUP=wss://mainnet.infura.io/ws/v3/YOUR_INFURA_PROJECT_ID_HERE
ETH_RPC_HTTP_BACKUP=https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID_HERE

# Arbitrum One
ARBITRUM_RPC_WS=wss://arb-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_API_KEY_HERE
ARBITRUM_RPC_HTTP=https://arb-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_API_KEY_HERE
# ... (repeat for Optimism and Base)

# Change JWT secret to something secure!
JWT_SECRET=change-this-to-random-64-character-string-min-32-chars-required
```

**Generate secure JWT secret** (recommended):

```bash
# On macOS/Linux
openssl rand -base64 48

# Copy the output and paste it as JWT_SECRET value
```

**Save the file** (`.env`)

---

## Step 5: Run Database Migrations

```bash
# Install sqlx-cli if you don't have it
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Verify tables were created
docker exec -it ethhook-postgres psql -U ethhook -d ethhook -c "\dt"
# You should see 9 tables: users, applications, endpoints, events, etc.
```

---

## Step 6: Build the Project

```bash
# Build all crates (this will take 5-10 minutes first time)
cargo build

# Check for errors
# If you see errors about missing dependencies, run:
cargo update
cargo build
```

**Coffee break!** ☕ First build compiles 300+ dependencies.

---

## Step 7: Run Tests (Optional but Recommended)

```bash
# Run all tests
cargo test

# You should see:
# - Config tests passing
# - Domain model tests passing
# - (More tests will be added as we build services)
```

---

## Step 8: Verify Configuration

```bash
# Test database connection
docker exec -it ethhook-postgres psql -U ethhook -d ethhook -c "SELECT version();"

# Test Redis connection
docker exec -it ethhook-redis redis-cli PING
# Should respond: PONG

# Test Alchemy connection (replace with your API key)
curl "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
# Should respond with latest block number
```

---

## Step 9: Explore the Codebase

```bash
# Open the project in VS Code
code .

# Key files to review:
# 1. ARCHITECTURE.md - System design and architecture
# 2. crates/domain/src/ - Domain models (User, Application, Endpoint, etc.)
# 3. crates/config/src/lib.rs - Configuration management
```

---

## Troubleshooting

### Issue: "Cannot connect to Docker daemon" error

This means Docker Desktop isn't running.

```bash
# Start Docker Desktop
open -a Docker

# Wait 30 seconds, then verify it's running
docker info

# Look for whale icon 🐳 in macOS menu bar (top-right)
# If the icon is animated, Docker is still starting - wait a bit more
```

### Issue: Docker services won't start

```bash
# Check if ports are already in use
lsof -i :5432  # PostgreSQL
lsof -i :6379  # Redis

# If something is using them, stop those services or change ports in docker-compose.yml
```

### Issue: Database connection fails

```bash
# Check PostgreSQL is running
docker compose ps postgres

# Check logs
docker compose logs postgres

# Restart if needed
docker compose restart postgres
```

### Issue: RPC provider returns errors

```bash
# Verify API keys are correct (no extra spaces)
cat .env | grep ALCHEMY

# Test connection with curl (see Step 8)

# Common issues:
# - Forgot to replace YOUR_API_KEY with actual key
# - Copied key with extra spaces or quotes
# - Free tier limits exceeded (unlikely in first week)
```

### Issue: Cargo build fails

```bash
# Update Rust to latest version
rustup update

# Clean and rebuild
cargo clean
cargo build

# If specific dependency fails, check internet connection
# Some crates are large (tokio, sqlx, etc.)
```

---

## Quick Reference

### Common Commands

```bash
# Start all services
docker compose up -d

# Stop all services
docker compose down

# View logs
docker compose logs -f [service_name]

# Build Rust project
cargo build

# Run tests
cargo test

# Run specific service (once implemented)
cargo run --bin event-ingestor
cargo run --bin message-processor
cargo run --bin webhook-delivery
cargo run --bin admin-api

# Format code
cargo fmt

# Check for errors without building
cargo check

# Run with debug logging
RUST_LOG=debug cargo run --bin event-ingestor
```

### Useful Docker Commands

```bash
# Connect to PostgreSQL
docker exec -it ethhook-postgres psql -U ethhook -d ethhook

# Connect to Redis CLI
docker exec -it ethhook-redis redis-cli

# View database tables
docker exec -it ethhook-postgres psql -U ethhook -d ethhook -c "\dt"

# View Redis streams
docker exec -it ethhook-redis redis-cli XREAD COUNT 10 STREAMS events:raw 0
```

### Environment Variables Quick Check

```bash
# Verify .env file is loaded (when running services)
printenv | grep ETH_RPC

# Test Alchemy connection
curl "https://eth-mainnet.g.alchemy.com/v2/$(grep ETH_RPC_HTTP .env | cut -d'/' -f6)" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

---

## Resources

### EthHook Docs

- `ARCHITECTURE.md` - System design and architecture
- `README.md` - Project overview and quick start

### Getting Help

- **Issues**: [GitHub Issues](https://github.com/ipcasj/ethhook/issues)
- **Rust Questions**: [Rust Users Forum](https://users.rust-lang.org/)
- **Blockchain Questions**: [Ethereum Stack Exchange](https://ethereum.stackexchange.com/)
- **Email**: [ihorpetroff@gmail.com](mailto:ihorpetroff@gmail.com)

---

## Success Checklist

Before starting development, verify:

- [ ] Docker services running (postgres, redis, prometheus, grafana)
- [ ] Database migrations completed (9 tables created)
- [ ] `.env` file configured with real API keys
- [ ] JWT secret changed from default
- [ ] `cargo build` completes successfully
- [ ] `cargo test` passes all tests
- [ ] Can connect to PostgreSQL via Docker
- [ ] Can connect to Redis via Docker
- [ ] Alchemy API key works (tested with curl)
- [ ] Infura API key works (tested with curl)
- [ ] Reviewed ARCHITECTURE.md

**All checked?** 🎉 You're ready to start building!

---

**Questions or issues?** Review the troubleshooting section above.

**Ready to code?** Check [ARCHITECTURE.md](ARCHITECTURE.md) for system design and start building! 🚀
