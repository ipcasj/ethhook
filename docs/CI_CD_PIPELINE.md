# CI/CD Pipeline Documentation

**Last Updated**: December 2024  
**Status**: ✅ Production Ready  
**GitHub Actions Workflow**: `.github/workflows/ci.yml`

## Overview

The EthHook project uses GitHub Actions for continuous integration and deployment. The CI pipeline runs automatically on pushes and pull requests to `main` and `develop` branches.

## Pipeline Architecture

```mermaid
graph TB
    A[Push/PR to main/develop] --> B[Lint & Format Check]
    A --> C[Unit & Integration Tests]
    A --> D[E2E Tests]
    A --> E[Build All Services]
    A --> F[SQLx Check]
    A --> G[Security Audit]
    A --> H[Code Coverage]
    A --> I[Docker Build Test]
    
    B --> J[CI Summary]
    C --> J
    D --> J
    E --> J
    F --> J
    G --> J
    H --> J
    I --> J
    
    J --> K{All Passed?}
    K -->|Yes| L[✅ CI Success]
    K -->|No| M[❌ CI Failed]
```

## CI Jobs

### 1. **Lint and Format Check** 🧹

**Purpose**: Ensure code quality and consistent formatting

**Steps**:

- Check code formatting with `cargo fmt`
- Run `cargo clippy` with strict warnings (`-D warnings`)

**When it fails**:

- Run `cargo fmt --all` locally to fix formatting
- Run `cargo clippy --all-targets --all-features -- -D warnings` to see warnings
- Fix clippy warnings or add `#[allow(...)]` with justification

**Duration**: ~2 minutes

---

### 2. **Unit & Integration Tests** 🧪

**Purpose**: Verify core functionality and integration with databases

**Infrastructure**:

- PostgreSQL 15 (Docker service)
- Redis 7 (Docker service)

**Steps**:

1. Run database migrations
2. Execute unit tests: `cargo test --workspace --lib --bins`
3. Execute integration tests: `cargo test -p ethhook-admin-api --test integration_test -- --include-ignored`

**Environment Variables**:

```bash
DATABASE_URL=postgresql://ethhook:password@localhost:5432/ethhook
REDIS_URL=redis://localhost:6379
JWT_SECRET=test-secret-for-ci
```

**When it fails**:

- Check test logs for specific failures
- Run tests locally: `cargo test --workspace`
- Ensure database schema is up-to-date
- Check Redis connection if stream tests fail

**Duration**: ~5 minutes

---

### 3. **End-to-End Tests** 🚀 **[NEW]**

**Purpose**: Validate entire system pipeline from event ingestion to webhook delivery

**Infrastructure**:

- PostgreSQL 15 (Docker service)
- Redis 7 (Docker service)

**Tests Executed**:

1. **Redis Stream Consumption** - Validates Message Processor consumes from Redis streams
2. **Full Pipeline** - Tests Event Ingestor → Message Processor → Webhook Delivery
3. **Consumer Groups** - Verifies XREADGROUP/XACK consumer group functionality
4. **Service Recovery** - Tests crash recovery and state persistence
5. **Batch Processing** - Validates processing 10 events in batches

**Command**:

```bash
cargo test --test e2e_tests -- --ignored --test-threads=1
```

**Environment Variables**:

```bash
DATABASE_URL=postgresql://ethhook:password@localhost:5432/ethhook
REDIS_HOST=localhost
REDIS_PORT=6379
JWT_SECRET=test-secret-for-e2e-tests
```

**Key Features**:

- ✅ Runs with `--test-threads=1` to prevent test interference
- ✅ Uses `--ignored` flag (E2E tests marked with `#[ignore]`)
- ✅ 10-minute timeout to prevent hanging
- ✅ Full service lifecycle testing

**When it fails**:

- Check logs for specific test failures
- Run E2E tests locally: `./scripts/run_e2e_tests.sh`
- Ensure PostgreSQL and Redis are healthy
- Check for port conflicts (5432, 6379)
- Verify database migrations ran successfully
- Look for timeout issues (services not starting)

**Common Issues**:

- **Timeout**: Service took too long to start → Check resource constraints
- **Database errors**: Migration failed → Check migration files
- **Redis errors**: Stream not created → Check Redis connection
- **XACK failures**: Consumer group issue → Check consumer group setup

**Duration**: ~2-3 minutes

---

### 4. **Build All Services** 🔨

**Purpose**: Ensure all services compile successfully

**Strategy**:

- Matrix build for multiple targets
- Currently: `x86_64-unknown-linux-gnu`
- Future: `x86_64-unknown-linux-musl` (Alpine Linux)

**Artifacts**:

- `event-ingestor` binary
- `ethhook-message-processor` binary
- `ethhook-webhook-delivery` binary
- `ethhook-admin-api` binary

**When it fails**:

- Check compilation errors
- Ensure all dependencies are available
- Verify target toolchain is installed
- Run `cargo build --workspace --release` locally

