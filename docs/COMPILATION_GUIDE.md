# Admin API - Compilation Guide

## Current Status

The Admin API code is complete but **requires PostgreSQL for compilation** due to SQLx's compile-time query verification feature.

### Why Does It Need a Database?

SQLx uses the `sqlx::query!()` macro which connects to your database at **compile time** to:
- Verify SQL syntax
- Check that column names exist
- Validate types match between SQL and Rust
- Catch errors before runtime

This is a powerful feature but requires a running database during `cargo build`.

---

## Solutions

You have **3 options** to compile the Admin API:

### Option 1: Start PostgreSQL with Docker (Recommended)

```bash
# 1. Start Docker Desktop (if not running)
open -a Docker

# Wait for Docker to start, then:

# 2. Start PostgreSQL
docker compose up -d postgres

# 3. Wait for PostgreSQL to be ready (5-10 seconds)
docker compose ps

# 4. Run migrations
export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
sqlx migrate run --source migrations

# 5. Build Admin API
cargo build -p ethhook-admin-api

# 6. (Optional) Stop PostgreSQL when done
docker compose down postgres
```

### Option 2: Use Existing PostgreSQL Instance

If you already have PostgreSQL installed locally:

```bash
# 1. Ensure PostgreSQL is running
brew services start postgresql@15  # macOS with Homebrew

# 2. Create database
createdb ethhook

# 3. Run migrations
export DATABASE_URL="postgresql://localhost:5432/ethhook"
sqlx migrate run --source migrations

# 4. Build Admin API
cargo build -p ethhook-admin-api
```

### Option 3: Use SQLx Offline Mode

This mode uses cached query data instead of connecting to a database:

```bash
# 1. First, you MUST complete Option 1 or 2 at least once
#    to generate the .sqlx/query-*.json files

# 2. Generate offline query data
cd crates/admin-api
cargo sqlx prepare --workspace
cd ../..

# 3. Now you can build without database
SQLX_OFFLINE=true cargo build -p ethhook-admin-api

# 4. Add to your .env for future builds
echo "SQLX_OFFLINE=true" >> .env
```

---

## Current Build Status

### ✅ Services That Compile Successfully

These services don't use SQLx macros and compile fine:

1. **Event Ingestor** - ✅ Compiles (1 warning)
2. **Webhook Delivery** - ✅ Compiles (6 warnings)
3. **Common Crate** - ✅ Compiles

Test them:
```bash
cargo build -p ethhook-event-ingestor
cargo build -p ethhook-webhook-delivery
cargo build -p ethhook-common
```

### ⚠️ Services That Need Database

These services use `sqlx::query!()` and need a database:

1. **Admin API** - ⚠️ Needs PostgreSQL
2. **Message Processor** - ⚠️ Needs PostgreSQL

---

## Quick Fix Summary

**If you just want to get everything working quickly:**

```bash
# Start Docker Desktop first, then:
docker compose up -d postgres redis
export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
sleep 10  # Wait for PostgreSQL to start
sqlx migrate run --source migrations
cargo build --workspace --release
```

This will:
1. Start PostgreSQL and Redis
2. Run all database migrations
3. Build all 4 services successfully

---

## Errors Fixed

The following non-database errors were already fixed:

### 1. ✅ Duplicate Module Declaration
- **File**: `crates/admin-api/src/lib.rs`
- **Issue**: `pub mod handlers;` was declared twice
- **Fix**: Removed duplicate line

### 2. ✅ Unsafe `env::set_var` in Tests
- **File**: `crates/admin-api/src/config.rs`
- **Issue**: In Rust 2024, `env::set_var` is unsafe
- **Fix**: Wrapped in `unsafe` block:
```rust
unsafe {
    env::set_var("DATABASE_URL", "postgresql://localhost/test");
    env::set_var("JWT_SECRET", "test-secret-key");
}
```

---

## Next Steps After Compilation

Once the Admin API compiles successfully:

1. **Run All Services**:
   ```bash
   # Terminal 1
   cargo run --release -p ethhook-event-ingestor
   
   # Terminal 2
   cargo run --release -p ethhook-message-processor
   
   # Terminal 3
   cargo run --release -p ethhook-webhook-delivery
   
   # Terminal 4
   cargo run --release -p ethhook-admin-api
   ```

2. **Test the API**:
   ```bash
   # Health check
   curl http://localhost:3000/api/v1/health
   
   # Register user
   curl -X POST http://localhost:3000/api/v1/auth/register \
     -H "Content-Type: application/json" \
     -d '{
       "email": "test@example.com",
       "password": "SecurePass123!",
       "name": "Test User"
     }'
   ```

3. **Run Integration Tests**:
   ```bash
   cargo test --workspace
   ```

---

## Warnings (Non-Critical)

The following warnings appear but don't prevent compilation:

- **Event Ingestor**: Unused field `ws_url` (1 warning)
- **Webhook Delivery**: Unused methods and fields (6 warnings)

These are helper methods/fields that may be used in future features. They're safe to ignore for now.

---

## Support

If you encounter issues:

1. **Check Docker is running**: `docker ps`
2. **Check PostgreSQL is ready**: `docker compose ps`
3. **Check logs**: `docker compose logs postgres`
4. **Verify connection**: `psql postgresql://ethhook:password@localhost:5432/ethhook -c '\l'`

If problems persist, see:
- [docs/ADMIN_API_IMPLEMENTATION.md](ADMIN_API_IMPLEMENTATION.md) - Full API documentation
- [SETUP_GUIDE.md](../SETUP_GUIDE.md) - General setup instructions
- [docker-compose.yml](../docker-compose.yml) - Docker configuration
