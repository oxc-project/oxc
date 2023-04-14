#!/usr/bin/env -S just --justfile

_default:
  just --list -u

# Initialize the project by installing all the necessary tools
init:
  cargo binstall cargo-nextest cargo-watch cargo-insta typos-cli -y

# Run all the tests
test:
  cargo nextest run

# Lint the whole project
lint:
  cargo lint

# Run all the conformance tests. See `tasks/coverage`
coverage:
  cargo coverage

# Run the benchmarks. See `tasks/benchmark`
benchmark:
  cargo benchmark

# Create a new lint rule by providing the ESLint name. See `tasks/rulegen`
new-rule name:
  cargo run -p rulegen ${{name}}

# We are ready, let's run the same CI commands
ready:
  git diff --exit-code --quiet
  typos
  cargo fmt
  just lint
  just test
  git status
