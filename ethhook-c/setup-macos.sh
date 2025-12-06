#!/bin/bash
# macOS Development Environment Setup for EthHook C
# This script installs all dependencies needed to build the C services locally

set -e  # Exit on error

echo "=================================="
echo "EthHook C - macOS Dev Setup"
echo "=================================="
echo ""

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew not found. Installing..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
    echo "✅ Homebrew already installed"
fi

echo ""
echo "Installing build tools and dependencies..."
echo ""

# Build essentials
brew install cmake || true
brew install gcc || true
brew install pkg-config || true

# Libraries
brew install libevent || true
brew install hiredis || true
brew install curl || true
brew install libmicrohttpd || true
brew install jwt || true  # libjwt
brew install openssl@3 || true
brew install sqlite || true
brew install libwebsockets || true

# Optional: Development tools
brew install cppcheck || true          # Static analysis
brew install clang-format || true      # Code formatting
brew install valgrind || true          # Memory leak detection (x86_64 only)

echo ""
echo "=================================="
echo "✅ Installation Complete!"
echo "=================================="
echo ""
echo "To build the project:"
echo "  cd /Users/igor/rust_projects/capstone0/ethhook-c"
echo "  rm -rf build"
echo "  cmake -B build"
echo "  cmake --build build"
echo ""
echo "To build with sanitizers (recommended for development):"
echo "  cmake -B build -DENABLE_ASAN=ON -DENABLE_UBSAN=ON"
echo "  cmake --build build"
echo ""
echo "To run static analysis:"
echo "  make cppcheck     # Static analysis"
echo "  make format       # Auto-format code"
echo "  make format-check # Check formatting"
echo ""
echo "=================================="
