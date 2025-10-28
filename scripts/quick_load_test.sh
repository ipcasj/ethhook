#!/bin/bash
# Quick load test - runs a smaller test for fast validation

set -e

echo "🚀 Running quick load test (1,000 events at 500/sec)..."
echo ""

./scripts/run_load_test.sh 1000 500 5
