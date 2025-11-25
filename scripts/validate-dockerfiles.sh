#!/bin/bash
# Validate that all Dockerfiles include all required workspace members

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "━━━ Dockerfile Workspace Validation ━━━"
echo ""

# Get all workspace members from root Cargo.toml (excluding comments)
WORKSPACE_MEMBERS=$(grep -A 30 '^\[workspace\]' Cargo.toml | grep '"crates/' | grep -v '^\s*#' | sed 's/.*"\(crates\/[^"]*\)".*/\1/' | sort)
TEST_MEMBER=$(grep -A 30 '^\[workspace\]' Cargo.toml | grep '"tests"' | grep -v '^\s*#' | sed 's/.*"\([^"]*\)".*/\1/')

echo "Workspace members:"
echo "$WORKSPACE_MEMBERS" | sed 's/^/  - /'
if [ -n "$TEST_MEMBER" ]; then
    echo "  - $TEST_MEMBER"
fi
echo ""

ERRORS=0

# Check each Rust Dockerfile
for dockerfile in crates/*/Dockerfile; do
    if [ ! -f "$dockerfile" ]; then
        continue
    fi
    
    echo "Checking $dockerfile..."
    
    # Extract COPY commands for crates and tests
    COPIED_CRATES=$(grep "COPY crates/" "$dockerfile" | sed 's/.*COPY \(crates\/[^ ]*\).*/\1/' | sort)
    COPIED_TESTS=$(grep "COPY tests " "$dockerfile" || echo "")
    
    # Check if all workspace members are copied
    MISSING=""
    while IFS= read -r member; do
        if ! echo "$COPIED_CRATES" | grep -q "^$member$"; then
            MISSING="$MISSING\n  - $member"
        fi
    done <<< "$WORKSPACE_MEMBERS"
    
    # Note: tests directory is optional in Dockerfiles (excluded by .dockerignore for production)
    
    if [ -n "$MISSING" ]; then
        echo -e "${RED}✗ Missing workspace members:${NC}$MISSING"
        ERRORS=$((ERRORS + 1))
    else
        echo -e "${GREEN}✓ All workspace members present${NC}"
    fi
    echo ""
done

if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}━━━ Validation Failed ━━━${NC}"
    echo -e "${YELLOW}Cargo workspaces require ALL member crates to be present in the build context,${NC}"
    echo -e "${YELLOW}even when building a single binary.${NC}"
    exit 1
else
    echo -e "${GREEN}━━━ All Dockerfiles Valid ━━━${NC}"
    exit 0
fi
