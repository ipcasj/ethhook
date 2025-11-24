#!/bin/bash
# Setup script for DigitalOcean automated deployment

set -e

echo "🚀 DigitalOcean Automated Deployment Setup"
echo "=========================================="
echo ""

# Configuration
DO_HOST="104.248.15.178"
SSH_KEY_PATH="$HOME/.ssh/github_deploy"

# Check if SSH key already exists
if [ -f "$SSH_KEY_PATH" ]; then
    echo "⚠️  SSH key already exists at $SSH_KEY_PATH"
    read -p "Do you want to use the existing key? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Aborted. Remove existing key or choose a different path."
        exit 1
    fi
else
    echo "📝 Generating new SSH key..."
    ssh-keygen -t ed25519 -C "github-deploy" -f "$SSH_KEY_PATH" -N ""
    echo "✅ SSH key generated"
fi

echo ""
echo "📤 Copying public key to DigitalOcean droplet..."
ssh-copy-id -i "${SSH_KEY_PATH}.pub" "root@$DO_HOST"
echo "✅ Public key added to droplet"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "🔑 GITHUB SECRETS SETUP"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Go to: https://github.com/ipcasj/ethhook/settings/secrets/actions"
echo ""
echo "Add these 2 secrets:"
echo ""
echo "1. Secret Name: DO_SSH_PRIVATE_KEY"
echo "   Secret Value: (copy from below)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cat "$SSH_KEY_PATH"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "2. Secret Name: DO_HOST"
echo "   Secret Value: $DO_HOST"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""
read -p "Press ENTER after you've added both secrets to GitHub..."
echo ""
echo "✅ Setup complete!"
echo ""
echo "🎉 Next steps:"
echo "   1. Commit and push your changes to 'main' branch"
echo "   2. CI will run and build Docker images"
echo "   3. Deployment will automatically trigger"
echo "   4. Monitor at: https://github.com/ipcasj/ethhook/actions"
echo ""
