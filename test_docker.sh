#!/bin/bash

# Script to test oxc in Ubuntu 24.04 Docker container

docker run --rm -v "$(pwd):/workspace" -w /workspace ubuntu:24.04 bash -c "
set -e

echo 'Setting up Ubuntu environment...'
apt-get update --allow-releaseinfo-change
apt-get install -y curl build-essential pkg-config libssl-dev ca-certificates

echo 'Installing Rust...'
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source /root/.cargo/env

echo 'Verifying Rust installation...'
rustc --version
cargo --version

echo 'Setting up environment...'
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

echo 'Running tests...'
cargo test --workspace --all-features --exclude website --exclude oxc_playground_napi 2>&1 | head -100
"