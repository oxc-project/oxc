#!/bin/bash
set -e

COMMIT=$1

if [ -z "$COMMIT" ]; then
  echo "Usage: $0 <commit-sha>"
  exit 1
fi

echo "=========================================="
echo "Testing commit: $COMMIT"
echo "=========================================="

# Checkout commit
git checkout "$COMMIT"

# Clean
cd napi/parser
rm -rf node_modules .pnpm-store

# Install
pnpm install --ignore-scripts

# Build
echo "Building..."
if ! pnpm build --features allocator --release 2>&1 | tail -10; then
  echo "❌ Build failed"
  exit 1
fi

echo "✅ Build succeeded"

# Run test - just try to import in one worker
echo "Testing import..."
node test-worker.mjs

echo "✅ Test passed"
