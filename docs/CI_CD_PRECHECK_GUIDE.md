# CI/CD Pre-Check Guide

## Problem: Why Do I See Errors After Each Push?

When you push to GitHub, multiple CI workflows run automatically:

1. **`ci.yml`** - Main CI pipeline (format, clippy, tests) ✅ **WORKING**
2. **`digitalocean-deploy.yml`** - Deployment to DigitalOcean ❌ **WAS FAILING** (now disabled)
3. **`release.yml`** - Release builds on version tags ✅ Only runs on tags

The DigitalOcean deployment workflow was running on **every push to main** but failing because:
- Missing required secrets (`DIGITALOCEAN_ACCESS_TOKEN`, `DO_REGISTRY`, etc.)
- Using non-existent GitHub Action (`digitalocean/app-action@v1.1.5`)

## Solution: What I Fixed

### 1. Disabled Auto-Deploy Workflow

Changed `.github/workflows/digitalocean-deploy.yml` to only run manually:

```yaml
on:
  workflow_dispatch:  # Manual trigger only - not on every push
```

Now it won't fail on every push. When you're ready to deploy:
1. Configure the required secrets in GitHub repository settings
2. Manually trigger the workflow from Actions tab

### 2. Created Comprehensive Pre-Push Check

New script: **`scripts/ci-check.sh`**

This runs the same checks as GitHub Actions **locally** before you push:
- ✅ Format check (`cargo fmt`)
- ✅ Clippy lints (all warnings as errors)
- ✅ Unit tests
- ✅ Debug build
- ✅ SQLX offline mode check (like CI)
- ✅ Security audit (non-blocking)
- ✅ Code quality checks (println!, unwrap(), TODOs)

## How to Use: Pre-Push Workflow

### Recommended Workflow

Before pushing to GitHub, run:

```bash
# Run all CI checks locally
./scripts/ci-check.sh
```

This catches issues **before** they reach GitHub, saving time and avoiding failed CI runs.

### What Each Check Does

1. **Format Check**: Ensures code is formatted consistently
   - Runs: `cargo fmt --all -- --check`
   - Fixes: `cargo fmt --all`

2. **Clippy Lints**: Catches common mistakes and bad patterns
   - Runs: `cargo clippy --all-targets --all-features -- -D warnings`
   - Fixes: `cargo clippy --fix --allow-dirty` (automatic)
   - Manual fixes: Address each warning based on clippy's suggestions

3. **Unit Tests**: Runs all unit and integration tests
   - Runs: `cargo test --workspace --lib --bins`
   - Fixes: Fix failing tests in code

4. **Debug Build**: Ensures code compiles
   - Runs: `cargo build --workspace`
   - Fixes: Fix compilation errors

5. **SQLX Offline Check**: Ensures CI can compile without database
   - Runs: `SQLX_OFFLINE=true cargo check --workspace`
   - Fixes: `cargo sqlx prepare --workspace` (regenerate .sqlx/ cache)

6. **Security Audit**: Checks for known vulnerabilities
   - Runs: `cargo audit`
   - Fixes: Update dependencies with `cargo update`
   - Note: Often transitive dependencies (from leptos, sqlx) - not blocking

### Quick Fixes

If `ci-check.sh` fails:

```bash
# Fix formatting
cargo fmt --all

# Auto-fix clippy warnings
cargo clippy --fix --allow-dirty --allow-staged

# Run tests to see what's broken
cargo test --workspace

# Regenerate SQLX cache if queries changed
cargo sqlx prepare --workspace
```

## Integration with Git Hooks

### Current Setup

The repository already has a pre-push hook at `.git/hooks/pre-push` that runs:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace --lib --bins`
- `cargo audit` (non-blocking)

This is good, but you can optionally replace it with the more comprehensive check.

### Optional: Use ci-check.sh as Pre-Push Hook

If you want more thorough checking:

```bash
# Edit .git/hooks/pre-push
cat > .git/hooks/pre-push << 'EOF'
#!/bin/sh
# Run comprehensive CI checks before push
exec ./scripts/ci-check.sh
EOF

