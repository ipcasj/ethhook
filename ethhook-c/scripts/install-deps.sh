#!/bin/bash
# Install dependencies for ETHhook-C

set -e

echo "🔧 Installing ETHhook-C dependencies..."

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Ubuntu/Debian
    if command -v apt-get &> /dev/null; then
        echo "📦 Detected Ubuntu/Debian"
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            cmake \
            git \
            libuv1-dev \
            libcurl4-openssl-dev \
            libpq-dev \
            libhiredis-dev \
            libwebsockets-dev \
            libssl-dev \
            valgrind \
            clang-format \
            clang-tidy
    # Fedora/RHEL
    elif command -v dnf &> /dev/null; then
        echo "📦 Detected Fedora/RHEL"
        sudo dnf install -y \
            gcc \
            gcc-c++ \
            cmake \
            git \
            libuv-devel \
            libcurl-devel \
            postgresql-devel \
            hiredis-devel \
            libwebsockets-devel \
            openssl-devel \
            valgrind \
            clang-tools-extra
    fi

elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "📦 Detected macOS"
    if ! command -v brew &> /dev/null; then
        echo "❌ Homebrew is required. Install from https://brew.sh"
        exit 1
    fi

    brew install \
        cmake \
        libuv \
        curl \
        postgresql \
        hiredis \
        libwebsockets \
        openssl \
        valgrind \
        clang-format

else
    echo "❌ Unsupported OS: $OSTYPE"
    exit 1
fi

echo "✅ Dependencies installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Run './scripts/build.sh' to build the project"
echo "  2. Run 'docker compose up -d' to start infrastructure"
