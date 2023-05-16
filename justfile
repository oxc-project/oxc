#!/usr/bin/env -S just --justfile

_default:
  just --list -u

# Initialize the project by installing all the necessary tools
init:
  cargo binstall cargo-nextest cargo-watch cargo-insta typos-cli taplo-cli wasm-pack cargo-llvm-cov -y

# Run `cargo watch`
watch command:
  # --no-vcs-ignores: cargo-watch has a bug loading all .gitignores, including the ones listed in .gitignore
  # use .ignore file getting the ignore list
  cargo watch --no-vcs-ignores -x '{{command}}'

# Format all files
fmt:
  cargo fmt
  taplo format

# Run cargo check
check:
  cargo ck

# Run all the tests
test:
  cargo nextest run

# Lint the whole project
lint:
  cargo lint

# Run all the conformance tests. See `tasks/coverage`, `tasks/minsize`
coverage:
  cargo coverage
  cargo minsize

# Get code coverage
codecov:
  cargo codecov --html

# Run the benchmarks. See `tasks/benchmark`
benchmark:
  cargo benchmark

# Create a new lint rule by providing the ESLint name. See `tasks/rulegen`
new-rule name:
  cargo run -p rulegen {{name}}

# We are ready, let's run the same CI commands
ready:
  git diff --exit-code --quiet
  typos
  cargo fmt
  just check
  just test
  just lint
  git status
