#!/bin/bash
set -e

echo "==================================="
echo "EthHook C - Build Script"
echo "==================================="

# Check dependencies
echo ""
echo "Checking build dependencies..."

check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 not found"
        return 1
    else
        echo "✅ $1 found"
        return 0
    fi
}

check_command cmake || exit 1
check_command make || exit 1
check_command gcc || check_command clang || exit 1
check_command pkg-config || exit 1

# Check libraries
echo ""
echo "Checking required libraries..."

check_library() {
    if pkg-config --exists $1 2>/dev/null; then
        echo "✅ $1 found ($(pkg-config --modversion $1))"
        return 0
    else
        echo "❌ $1 not found"
        return 1
    fi
}

MISSING_LIBS=0
check_library libevent || MISSING_LIBS=1
check_library libwebsockets || MISSING_LIBS=1
check_library hiredis || MISSING_LIBS=1
check_library jansson || MISSING_LIBS=1
check_library libcurl || MISSING_LIBS=1
check_library libmicrohttpd || MISSING_LIBS=1
check_library libjwt || MISSING_LIBS=1
check_library sqlite3 || MISSING_LIBS=1

if [ $MISSING_LIBS -eq 1 ]; then
    echo ""
    echo "❌ Missing required libraries!"
    echo ""
    echo "Install on Alpine:"
    echo "  apk add libevent-dev libwebsockets-dev hiredis-dev jansson-dev"
    echo "  apk add curl-dev libmicrohttpd-dev libjwt-dev openssl-dev sqlite-dev"
    echo ""
    echo "Install on Ubuntu/Debian:"
    echo "  sudo apt-get install libevent-dev libwebsockets-dev libhiredis-dev"
    echo "  sudo apt-get install libjansson-dev libcurl4-openssl-dev libmicrohttpd-dev"
    echo "  sudo apt-get install libjwt-dev libssl-dev libsqlite3-dev"
    exit 1
fi

# Create build directory
echo ""
echo "Creating build directory..."
rm -rf build
mkdir build
cd build

# Configure with CMake
echo ""
echo "Configuring with CMake..."
cmake -DCMAKE_BUILD_TYPE=Release ..

# Build
echo ""
echo "Building all targets..."
make -j$(nproc) || make -j4 || make

# Check binaries
echo ""
echo "Build complete! Generated binaries:"
ls -lh ethhook-* 2>/dev/null || echo "❌ No binaries generated"

echo ""
echo "==================================="
echo "✅ Build successful!"
echo "==================================="
echo ""
echo "Next steps:"
echo "  1. Create configuration: cp ../config/config.example.toml ../config.toml"
echo "  2. Edit config.toml with your settings"
echo "  3. Run services:"
echo "     ./ethhook-ingestor ../config.toml &"
echo "     ./ethhook-processor ../config.toml &"
echo "     ./ethhook-delivery ../config.toml &"
echo "     ./ethhook-admin-api ../config.toml &"
echo ""
echo "Or use Docker:"
echo "  cd ../docker"
echo "  docker-compose up -d"
echo ""