**Duration**: ~10 minutes (release build)

---

### 5. **SQLx Check** 🗄️

**Purpose**: Verify SQLx offline mode works (compile-time query verification)

**Steps**:

1. Verify `.sqlx/` directory exists
2. Check `.cargo/config.toml` has SQLX_OFFLINE=true
3. Build with offline mode (no database needed)

**When it fails**:

- Run `cargo sqlx prepare` to regenerate query metadata
- Commit `.sqlx/` directory changes
- Ensure all SQL queries are valid

**Duration**: ~3 minutes

---

### 6. **Security Audit** 🔒

**Purpose**: Detect known security vulnerabilities in dependencies

**Tool**: `cargo-audit`

**Allowed Vulnerabilities** (with justification):

- `RUSTSEC-2024-0437`: protobuf 2.28.0 (prometheus 0.13 dependency, breaking changes in 0.14)
- `RUSTSEC-2023-0071`: rsa in sqlx-mysql (we use PostgreSQL, not MySQL)
- `RUSTSEC-2024-0436`: paste unmaintained (sqlx dependency)
- `RUSTSEC-2024-0370`: proc-macro-error unmaintained (validator dependency)

**When it fails**:

- Check vulnerability details
- Update affected dependencies if possible
- If no fix available, evaluate risk and add to ignore list with justification
- Document security decisions

**Duration**: ~1 minute

---

### 7. **Code Coverage** 📊

**Purpose**: Track test coverage across the codebase

**Tool**: `cargo-llvm-cov`

**Steps**:

1. Generate LCOV coverage report
2. Upload to Codecov (if token configured)

**When it fails**:

- Check if tests are failing (this job depends on tests passing)
- Verify LCOV generation command
- Codecov upload failure is non-blocking (`fail_ci_if_error: false`)

**Duration**: ~7 minutes

---

### 8. **Docker Build Test** 🐳

**Purpose**: Validate Docker configuration (when present)

**Steps**:

1. Check for Dockerfile or docker-compose.yml
2. Validate docker-compose configuration

**When it fails**:

- Check Docker syntax errors
- Validate docker-compose.yml with `docker compose config`
- Currently informational only (no Dockerfiles yet)

**Duration**: ~1 minute

---

### 9. **CI Summary** 📝

**Purpose**: Aggregate results from all jobs and provide clear pass/fail status

**Dependencies**: All previous jobs (lint, test, e2e-tests, build, sqlx-check, security-audit)

**Output**: GitHub Actions Summary with results table

**Example**:

```markdown
## CI Pipeline Results

| Job | Status |
|-----|--------|
| Lint | success |
| Unit & Integration Tests | success |
| E2E Tests | success |
| Build | success |
| SQLx Check | success |
| Security Audit | success |

✅ All CI checks passed!
```

**When it fails**: One or more required jobs failed - check individual job logs

---

## Environment Variables

### Required for All Jobs

- `CARGO_TERM_COLOR: always` - Colored output
- `RUST_BACKTRACE: 1` - Stack traces on panic
- `SQLX_OFFLINE: true` - Compile-time query verification

### Test-Specific

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` / `REDIS_HOST` / `REDIS_PORT` - Redis connection
- `JWT_SECRET` - Secret for JWT token generation

### Optional

- `CODECOV_TOKEN` - Codecov upload token (coverage job)

---

## Running CI Locally

### Prerequisites

```bash
# Install required tools
cargo install sqlx-cli --no-default-features --features postgres
cargo install cargo-llvm-cov
cargo install cargo-audit

# Start infrastructure
docker-compose up -d postgres redis
```

### Run Individual Job Steps

**Lint**:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

**Unit Tests**:

```bash
export DATABASE_URL=postgresql://ethhook:password@localhost:5432/ethhook
export REDIS_URL=redis://localhost:6379
export JWT_SECRET=test-secret

sqlx database create
sqlx migrate run
cargo test --workspace --lib --bins
```

**E2E Tests**:

```bash
export DATABASE_URL=postgresql://ethhook:password@localhost:5432/ethhook
export REDIS_HOST=localhost
export REDIS_PORT=6379
export JWT_SECRET=test-secret

cargo test --test e2e_tests -- --ignored --test-threads=1
```

**Build**:

```bash
cargo build --workspace --release
```

**SQLx Check**:

```bash
SQLX_OFFLINE=true cargo build --workspace
```

**Security Audit**:

```bash
cargo audit
```

**Coverage**:

```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

---

## Troubleshooting

### Common Issues

#### 1. **Flaky E2E Tests**

**Symptoms**: Tests pass locally but fail in CI

**Solutions**:

- Check for timing issues (services not fully started)
- Increase timeouts in tests
- Add health checks before running tests
- Check for resource constraints in GitHub Actions runners

#### 2. **Database Connection Failures**

**Symptoms**: "connection refused" or "database does not exist"

