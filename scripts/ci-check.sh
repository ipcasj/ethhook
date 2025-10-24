#!/usr/bin/env bash
# Comprehensive CI checks - Run this before pushing to catch issues early
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must run from project root${NC}"
    exit 1
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}EthHook CI Pre-Check${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Track overall status
FAILED=0

# Function to run check and report status
run_check() {
    local name="$1"
    local cmd="$2"
    
    echo -e "${BLUE}▶ $name${NC}"
    if eval "$cmd"; then
        echo -e "${GREEN}✓ $name passed${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ $name failed${NC}"
        echo ""
        FAILED=1
        return 1
    fi
}

# 1. Format check
run_check "Format Check" "cargo fmt --all -- --check"

# 2. Clippy (all warnings as errors, like CI)
run_check "Clippy Lints" "cargo clippy --all-targets --all-features --workspace -- -D warnings"

# 3. Unit tests
run_check "Unit Tests" "cargo test --workspace --lib --bins"

# 4. Build check (debug mode)
run_check "Debug Build" "cargo build --workspace"

# 5. Check SQLX offline mode (CI uses this)
run_check "SQLX Offline Check" "SQLX_OFFLINE=true cargo check --workspace"

# 6. Security audit (allow failure but report)
echo -e "${BLUE}▶ Security Audit${NC}"
if cargo audit; then
    echo -e "${GREEN}✓ No security vulnerabilities found${NC}"
else
    echo -e "${YELLOW}⚠ Security audit found issues (not blocking)${NC}"
fi
echo ""

# 7. Check for common issues
echo -e "${BLUE}▶ Additional Checks${NC}"

# Check for println! in non-test code (should use tracing)
if grep -r "println!" crates/*/src/*.rs crates/*/src/**/*.rs 2>/dev/null | grep -v "test" | grep -v "example"; then
    echo -e "${YELLOW}⚠ Found println! statements (consider using tracing::info/debug/error)${NC}"
else
    echo -e "${GREEN}✓ No println! statements found${NC}"
fi

# Check for unwrap() in non-test code (should use proper error handling)
UNWRAPS=$(grep -r "\.unwrap()" crates/*/src/*.rs crates/*/src/**/*.rs 2>/dev/null | grep -v "test" | grep -v "example" | wc -l || echo "0")
if [ "$UNWRAPS" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Found $UNWRAPS .unwrap() calls (consider proper error handling)${NC}"
else
    echo -e "${GREEN}✓ No .unwrap() calls found${NC}"
fi

# Check for TODO/FIXME comments
TODOS=$(grep -r "TODO\|FIXME" crates/*/src/*.rs crates/*/src/**/*.rs 2>/dev/null | wc -l || echo "0")
if [ "$TODOS" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Found $TODOS TODO/FIXME comments${NC}"
else
    echo -e "${GREEN}✓ No TODO/FIXME comments${NC}"
fi

echo ""

# Final summary
echo -e "${BLUE}========================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo -e "${GREEN}Ready to push to GitHub${NC}"
    echo -e "${BLUE}========================================${NC}"
    exit 0
else
    echo -e "${RED}✗ Some checks failed${NC}"
    echo -e "${RED}Fix the issues above before pushing${NC}"
    echo -e "${BLUE}========================================${NC}"
    exit 1
fi
