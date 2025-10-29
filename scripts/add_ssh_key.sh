#!/bin/bash

# =============================================================================
# Add SSH Key to DigitalOcean Droplet
# =============================================================================
# This script helps you add your SSH key to a Droplet using password auth
#
# Usage:
#   ./scripts/add_ssh_key.sh <droplet-ip>
#
# You'll be prompted for the root password (check your email from DO)
# =============================================================================

set -e

if [ $# -eq 0 ]; then
    echo "Error: Droplet IP address required"
    echo "Usage: ./scripts/add_ssh_key.sh <droplet-ip>"
    echo "Example: ./scripts/add_ssh_key.sh 104.248.15.178"
    exit 1
fi

DROPLET_IP=$1

echo "════════════════════════════════════════"
echo "  Add SSH Key to Droplet"
echo "════════════════════════════════════════"
echo ""
echo "Droplet IP: $DROPLET_IP"
echo ""
echo "This will copy your SSH public key to the Droplet."
echo "You'll be prompted for the root password."
echo "(Check your DigitalOcean email for the password)"
echo ""
echo "════════════════════════════════════════"
echo ""

# Check if ssh-copy-id exists
if ! command -v ssh-copy-id &> /dev/null; then
    echo "Installing ssh-copy-id..."
    brew install ssh-copy-id 2>/dev/null || {
        echo "Error: ssh-copy-id not found and couldn't be installed"
        echo ""
        echo "Manual method:"
        echo "1. Get your SSH public key:"
        echo "   cat ~/.ssh/id_rsa.pub"
        echo ""
        echo "2. SSH into Droplet with password:"
        echo "   ssh root@$DROPLET_IP"
        echo ""
        echo "3. Add the key:"
        echo "   mkdir -p ~/.ssh"
        echo "   echo 'YOUR_PUBLIC_KEY_HERE' >> ~/.ssh/authorized_keys"
        echo "   chmod 600 ~/.ssh/authorized_keys"
        echo "   chmod 700 ~/.ssh"
        exit 1
    }
fi

# Copy SSH key
echo "Copying SSH key to $DROPLET_IP..."
echo "Enter the root password when prompted:"
echo ""

ssh-copy-id -o StrictHostKeyChecking=no root@$DROPLET_IP

echo ""
echo "════════════════════════════════════════"
echo "✓ SSH key added successfully!"
echo "════════════════════════════════════════"
echo ""
echo "Testing connection..."
if ssh -o ConnectTimeout=5 -o BatchMode=yes root@$DROPLET_IP exit 2>/dev/null; then
    echo "✓ SSH connection works!"
    echo ""
    echo "You can now run:"
    echo "  ./deploy.sh $DROPLET_IP"
else
    echo "✗ SSH connection still failing"
    echo ""
    echo "Try manually:"
    echo "  ssh root@$DROPLET_IP"
fi
echo ""
