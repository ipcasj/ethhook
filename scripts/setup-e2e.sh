#!/bin/bash

set -e

echo "🎭 Setting up Playwright E2E Testing..."

cd ui

# Install Playwright
echo "📦 Installing Playwright..."
npm install --save-dev @playwright/test

# Install browsers
echo "🌐 Installing browser dependencies..."
npx playwright install chromium

# Create test directory structure
echo "📁 Creating test structure..."
mkdir -p e2e/fixtures

echo "✅ Playwright setup complete!"
echo ""
echo "Next steps:"
echo "1. Copy test files from E2E_TESTING_GUIDE.md"
echo "2. Run: npm run test:e2e:ui (interactive mode)"
echo "3. Run: npm run test:e2e (headless mode)"
