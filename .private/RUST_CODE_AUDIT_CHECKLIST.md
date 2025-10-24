# Rust Code Quality Audit Checklist

**Date**: October 21, 2025
**Purpose**: Ensure code follows Rust best practices before production deployment

---

## Quick Audit Commands

Run these before deploying:

```bash
# 1. Check for common mistakes and code smells
cargo clippy --all-targets --all-features -- -D warnings

# 2. Check code formatting
cargo fmt --all -- --check

# 3. Check for security vulnerabilities
cargo audit

# 4. Run all tests
cargo test --all

# 5. Check unused dependencies
cargo +nightly udeps --all-targets

# 6. Build in release mode (verify optimizations work)
cargo build --release
```

---

## Detailed Code Review Areas

### 1. Error Handling ⚠️

#### Check for:
```rust
// ❌ BAD: Using unwrap() in production
let value = some_option.unwrap();

// ❌ BAD: Using expect() without context
let value = result.expect("failed");

// ✅ GOOD: Proper error handling
let value = some_option.ok_or_else(|| anyhow!("Missing value"))?;

// ✅ GOOD: Expect with context
let value = result.expect("Failed to parse config: check DATABASE_URL format");
```

#### Search for issues:
```bash
# Find all unwrap() calls (review each one)
grep -rn "\.unwrap()" crates/ --include="*.rs"

# Find all expect() calls (ensure they have good messages)
grep -rn "\.expect(" crates/ --include="*.rs"

# Find all panic! calls (should be rare)
grep -rn "panic!" crates/ --include="*.rs"
```

#### What to fix:
- Replace `.unwrap()` with `?` operator
- Add context to `.expect()` messages
- Remove `panic!()` from library code

---

### 2. Resource Cleanup 🧹

#### Check for:
```rust
// ❌ BAD: No explicit cleanup
async fn process() {
    let file = File::open("data.txt")?;
    // If error occurs here, file not closed
    do_something()?;
}

// ✅ GOOD: RAII or explicit cleanup
async fn process() -> Result<()> {
    let _file = File::open("data.txt")?;
    do_something()?;
    // File automatically closed when _file goes out of scope
    Ok(())
}

// ✅ GOOD: Drop guard
let connection = pool.get().await?;
defer! { connection.close(); }
```

#### Search for issues:
```bash
# Find unclosed files
grep -rn "File::open" crates/ --include="*.rs" | grep -v "?"

# Find manual resource management
grep -rn "\.close()" crates/ --include="*.rs"
```

---

### 3. Async/Await Usage 🔄

#### Check for:
```rust
// ❌ BAD: Blocking in async
async fn fetch_data() {
    std::thread::sleep(Duration::from_secs(1)); // BLOCKING!
}

// ✅ GOOD: Async sleep
async fn fetch_data() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// ❌ BAD: Unnecessary async
async fn add(a: i32, b: i32) -> i32 {
    a + b // No await, doesn't need to be async
}

// ✅ GOOD: Remove async if not needed
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

#### Search for issues:
```bash
# Find blocking operations in async code
grep -rn "std::thread::sleep" crates/ --include="*.rs"
grep -rn "std::fs::" crates/ --include="*.rs"

# Find async functions without .await
# (clippy will catch these)
```

---

### 4. Database Queries 🗄️

#### Check for:
```rust
// ❌ BAD: SQL injection risk
let query = format!("SELECT * FROM users WHERE id = {}", user_id);

// ✅ GOOD: Parameterized query
sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)

// ❌ BAD: N+1 query problem
for user in users {
    let posts = get_posts_for_user(user.id).await?;
}

// ✅ GOOD: Batch query
let user_ids: Vec<_> = users.iter().map(|u| u.id).collect();
let posts = get_posts_for_users(&user_ids).await?;
```

#### Search for issues:
```bash
# Find format!() with SQL (potential injection)
grep -rn 'format!.*SELECT\|INSERT\|UPDATE\|DELETE' crates/ --include="*.rs"

# Find direct string interpolation in queries
grep -rn '\$\|%' crates/ --include="*.rs" | grep -i 'select\|insert'
```

---

### 5. Clone() Usage 📋

#### Check for:
```rust
// ❌ BAD: Unnecessary clone
fn process(data: Vec<String>) {
    let copy = data.clone(); // Expensive!
    do_something(copy);
}