**Solutions**:

- Ensure PostgreSQL service is healthy
- Check health check configuration in workflow
- Verify DATABASE_URL is correct
- Wait for migrations to complete

#### 3. **Redis Connection Failures**

**Symptoms**: "connection refused" or "stream not found"

**Solutions**:

- Ensure Redis service is healthy
- Check REDIS_HOST and REDIS_PORT variables
- Verify Redis health check passes
- Check if streams are properly created

#### 4. **Build Cache Issues**

**Symptoms**: Unexpected compilation errors or old code behavior

**Solutions**:

- Clear GitHub Actions cache
- Run `cargo clean` locally
- Check cache key patterns in workflow
- Update Cargo.lock

#### 5. **Timeout in E2E Tests**

**Symptoms**: Tests exceed 10-minute timeout

**Solutions**:

- Optimize test setup (reuse connections, fixtures)
- Check for deadlocks or infinite loops
- Add more granular timeouts to individual tests
- Increase job timeout if legitimately needed

---

## Performance Optimization

### Caching Strategy

The CI uses aggressive caching to speed up builds:

1. **Cargo Registry** - Downloaded crates
2. **Cargo Index** - crates.io index
3. **Build Artifacts** - Compiled dependencies

**Cache Keys**: Based on `Cargo.lock` hash

### Typical Job Durations

| Job | First Run | Cached Run |
|-----|-----------|------------|
| Lint | 3 min | 2 min |
| Tests | 8 min | 5 min |
| E2E Tests | 5 min | 2-3 min |
| Build | 15 min | 10 min |
| SQLx Check | 5 min | 3 min |
| Security Audit | 2 min | 1 min |
| Coverage | 10 min | 7 min |
| **Total** | **~50 min** | **~30 min** |

---

## Migration Guide: Adding E2E Tests

This section documents the changes made to add E2E tests to CI (December 2024).

### What Changed

1. **New Job**: `e2e-tests`
   - Runs all E2E tests with `--ignored --test-threads=1`
   - Uses PostgreSQL and Redis services
   - 10-minute timeout
   - Proper environment variable setup

2. **Updated Summary**: Added E2E test results to CI summary

3. **Documentation**: This guide!

### Benefits

- ✅ **Confidence**: Full pipeline tested on every commit
- ✅ **Early Detection**: Catch integration issues before deployment
- ✅ **Regression Prevention**: All critical paths verified
- ✅ **Production Parity**: Tests run in production-like environment

### Migration Steps (For Future Reference)

1. Add E2E test job with services (PostgreSQL, Redis)
2. Configure environment variables (DATABASE_URL, REDIS_HOST, REDIS_PORT)
3. Run migrations before tests
4. Execute tests with proper flags (--ignored, --test-threads=1)
5. Update summary job dependencies
6. Update documentation

---

## Best Practices

### For Developers

1. **Run Tests Locally**: Before pushing, run full test suite

   ```bash
   ./scripts/run_all_tests.sh
   ```

2. **Check Lint**: Fix formatting and clippy warnings

   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Test E2E**: Verify E2E tests pass locally

   ```bash
   ./scripts/run_e2e_tests.sh
   ```

4. **Update SQLx**: If changing queries, regenerate metadata

   ```bash
   cargo sqlx prepare
   ```

5. **Check Security**: Run audit before adding dependencies

   ```bash
   cargo audit
   ```

### For Reviewers

1. **Check CI Status**: All jobs must pass
2. **Review Test Coverage**: Check coverage report
3. **Verify E2E**: Ensure E2E tests validate changes
4. **Check Security**: Review any new dependencies

### For Maintainers

1. **Monitor CI Duration**: Keep jobs under reasonable time limits
2. **Update Dependencies**: Regular security updates
3. **Review Ignored Vulnerabilities**: Quarterly review of allowed CVEs
4. **Optimize Caching**: Improve cache hit rates

---

## Future Improvements

### Short Term

- [ ] Add performance benchmarks to CI
- [ ] Implement parallel E2E test execution (with proper isolation)
- [ ] Add Dockerfile and container image builds
- [ ] Set up automatic dependency updates (Dependabot/Renovate)

### Medium Term

- [ ] Add staging deployment on successful CI
- [ ] Implement canary deployments
- [ ] Add load testing to CI
- [ ] Set up automatic rollback on E2E test failures

### Long Term

- [ ] Multi-region deployment testing
- [ ] Chaos engineering tests
- [ ] Automated performance regression detection
- [ ] Production monitoring integration

---

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [SQLx Offline Mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)
- [E2E Test Documentation](../tests/README.md)
- [Testing Strategy](./TESTING_STRATEGY.md)

---

## Support

For CI issues or questions:

1. Check this documentation
2. Review job logs in GitHub Actions
3. Check related test documentation
4. Open an issue with CI logs attached

---

**Status**: ✅ CI pipeline fully operational with comprehensive E2E testing
