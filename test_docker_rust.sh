#!/bin/bash

# Script to test oxc using official Rust Docker image

docker run --rm -v "$(pwd):/workspace" -w /workspace rust:1.87-bookworm bash -c "
set -e

echo 'Updating package lists...'
apt-get update
apt-get install -y pkg-config libssl-dev ca-certificates

echo 'Rust version:'
rustc --version
cargo --version

echo 'Setting up environment...'
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

echo 'Running tests (first 100 lines of output)...'
timeout 300 cargo test --workspace --all-features --exclude website --exclude oxc_playground_napi 2>&1 | head -100 || echo 'Test execution timed out or completed'
"