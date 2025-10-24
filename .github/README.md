# GitHub Actions CI/CD Setup

This directory contains the GitHub Actions workflows for the EthHook project.

## Workflows

### 1. CI Pipeline (`ci.yml`)

Runs on every push and pull request to `main` and `develop` branches.

**Jobs:**

- **Lint**: Code formatting and Clippy checks
- **Test**: Full test suite with PostgreSQL and Redis
- **Build**: Multi-platform builds (Linux GNU, Linux MUSL)
- **SQLx Check**: Verifies offline mode configuration
- **Security Audit**: Dependency vulnerability scanning
- **Coverage**: Code coverage reporting with Codecov
- **Docker Build**: Validates Docker configuration

**Requirements:**

- `.sqlx/` directory must be committed (SQLx offline mode)
- `.cargo/config.toml` must set `SQLX_OFFLINE=true`
- All tests must pass

### 2. Release Pipeline (`release.yml`)

Triggers on version tags (`v*.*.*`).

**Jobs:**

- **Create Release**: Creates GitHub release
- **Build Release**: Builds binaries for multiple platforms
- **Docker Release**: Builds and pushes Docker images

**Platform Support:**

- `x86_64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `x86_64-apple-darwin` (Intel Mac)
- `aarch64-apple-darwin` (Apple Silicon)

## Setup Instructions

### 1. Required Secrets

Configure these in GitHub Settings → Secrets and variables → Actions:

```
CODECOV_TOKEN       # Optional: For code coverage reporting
DOCKER_USERNAME     # Optional: For Docker Hub releases
DOCKER_PASSWORD     # Optional: For Docker Hub releases
```

### 2. First-Time Setup

1. **Enable Actions**:
   - Go to repository Settings → Actions → General
   - Allow all actions and reusable workflows

2. **Code Coverage** (Optional):
   - Sign up at https://codecov.io/
   - Add repository to Codecov
   - Copy token to GitHub secrets as `CODECOV_TOKEN`

3. **Docker Hub** (Optional for releases):
   - Create Docker Hub account
   - Add credentials as GitHub secrets

### 3. Running Locally

Before pushing, you can test similar to CI:

```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --workspace

# Admin API integration tests (requires PostgreSQL)
docker-compose up -d
cargo test -p ethhook-admin-api --test integration_test -- --include-ignored

# Build all services
cargo build --workspace --release

# Security audit
cargo install cargo-audit
cargo audit
```

## Troubleshooting

### "SQLX_OFFLINE but no cached data"

**Solution**: Regenerate query cache:
```bash
cargo sqlx prepare --workspace
git add .sqlx/
git commit -m "chore: update SQLx query cache"
```

### Tests failing in CI but passing locally

**Common causes**:
1. Missing `.sqlx/` files in git
2. Database migrations not applied
3. Environment variables not set
4. Redis/PostgreSQL not available

**Solution**: Check workflow logs and ensure all test dependencies are available.

### Docker build fails

**Common causes**:
1. Dockerfile doesn't exist (normal if not created yet)
2. Docker secrets not configured

**Solution**: The docker-build job validates config but won't fail if Dockerfiles don't exist.

## Performance

**Typical CI run time**: 5-8 minutes

**Optimization**:
- Cargo caching reduces build time by ~60%
- Parallel job execution
- SQLx offline mode eliminates database requirement for builds

## Maintenance

### Weekly

- Review security audit results
- Update dependencies if needed

### Monthly

- Review and update GitHub Actions versions
- Check for new Rust stable releases

### Per Release

- Update version in Cargo.toml files
- Create git tag
- Monitor release workflow
- Test downloaded release artifacts