chmod +x .git/hooks/pre-push
```

⚠️ **Note**: This will make pushes slower (30-60 seconds) but catches more issues.

## GitHub Actions Workflows

### ci.yml (Main CI Pipeline)

**Triggers**: Push to `main` or `develop`, and pull requests

**Jobs**:
1. **Lint**: Format and clippy checks
2. **Test**: Unit tests with PostgreSQL and Redis services
3. **E2E Tests**: Full integration tests

**Status**: ✅ **Working** (after SQLX offline cache + CI timing fixes)

### digitalocean-deploy.yml (Deployment)

**Triggers**: Manual only (`workflow_dispatch`)

**Status**: 🔒 **Disabled** until you configure:

Required GitHub Secrets:
- `DIGITALOCEAN_ACCESS_TOKEN` - Your DO API token
- `DIGITALOCEAN_USERNAME` - Your DO username/email
- `DO_REGISTRY` - Your DO container registry name

When ready to deploy:
1. Go to: Settings → Secrets and variables → Actions
2. Add the required secrets
3. Manually trigger: Actions tab → DigitalOcean Deploy → Run workflow

### release.yml (Release Builds)

**Triggers**: Git tags matching `v*.*.*` (e.g., `v1.0.0`)

**Jobs**: Builds release binaries for multiple platforms

**Status**: ✅ **Working** (only runs on version tags)

## Best Practices

### Before Every Push

```bash
# 1. Make your changes
git add .

# 2. Run CI checks locally
./scripts/ci-check.sh

# 3. If all passes, commit and push
git commit -m "feat: Your feature"
git push origin main
```

### If CI Fails on GitHub

1. **Don't panic** - Check which job failed in the Actions tab
2. **Pull the logs** - Click on the failed job to see detailed logs
3. **Reproduce locally** - Run the same command from the CI workflow
4. **Fix and test** - Fix the issue and run `./scripts/ci-check.sh`
5. **Push again** - The CI should pass now

### Common CI Failures

#### Format Check Failed

```bash
# Fix: Format your code
cargo fmt --all

# Commit the formatting changes
git add -u
git commit --amend --no-edit
git push --force
```

#### Clippy Failed

```bash
# Fix: Auto-fix clippy warnings
cargo clippy --fix --allow-dirty --allow-staged

# Or manually fix based on warnings
cargo clippy --all-targets --all-features

# Commit fixes
git add -u
git commit -m "fix: Resolve clippy warnings"
git push
```

#### Tests Failed

```bash
# Run tests locally to see the failure
cargo test --workspace

# Fix the failing test
# Then commit and push
git add .
git commit -m "fix: Fix failing test"
git push
```

#### SQLX Compilation Error (CI only)

```bash
# Regenerate SQLX offline cache
DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook" \
  cargo sqlx prepare --workspace

# Commit the updated .sqlx/*.json files
git add .sqlx/
git commit -m "fix: Update SQLX offline query cache"
git push
```

## Monitoring CI Status

### GitHub Actions Badge

Add to README.md:

```markdown
![CI](https://github.com/yourorg/ethhook/workflows/CI/badge.svg)
```

This shows the current CI status on your README.

### Check CI Status

Before pushing:

```bash
# Check if previous CI run passed
gh run list --workflow=ci.yml --limit 1
```

After pushing:

```bash
# Watch CI run in real-time
gh run watch
```

## Summary

**Problem Solved**:
- ✅ Disabled failing DigitalOcean deployment workflow
- ✅ Created comprehensive local CI check script
- ✅ Documented how to avoid CI failures

**Your New Workflow**:
1. Make changes
2. Run `./scripts/ci-check.sh`
3. Fix any issues it reports
4. Push with confidence ✨

**Result**: No more surprise CI failures after pushing!

## Additional Resources

- **Main CI workflow**: `.github/workflows/ci.yml`
- **Local CI check**: `./scripts/ci-check.sh`
- **Pre-push hook**: `.git/hooks/pre-push`
- **GitHub Actions docs**: <https://docs.github.com/en/actions>
- **Cargo commands reference**: <https://doc.rust-lang.org/cargo/commands/>
