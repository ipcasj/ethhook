# Tomorrow Morning Checklist (Day 2 - October 4, 2025)

**Time needed**: ~30 minutes setup, then start coding  
**Goal**: Get API keys, verify environment, start implementing Common crate

---

## ☕ Before You Code (30 minutes)

### Step 1: Get Alchemy API Keys (10 minutes)

1. **Go to**: [https://www.alchemy.com/](https://www.alchemy.com/)
2. **Sign up**: Use GitHub or Google (free)
3. **Create 4 apps**:
   - Click "+ Create App"
   - App 1: Name: "EthHook-Ethereum", Chain: Ethereum Mainnet, Network: Mainnet
   - App 2: Name: "EthHook-Arbitrum", Chain: Arbitrum, Network: Arbitrum One
   - App 3: Name: "EthHook-Optimism", Chain: Optimism, Network: Optimism Mainnet
   - App 4: Name: "EthHook-Base", Chain: Base, Network: Base Mainnet
4. **Copy API keys**:
   - Click each app
   - Click "API Key" button
   - Copy the key (looks like: `abc123xyz...`)
   - Paste into a text file temporarily

**You should have**: 4 API keys (one per chain)

---

### Step 2: Get Infura Project ID (10 minutes)

1. **Go to**: [https://www.infura.io/](https://www.infura.io/)
2. **Sign up**: Use email (free)
3. **Create project**:
   - Click "Create New Project"
   - Name: "EthHook"
   - Click "Create"
4. **Enable networks**:
   - In project dashboard, click "Settings"
   - Under "Endpoints", enable:
     - [x] Ethereum Mainnet
     - [x] Arbitrum One
     - [x] Optimism
     - [x] Base
   - Save changes
5. **Copy Project ID**:
   - In project overview, copy "Project ID"
   - Paste into text file

**You should have**: 1 Project ID (used for all chains)

---

### Step 3: Update .env File (5 minutes)

```bash
# Open .env file
cd ~/rust_projects/capstone0
cp .env.example .env  # if you haven't already
code .env  # or nano .env

# Replace these values with your actual keys:

# Ethereum
ETH_RPC_WS=wss://eth-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_KEY_HERE
ETH_RPC_HTTP=https://eth-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_KEY_HERE
ETH_RPC_WS_BACKUP=wss://mainnet.infura.io/ws/v3/YOUR_INFURA_PROJECT_ID_HERE
ETH_RPC_HTTP_BACKUP=https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID_HERE

# Arbitrum (use Arbitrum API key from Alchemy)
ARBITRUM_RPC_WS=wss://arb-mainnet.g.alchemy.com/v2/YOUR_ARBITRUM_KEY_HERE
ARBITRUM_RPC_HTTP=https://arb-mainnet.g.alchemy.com/v2/YOUR_ARBITRUM_KEY_HERE
ARBITRUM_RPC_WS_BACKUP=wss://arbitrum-mainnet.infura.io/ws/v3/YOUR_INFURA_PROJECT_ID_HERE
ARBITRUM_RPC_HTTP_BACKUP=https://arbitrum-mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID_HERE

# Optimism (use Optimism API key from Alchemy)
OPTIMISM_RPC_WS=wss://opt-mainnet.g.alchemy.com/v2/YOUR_OPTIMISM_KEY_HERE
OPTIMISM_RPC_HTTP=https://opt-mainnet.g.alchemy.com/v2/YOUR_OPTIMISM_KEY_HERE
OPTIMISM_RPC_WS_BACKUP=wss://optimism-mainnet.infura.io/ws/v3/YOUR_INFURA_PROJECT_ID_HERE
OPTIMISM_RPC_HTTP_BACKUP=https://optimism-mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID_HERE

# Base (use Base API key from Alchemy)
BASE_RPC_WS=wss://base-mainnet.g.alchemy.com/v2/YOUR_BASE_KEY_HERE
BASE_RPC_HTTP=https://base-mainnet.g.alchemy.com/v2/YOUR_BASE_KEY_HERE
BASE_RPC_WS_BACKUP=wss://base-mainnet.infura.io/ws/v3/YOUR_INFURA_PROJECT_ID_HERE
BASE_RPC_HTTP_BACKUP=https://base-mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID_HERE

# Change JWT secret!
JWT_SECRET=$(openssl rand -base64 48)  # Run this in terminal, paste output

# Save file
```

**Check**: No "YOUR_" placeholders should remain

---

### Step 4: Start Docker Services (2 minutes)

**IMPORTANT**: Make sure Docker Desktop is running first!

```bash
# Start Docker Desktop if not already running
open -a Docker
# Wait 30 seconds (watch for whale icon 🐳 in menu bar)

# Verify Docker is ready
docker info

# Start all services
docker compose up -d

# Verify they're running
docker compose ps
# Should show: postgres (healthy), redis (healthy), prometheus (up), grafana (up)

# If any service is unhealthy, check logs:
docker compose logs postgres
docker compose logs redis
```

**Check**: All 4 services should be running

---

### Step 5: Run Database Migrations (2 minutes)

```bash
# If you haven't installed sqlx-cli yet:
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Verify tables exist
docker exec -it ethhook-postgres psql -U ethhook -d ethhook -c "\dt"
# Should list 9 tables: users, applications, endpoints, events, delivery_attempts, etc.
```

**Check**: 9 tables should be created

---

### Step 6: Test Connections (3 minutes)

```bash
# Test PostgreSQL
docker exec -it ethhook-postgres psql -U ethhook -d ethhook -c "SELECT 1;"
# Should return: 1

# Test Redis
docker exec -it ethhook-redis redis-cli PING
# Should return: PONG

# Test Alchemy (replace with your actual key)
curl "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
# Should return: {"jsonrpc":"2.0","id":1,"result":"0x..."}

# Test Infura (replace with your project ID)
curl "https://mainnet.infura.io/v3/YOUR_PROJECT_ID" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
# Should return: {"jsonrpc":"2.0","id":1,"result":"0x..."}
```

**Check**: All 4 tests should return successful responses

---

## 🚀 Ready to Code

### Open Your Work Environment

```bash
# Open VS Code (or your preferred editor)
cd ~/rust_projects/capstone0
code .

# Open the roadmap in a browser tab for reference
open docs/3_WEEK_ROADMAP.md  # macOS
# or visit: file:///Users/igor/rust_projects/capstone0/docs/3_WEEK_ROADMAP.md
```

---

### Today's Task: Implement Common Crate

**Location**: `crates/common/`

**Time estimate**: 6-8 hours

**Subtasks**:

#### 1. Database Connection Pool (2 hours)

**Goal**: Create PostgreSQL connection pool using sqlx

**Files to create**:

- `crates/common/Cargo.toml` (add dependencies)
- `crates/common/src/lib.rs` (module exports)
- `crates/common/src/db.rs` (pool implementation)

**Key concepts** (Java → Rust):

```java
// Java: HikariCP
HikariConfig config = new HikariConfig();
config.setJdbcUrl("jdbc:postgresql://localhost:5432/ethhook");
config.setMaximumPoolSize(20);
HikariDataSource pool = new HikariDataSource(config);

// Rust: sqlx::PgPool
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;
```

**See**: `docs/3_WEEK_ROADMAP.md` lines 42-62 for detailed code examples

---

#### 2. Redis Client (2 hours)

**Goal**: Create Redis connection manager

**Files to create**:

- `crates/common/src/redis.rs`

**Key concepts**:

```java
// Java: Jedis
JedisPool pool = new JedisPool("localhost", 6379);
Jedis jedis = pool.getResource();
jedis.set("key", "value");

// Rust: redis-rs
let client = redis::Client::open("redis://localhost:6379")?;
let mut con = client.get_async_connection().await?;
con.set("key", "value").await?;
```

**See**: Roadmap for Stream (XADD, XREAD) and Queue (LPUSH, BRPOP) helpers

---

#### 3. Error Types (1 hour)

**Goal**: Define custom error types

**Files to create**:

- `crates/common/src/error.rs`

**Key concepts**:

```java
// Java: Exception hierarchy
class EthHookException extends Exception { }
class DatabaseException extends EthHookException { }
class RedisException extends EthHookException { }

// Rust: Error enum with thiserror
#[derive(Debug, thiserror::Error)]
pub enum EthHookError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}
```

---

#### 4. Auth Helpers (2 hours)

**Goal**: JWT and password hashing utilities

**Files to create**:

- `crates/common/src/auth.rs`

**Key functions**:

- `create_jwt(user_id, secret) -> Result<String>`
- `verify_jwt(token, secret) -> Result<Claims>`
- `hash_password(password) -> Result<String>`
- `verify_password(password, hash) -> Result<bool>`
- `sign_hmac(payload, secret) -> String`
- `verify_hmac(payload, signature, secret) -> bool`

**Key concepts**:

```java
// Java: JWT library
Algorithm algorithm = Algorithm.HMAC256("secret");
String token = JWT.create()
    .withSubject("user123")
    .withExpiresAt(new Date(System.currentTimeMillis() + 3600000))
    .sign(algorithm);

// Rust: jsonwebtoken
let claims = Claims {
    sub: "user123".to_string(),
    exp: (SystemTime::now() + Duration::from_secs(3600)).as_secs(),
};
let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret"))?;
```

---

#### 5. Logging Setup (1 hour)

**Goal**: Configure tracing-subscriber

**Files to create**:

- `crates/common/src/logging.rs`

**Key concepts**:

```java
// Java: Log4j
Logger logger = LogManager.getLogger(MyClass.class);
logger.info("User {} logged in", userId);
logger.error("Failed to connect", exception);

// Rust: tracing
use tracing::{info, error};
info!(user_id = %user_id, "User logged in");
error!(error = %e, "Failed to connect");
```

---

### Testing Your Work

After implementing each module, test it:

```bash
# Run tests for common crate
cargo test -p common

# Run tests for specific module
cargo test -p common db::  # Test database module
cargo test -p common redis::  # Test Redis module

# Check for compilation errors
cargo check -p common

# Format code
cargo fmt -p common

# Run clippy (linter)
cargo clippy -p common
```

---

### Taking Breaks

**Pomodoro technique** (recommended):

- Work: 50 minutes focused coding
- Break: 10 minutes (walk, stretch, coffee)
- After 4 cycles: Take 30-minute break

**6-8 hour day breakdown**:

- 9:00-11:00: Database + Redis (2 hours)
- 11:00-11:15: Break ☕
- 11:15-12:15: Error types (1 hour)
- 12:15-13:15: Lunch break 🍽️
- 13:15-15:15: Auth helpers (2 hours)
- 15:15-15:30: Break ☕
- 15:30-16:30: Logging setup (1 hour)
- 16:30-17:00: Testing & cleanup

**End of day**: Common crate complete! ✅

---

## 📝 Throughout the Day

### Keep Notes

Create `notes/day2.md`:

```markdown
# Day 2 Notes - October 4, 2025

## What I learned
- sqlx query macros vs raw queries
- Connection pool sizing decisions
- Redis async vs sync clients

## Challenges faced
- [Issue]: Async await confusion with tokio
- [Solution]: Read Tokio book chapter 2

## Tomorrow's questions
- How to handle connection pool exhaustion?
- Should we use prepared statements?

## Code snippets to remember
[paste useful code here]
```

---

### Ask Questions

**When stuck** (more than 15 minutes):

1. Read error message carefully (Rust errors are helpful!)
2. Check documentation (docs.rs)
3. Search GitHub issues
4. Ask in Rust Discord/Forum
5. Google: "rust sqlx [your question]"

**Remember**: Getting stuck is normal! You're learning.

---

## ✅ End-of-Day Checklist

Before stopping for the day:

- [ ] All code compiles (`cargo check -p common`)
- [ ] All tests pass (`cargo test -p common`)
- [ ] Code is formatted (`cargo fmt -p common`)
- [ ] No clippy warnings (`cargo clippy -p common`)
- [ ] Git commit with clear message
- [ ] Updated notes for the day
- [ ] Reviewed tomorrow's tasks (Day 3: Event Ingestor)

---

## 🎉 Success Criteria for Today

By end of Day 2, you should have:

✅ **Database module**: Can create pool, execute queries, handle errors  
✅ **Redis module**: Can connect, publish/subscribe, use streams  
✅ **Error module**: Custom error types with good messages  
✅ **Auth module**: JWT creation/validation, password hashing, HMAC  
✅ **Logging module**: Structured logging configured  

**Progress**: ~25% of Week 1 complete

---

## 💪 You've Got This

**Remember**:

- You have 15 years Java experience
- Rust is just new syntax for familiar concepts
- You have detailed roadmap with examples
- You can refer to docs anytime

**Java → Rust mappings are in your head**:

- Connection Pool → PgPool
- Jedis → redis-rs
- JWT library → jsonwebtoken
- Log4j → tracing
- Try/Catch → Result<T, E>

**Start coding!** 🚀

---

## Quick Reference Commands

```bash
# Build common crate
cargo build -p common

# Run tests
cargo test -p common

# Check for errors
cargo check -p common

# Format code
cargo fmt -p common

# Lint code
cargo clippy -p common

# Watch for changes (install with: cargo install cargo-watch)
cargo watch -x "check -p common"

# Run with debug logs
RUST_LOG=debug cargo test -p common -- --nocapture
```

---

## Good luck! See you at the end of Day 2! 🎯
