#!/bin/bash
# Script to set up GitHub Container Registry authentication on deployment server
#
# Usage:
#   1. Create a GitHub Personal Access Token (Classic) with 'read:packages' scope
#      https://github.com/settings/tokens/new
#   2. Run this script on the deployment server:
#      ./setup-ghcr-auth.sh YOUR_GITHUB_TOKEN

set -e

if [ -z "$1" ]; then
    echo "Error: GitHub token required"
    echo "Usage: $0 <GITHUB_TOKEN>"
    echo ""
    echo "To create a token:"
    echo "1. Go to https://github.com/settings/tokens/new"
    echo "2. Give it a name like 'ethhook-deployment'"
    echo "3. Select scope: read:packages"
    echo "4. Click 'Generate token'"
    echo "5. Copy the token and run: $0 <token>"
    exit 1
fi

GITHUB_TOKEN="$1"
GITHUB_USER="ipcasj"

echo "🔐 Setting up GitHub Container Registry authentication..."

# Login to ghcr.io
echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$GITHUB_USER" --password-stdin

if [ $? -eq 0 ]; then
    echo "✅ Successfully authenticated with ghcr.io"
    echo "You can now pull images with:"
    echo "  docker pull ghcr.io/ipcasj/ethhook-event-ingestor:latest"
else
    echo "❌ Authentication failed"
    exit 1
fi
