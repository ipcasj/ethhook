#!/bin/bash

# EthHook Railway Deployment Script
# This script helps you deploy EthHook to Railway.app
#
# Prerequisites:
# - Railway CLI installed: npm install -g @railway/cli
# - Railway account created
# - Logged in: railway login

set -e  # Exit on error

echo "🚀 EthHook Railway Deployment Script"
echo "======================================"
echo ""

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found"
    echo "Install it with: npm install -g @railway/cli"
    exit 1
fi

echo "✅ Railway CLI found"
echo ""

# Check if logged in
if ! railway whoami &> /dev/null; then
    echo "❌ Not logged into Railway"
    echo "Run: railway login"
    exit 1
fi

echo "✅ Logged into Railway"
echo ""

# Step 1: Create or link project
echo "Step 1: Project Setup"
echo "--------------------"
read -p "Do you want to (C)reate new project or (L)ink existing? [C/L]: " choice

if [ "$choice" == "C" ] || [ "$choice" == "c" ]; then
    read -p "Enter project name [ethhook-production]: " project_name
    project_name=${project_name:-ethhook-production}

    echo "Creating new Railway project: $project_name"
    railway init -n "$project_name"
else
    echo "Linking to existing project"
    railway link
fi

echo ""

# Step 2: Check for environment variables
echo "Step 2: Environment Variables"
echo "-----------------------------"

if [ ! -f .env ]; then
    echo "⚠️  No .env file found"
    read -p "Do you want to use .env.production.example as template? [Y/n]: " use_template

    if [ "$use_template" != "n" ] && [ "$use_template" != "N" ]; then
        cp .env.production.example .env
        echo "✅ Created .env from template"
        echo ""
        echo "⚠️  IMPORTANT: Edit .env and add your API keys:"
        echo "   - Alchemy API keys (SEPOLIA_RPC_WS, SEPOLIA_RPC_HTTP)"
        echo "   - JWT_SECRET (generate with: openssl rand -base64 32)"
        echo ""
        read -p "Press Enter when you've edited .env..."
    fi
fi

# Step 3: Deploy services
echo ""
echo "Step 3: Service Deployment"
echo "-------------------------"
echo ""
echo "You need to deploy 4 services manually via Railway dashboard:"
echo ""
echo "1. admin-api (Dockerfile: crates/admin-api/Dockerfile)"
echo "2. event-ingestor (Dockerfile: crates/event-ingestor/Dockerfile)"
echo "3. message-processor (Dockerfile: crates/message-processor/Dockerfile)"
echo "4. webhook-delivery (Dockerfile: crates/webhook-delivery/Dockerfile)"
echo ""
echo "For each service:"
echo "  - Go to Railway dashboard"
echo "  - Click '+ New' → 'GitHub Repo'"
echo "  - Select this repository"
echo "  - Set Dockerfile path"
echo "  - Configure environment variables"
echo ""

read -p "Have you deployed all 4 services? [y/N]: " deployed

if [ "$deployed" != "y" ] && [ "$deployed" != "Y" ]; then
    echo ""
    echo "Please deploy services first via Railway dashboard"
    echo "See docs/RAILWAY_DEPLOYMENT_GUIDE.md for detailed instructions"
    exit 0
fi

# Step 4: Run migrations
echo ""
echo "Step 4: Database Migrations"
echo "---------------------------"
read -p "Do you want to run database migrations? [Y/n]: " run_migrations

if [ "$run_migrations" != "n" ] && [ "$run_migrations" != "N" ]; then
    echo "Running migrations via admin-api service..."
    railway run --service admin-api sqlx migrate run
    echo "✅ Migrations completed"
fi

# Step 5: Verify deployment
echo ""
echo "Step 5: Verification"
echo "-------------------"
echo ""
echo "Checking service health..."
echo ""

# Get admin-api URL
read -p "Enter your admin-api Railway URL (e.g., https://admin-api-xxxx.up.railway.app): " api_url

if [ -n "$api_url" ]; then
    echo "Testing health endpoint: $api_url/api/v1/health"

    if curl -s "$api_url/api/v1/health" | grep -q "healthy"; then
        echo "✅ Admin API is healthy!"
    else
        echo "⚠️  Health check failed - check service logs"
    fi
fi

echo ""
echo "===================================="
echo "🎉 Deployment Complete!"
echo "===================================="
echo ""
echo "Next steps:"
echo "1. Check all service logs in Railway dashboard"
echo "2. Deploy frontend (see docs/RAILWAY_DEPLOYMENT_GUIDE.md)"
echo "3. Update CORS_ALLOWED_ORIGINS in admin-api with frontend URL"
echo "4. Create a test endpoint and verify webhook delivery"
echo ""
echo "Useful commands:"
echo "  - View logs: railway logs --service <service-name>"
echo "  - Run command: railway run --service <service-name> <command>"
echo "  - Open dashboard: railway open"
echo ""
echo "Docs: docs/RAILWAY_DEPLOYMENT_GUIDE.md"
echo "Support: https://discord.gg/railway"
echo ""
