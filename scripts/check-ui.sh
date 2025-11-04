#!/bin/bash

set -e  # Exit on error

echo "🔍 Step 1/3: Type checking..."
cd ui
npx tsc --noEmit

echo "🧹 Step 2/3: Linting..."
npm run lint

echo "🏗️  Step 3/3: Building..."
npm run build

echo ""
echo "✅ All checks passed!"
echo "🚀 Ready to deploy!"
