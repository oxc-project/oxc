#!/bin/bash
# Bisect test script for issue #14732
# This script builds oxc-parser and runs the worker thread test
# Returns 0 if the test passes (no crash), 1 if it fails (crash/error)

set -e

echo "Building oxc-parser..."
cd napi/parser
pnpm build-dev --features allocator --release 2>&1 | tail -5

echo "Running worker thread test..."
# Run the test and capture exit code
if timeout 30 node test-worker-main.mjs 2>&1 | grep -q "All workers done!"; then
    echo "✓ Test PASSED - No crash detected"
    exit 0
else
    echo "✗ Test FAILED - Crash or timeout detected"
    exit 1
fi
