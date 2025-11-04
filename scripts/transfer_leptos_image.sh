#!/bin/bash

# =============================================================================
# Transfer leptos-portal Docker Image to Droplet
# =============================================================================
# This script saves the locally built leptos-portal image and transfers it
# to the production Droplet, then loads it there.
#
# Usage:
#   ./scripts/transfer_leptos_image.sh 104.248.15.178
# =============================================================================

set -e

DROPLET_IP="${1:-104.248.15.178}"
IMAGE_NAME="capstone0-leptos-portal"
TEMP_FILE="/tmp/leptos-portal.tar"

echo "🔍 Checking if leptos-portal image exists locally..."
if ! docker images | grep -q "$IMAGE_NAME"; then
    echo "❌ Error: Image $IMAGE_NAME not found locally"
    echo "   Run: docker compose -f docker-compose.prod.yml build leptos-portal"
    exit 1
fi

echo "📦 Saving Docker image to tar file..."
docker save "$IMAGE_NAME:latest" -o "$TEMP_FILE"

echo "📊 Image size: $(du -h $TEMP_FILE | cut -f1)"

echo "🚀 Transferring image to Droplet ($DROPLET_IP)..."
scp "$TEMP_FILE" "root@$DROPLET_IP:/tmp/leptos-portal.tar"

echo "📥 Loading image on Droplet..."
ssh "root@$DROPLET_IP" "docker load -i /tmp/leptos-portal.tar && rm /tmp/leptos-portal.tar"

echo "🏷️  Tagging image with correct name..."
ssh "root@$DROPLET_IP" "docker tag $IMAGE_NAME:latest ethhook-leptos-portal:latest"

echo "🧹 Cleaning up local temp file..."
rm "$TEMP_FILE"

echo "✅ Image transfer complete!"
echo ""
echo "Next steps:"
echo "  ssh root@$DROPLET_IP"
echo "  cd /root/ethhook"
echo "  docker-compose -f docker-compose.prod.yml up -d leptos-portal"
