#!/bin/bash

# EthHook - Clean up /etc/hosts file
# This removes the grocery store domain entries

echo "🧹 Cleaning up /etc/hosts file..."
echo ""

# Show current extra entries
echo "📋 Current extra entries that will be removed:"
grep -E "giantfood|stopandshop|martinsfoods|giantfoodstores|foodlion|hannaford|auth-server" /etc/hosts || echo "   (none found)"

echo ""
read -p "❓ Do you want to remove these entries? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cancelled."
    exit 0
fi

# Backup first
echo "💾 Creating backup at /etc/hosts.backup..."
sudo cp /etc/hosts /etc/hosts.backup

# Remove the entries
echo "🧹 Removing entries..."
sudo sed -i.tmp '/giantfood\|stopandshop\|martinsfoods\|giantfoodstores\|foodlion\|hannaford\|auth-server/d' /etc/hosts
sudo rm /etc/hosts.tmp 2>/dev/null || true

echo ""
echo "✅ Done! Your /etc/hosts file has been cleaned."
echo "📋 New /etc/hosts content:"
echo ""
cat /etc/hosts
echo ""
echo "💡 Backup saved at: /etc/hosts.backup"
echo "💡 To restore: sudo cp /etc/hosts.backup /etc/hosts"