// ✅ GOOD: Use reference
fn process(data: &[String]) {
    do_something(data);
}

// ❌ BAD: Cloning in loop
for item in items {
    let cloned = item.clone(); // Clones every iteration
    process(cloned);
}

// ✅ GOOD: Clone once or use reference
let items_cloned = items.clone();
for item in items_cloned {
    process(item);
}
```

#### Search for issues:
```bash
# Find clone() calls (review each one)
grep -rn "\.clone()" crates/ --include="*.rs" | wc -l

# Check if clone is in hot path (loops)
grep -A5 -B5 "for " crates/ --include="*.rs" | grep "\.clone()"
```

---

### 6. Unwrap in Production Code 🚨

**CRITICAL**: Review EVERY unwrap() in production code

```bash
# Find all unwrap() calls
grep -rn "\.unwrap()" crates/ --include="*.rs" > unwraps.txt

# Count unwraps
wc -l unwraps.txt
```

#### Acceptable unwraps:
- In test code: `#[cfg(test)]`
- After explicit check: `if is_some() { .unwrap() }`
- Constants: `const VALUE: u32 = parse("123").unwrap();`

#### Unacceptable unwraps:
- In request handlers
- In async functions
- In loops
- Anywhere user input is involved

---

### 7. Memory Safety 🔒

#### Check for:
```rust
// ❌ BAD: Unsafe without documentation
unsafe {
    *ptr = value;
}

// ✅ GOOD: Document safety invariants
unsafe {
    // SAFETY: ptr is valid because we just allocated it above
    *ptr = value;
}

// ❌ BAD: Transmute without comment
let bytes: [u8; 4] = std::mem::transmute(value);

// ✅ GOOD: Use safe alternative
let bytes = value.to_ne_bytes();
```

#### Search for issues:
```bash
# Find all unsafe blocks
grep -rn "unsafe {" crates/ --include="*.rs"

# Find transmute usage
grep -rn "transmute" crates/ --include="*.rs"

# Find raw pointer usage
grep -rn "\*const\|\*mut" crates/ --include="*.rs"
```

---

### 8. Performance ⚡

#### Check for:
```rust
// ❌ BAD: String allocation in loop
for i in 0..1000 {
    let s = format!("Item {}", i); // Allocates every iteration
}

// ✅ GOOD: Reuse buffer
let mut buf = String::new();
for i in 0..1000 {
    buf.clear();
    write!(&mut buf, "Item {}", i)?;
}

// ❌ BAD: Unnecessary Vec collection
let count = users.iter().collect::<Vec<_>>().len();

// ✅ GOOD: Direct count
let count = users.iter().count();
```

#### Tools:
```bash
# Profile CPU usage
cargo flamegraph --bin admin-api

# Check for allocations
cargo build --release
DHAT_PROFILER=1 ./target/release/admin-api

# Benchmark critical paths
cargo bench
```

---

### 9. Logging & Debugging 📝

#### Check for:
```rust
// ❌ BAD: println! in production
println!("User logged in: {}", user_id);

// ✅ GOOD: Use tracing
tracing::info!("User logged in: user_id={}", user_id);

// ❌ BAD: Logging sensitive data
tracing::info!("Password: {}", password);

// ✅ GOOD: Never log secrets
tracing::info!("Password updated for user_id={}", user_id);

// ❌ BAD: Debug logs in hot path
for item in items {
    tracing::debug!("Processing: {:?}", item); // Too verbose
}

// ✅ GOOD: Log summaries
tracing::info!("Processing {} items", items.len());
```

#### Search for issues:
```bash
# Find println! in production code (should be in tests only)
grep -rn "println!" crates/ --include="*.rs" | grep -v "#\[cfg(test)\]"

# Find potential secret logging
grep -rn 'tracing.*password\|secret\|token\|key' crates/ --include="*.rs" -i
```

---

### 10. Dependencies 📦

#### Check for:
- Outdated dependencies
- Unused dependencies
- Known vulnerabilities
- License compatibility

