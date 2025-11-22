#!/bin/bash
# Build script for ETHhook-C

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔨 Building ETHhook-C...${NC}"

# Check for required tools
command -v cmake >/dev/null 2>&1 || { echo -e "${RED}❌ cmake is required but not installed.${NC}" >&2; exit 1; }

# Build type (default: Release)
BUILD_TYPE="${1:-Release}"

echo -e "${YELLOW}📋 Configuration:${NC}"
echo "   Build type: $BUILD_TYPE"

# Configure
echo -e "${YELLOW}⚙️  Configuring...${NC}"
cmake -B build \
    -DCMAKE_BUILD_TYPE=$BUILD_TYPE \
    -DENABLE_TESTS=ON \
    -DENABLE_EXAMPLES=ON

# Build
echo -e "${YELLOW}🔧 Compiling...${NC}"
cmake --build build -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

echo -e "${GREEN}✅ Build complete!${NC}"
echo -e "\nBinaries available in ${YELLOW}build/bin/${NC}:"
ls -lh build/bin/ 2>/dev/null || echo "   (build directory not found)"

echo -e "\n${GREEN}To run services:${NC}"
echo "   ./build/bin/event-ingestor"
echo "   ./build/bin/message-processor"
echo "   ./build/bin/webhook-delivery"
echo "   ./build/bin/admin-api"
