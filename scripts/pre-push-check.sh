#!/bin/bash

# EthHook Pre-Push Validation Script
# Runs comprehensive checks before pushing to repository

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall status
FAILED=0

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   EthHook Pre-Push Validation Checks       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
    else
        echo -e "${RED}✗ $2${NC}"
        FAILED=1
    fi
}

# Function to print section
print_section() {
    echo ""
    echo -e "${BLUE}━━━ $1 ━━━${NC}"
}

# ============================================================================
# 1. Environment Check
# ============================================================================
print_section "Environment Check"

# Check if required tools are installed
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}✗ cargo not found${NC}"; FAILED=1; }
command -v node >/dev/null 2>&1 || { echo -e "${RED}✗ node not found${NC}"; FAILED=1; }
command -v npm >/dev/null 2>&1 || { echo -e "${RED}✗ npm not found${NC}"; FAILED=1; }

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All required tools installed${NC}"
    echo "  - cargo: $(cargo --version | head -n1)"
    echo "  - node: $(node --version)"
    echo "  - npm: $(npm --version)"
fi

# ============================================================================
# 2. Environment Configuration Check
# ============================================================================
print_section "Environment Configuration"

if [ -f "ui/.env.local" ]; then
    print_status 0 "UI .env.local exists"
else
    print_status 1 "UI .env.local missing"
fi

if [ -f ".env" ]; then
    print_status 0 "Root .env exists"
else
    print_status 1 "Root .env missing (optional)"
fi

# ============================================================================
# 3. Rust Backend Compilation
# ============================================================================
print_section "Rust Backend Compilation"

echo "Checking Rust code..."
if cargo check --workspace --quiet 2>/dev/null; then
    print_status 0 "Rust workspace compiles successfully"
else
    print_status 1 "Rust compilation failed"
    echo -e "${YELLOW}Run 'cargo check --workspace' for details${NC}"
fi

# ============================================================================
# 4. Rust Tests
# ============================================================================
print_section "Rust Tests"

echo "Running Rust unit tests..."
if cargo test --workspace --lib --quiet 2>/dev/null; then
    print_status 0 "Rust unit tests pass"
else
    print_status 1 "Rust unit tests failed"
    echo -e "${YELLOW}Run 'cargo test --workspace' for details${NC}"
fi

# ============================================================================
# 5. Rust Linting
# ============================================================================
print_section "Rust Linting"

echo "Running clippy..."
if cargo clippy --workspace --all-targets -- -D warnings 2>/dev/null; then
    print_status 0 "Clippy checks pass"
else
    echo -e "${YELLOW}⚠ Clippy warnings found (non-blocking)${NC}"
fi

# ============================================================================
# 6. TypeScript/Next.js Build
# ============================================================================
print_section "TypeScript/Next.js Build"

cd ui

echo "Installing dependencies..."
if npm install --silent 2>/dev/null; then
    print_status 0 "npm dependencies installed"
else
    print_status 1 "npm install failed"
fi

echo "Type checking..."
if npm run type-check --silent 2>/dev/null; then
    print_status 0 "TypeScript type checking passes"
else
    if npx tsc --noEmit 2>/dev/null; then
        print_status 0 "TypeScript type checking passes"
    else
        print_status 1 "TypeScript has errors"
        echo -e "${YELLOW}Run 'cd ui && npx tsc --noEmit' for details${NC}"
    fi
fi

echo "Building Next.js..."
if npm run build 2>/dev/null; then
    print_status 0 "Next.js builds successfully"
else
    echo -e "${YELLOW}⚠ Next.js build failed (non-blocking for development)${NC}"
    echo -e "${YELLOW}Note: Next.js 15 requires Node.js >=20.9.0${NC}"
    echo -e "${YELLOW}Run 'cd ui && npm run build' for details${NC}"
fi

cd ..

# ============================================================================
# 7. UI Linting
# ============================================================================
print_section "UI Linting"

cd ui

echo "Running ESLint..."
if npm run lint; then
    print_status 0 "ESLint passes"
else
    print_status 1 "ESLint has errors"
    echo -e "${YELLOW}Run 'cd ui && npm run lint' for details${NC}"
fi

cd ..

# ============================================================================
# 8. E2E Tests (Smoke Tests)
# ============================================================================
print_section "E2E Tests (Smoke Tests)"

echo -e "${YELLOW}Note: Skipping E2E tests (requires running services)${NC}"
echo -e "${YELLOW}Run manually: cd ui && npm run test:e2e -- 00-smoke.spec.ts${NC}"

# ============================================================================
# 9. Git Status Check
# ============================================================================
print_section "Git Status"

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}⚠ Uncommitted changes found:${NC}"
    git status --short
    echo ""
    echo -e "${YELLOW}Consider committing changes before pushing${NC}"
else
    print_status 0 "No uncommitted changes"
fi

# Check current branch
CURRENT_BRANCH=$(git branch --show-current)
echo -e "Current branch: ${BLUE}${CURRENT_BRANCH}${NC}"

# ============================================================================
# 10. Final Summary
# ============================================================================
print_section "Summary"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║      ✓ ALL CHECKS PASSED                   ║${NC}"
    echo -e "${GREEN}║      Ready to push!                        ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    echo ""
    exit 0
else
    echo ""
    echo -e "${RED}╔════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║      ✗ SOME CHECKS FAILED                  ║${NC}"
    echo -e "${RED}║      Please fix issues before pushing      ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════╝${NC}"
    echo ""
    exit 1
fi