```bash
# Check for outdated dependencies
cargo outdated

# Check for unused dependencies
cargo +nightly udeps --all-targets

# Check for security vulnerabilities
cargo audit

# Check dependency tree size
cargo tree | wc -l
cargo tree --depth=1
```

#### Update strategy:
```bash
# Update all dependencies (carefully!)
cargo update

# Test after updating
cargo test --all

# Check what changed
git diff Cargo.lock
```

---

## Automated Checks

### 1. Set up Clippy in CI

```yaml
# .github/workflows/ci.yml
- name: Clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
```

### 2. Set up Rustfmt in CI

```yaml
- name: Format check
  run: cargo fmt --all -- --check
```

### 3. Set up Audit in CI

```yaml
- name: Security audit
  run: cargo audit
```

---

## Manual Review Checklist

For each service (admin-api, event-ingestor, message-processor, webhook-delivery):

### Code Quality:
- [ ] No `.unwrap()` in request handlers
- [ ] No `.expect()` without context
- [ ] No `panic!()` in production code
- [ ] Error messages are helpful
- [ ] Resources properly cleaned up
- [ ] No memory leaks

### Performance:
- [ ] No unnecessary clones in hot paths
- [ ] Database queries optimized
- [ ] Connection pools configured
- [ ] Caching where appropriate
- [ ] Async properly used

### Security:
- [ ] No SQL injection
- [ ] No XSS vulnerabilities
- [ ] CORS configured correctly
- [ ] JWT secrets not hardcoded
- [ ] Rate limiting in place
- [ ] Input validation

### Observability:
- [ ] Proper logging levels
- [ ] No sensitive data in logs
- [ ] Metrics exported
- [ ] Tracing configured
- [ ] Error tracking

---

## Quick Fixes

### Fix 1: Replace unwrap with ?

```bash
# Find unwraps
rg "\.unwrap\(\)" -t rust

# For each unwrap:
# Before:
let value = some_result.unwrap();

# After:
let value = some_result?;
```

### Fix 2: Add context to expects

```bash
# Before:
let config = Config::from_env().expect("Failed");

# After:
let config = Config::from_env().expect(
    "Failed to load config: check DATABASE_URL environment variable"
);
```

### Fix 3: Remove unnecessary clones

```bash
# Before:
fn process(data: Vec<String>) {
    for item in data.clone() {
        // ...
    }
}

# After:
fn process(data: Vec<String>) {
    for item in data {
        // ...
    }
}
```

---

## Priority

### CRITICAL (Fix before deployment):
1. Remove all `.unwrap()` in request handlers
2. Fix any SQL injection vulnerabilities
3. Ensure secrets not logged
4. Fix resource leaks

### HIGH (Fix in first week):
1. Add context to all `.expect()`
2. Remove unnecessary `.clone()`
3. Optimize database queries
4. Add proper error handling

### MEDIUM (Fix in first month):
1. Improve logging
2. Add more tests
3. Optimize performance
4. Update dependencies

---

## Scoring

Run these commands and aim for these scores:

```bash
# Clippy warnings (should be 0)
cargo clippy --all-targets 2>&1 | grep warning | wc -l
# Target: 0 warnings

# Unsafe blocks (should be minimal)
grep -r "unsafe {" crates/ --include="*.rs" | wc -l
# Target: < 5

# Unwraps (should be minimal)
grep -r "\.unwrap()" crates/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l
# Target: < 20

# TODOs (track technical debt)
grep -r "TODO\|FIXME\|XXX" crates/ --include="*.rs" | wc -l
# Track: Document all TODOs

# Test coverage (aim high)
cargo tarpaulin --out Html
# Target: > 70%
```

---

## Resources

- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Clippy Lints**: https://rust-lang.github.io/rust-clippy/
- **Common Mistakes**: https://github.com/pretzelhammer/rust-blog
- **Performance Book**: https://nnethercote.github.io/perf-book/

---

**Next Steps**:

1. Run automated checks: `cargo clippy && cargo audit`
2. Review manual checklist for each service
3. Fix CRITICAL issues
4. Document HIGH priority issues for later
5. Create GitHub issues for MEDIUM priority

**Estimated Time**: 1-2 hours for full audit
