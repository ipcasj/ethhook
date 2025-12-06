#!/bin/sh
set -e

# Entrypoint script for ethhook-admin-api
# Ensures /data directory exists with proper permissions before starting

echo "=== EthHook Admin API Entrypoint ==="
echo "User: $(whoami) (uid=$(id -u), gid=$(id -g))"
echo "Working directory: $(pwd)"

# Check and create /data directory if needed
if [ ! -d "/data" ]; then
    echo "WARNING: /data directory does not exist, attempting to create..."
    mkdir -p /data 2>/dev/null || {
        echo "ERROR: Cannot create /data directory (permission denied)"
        echo "This is likely a Docker volume mount issue"
        echo "Attempting to continue anyway - app will try to create it..."
    }
else
    echo "✓ /data directory exists"
fi

# Check write permissions
if [ -w "/data" ]; then
    echo "✓ /data directory is writable"
else
    echo "WARNING: /data directory is NOT writable"
    echo "Directory permissions:"
    ls -ld /data 2>/dev/null || echo "Cannot stat /data"
    echo "This may cause database initialization to fail"
fi

# List /data contents for debugging
echo "Contents of /data:"
ls -la /data 2>/dev/null || echo "Cannot list /data contents"

# Display database URL from environment
if [ -n "$DATABASE_URL" ]; then
    echo "Database URL: $DATABASE_URL"
else
    echo "WARNING: DATABASE_URL environment variable not set"
fi

echo "=== Starting application ==="
echo ""

# Execute the admin API with all arguments
exec /app/ethhook-admin-api "$@"
