#!/usr/bin/env -S just --justfile

_default:
  @just --list -u

alias r := ready
alias c := coverage

# Make sure you have cargo-binstall installed.
# You can download the pre-compiled binary from <https://github.com/cargo-bins/cargo-binstall#installation>
# or install via `cargo install cargo-binstall`
# Initialize the project by installing all the necessary tools.
init:
  cargo binstall cargo-watch cargo-insta cargo-edit typos-cli taplo-cli wasm-pack cargo-llvm-cov -y

# When ready, run the same CI commands
ready:
  git diff --exit-code --quiet
  typos
  cargo fmt
  just check
  just test
  just lint
  git status

# Update our local branch with the remote branch (this is for you to sync the git submodules)
update:
  git submodule sync
  git submodule update --init --recursive

# --no-vcs-ignores: cargo-watch has a bug loading all .gitignores, including the ones listed in .gitignore
# use .ignore file getting the ignore list
# Run `cargo watch`
watch command:
  cargo watch --no-vcs-ignores -i '*snap*' -x '{{command}}'

# Run the example in `parser`, `formatter`, `linter`
example tool *args='':
  just watch 'run -p oxc_{{tool}} --example {{tool}} -- {{args}}'

# Format all files
fmt:
  cargo fmt
  taplo format

# Run cargo check
check:
  cargo ck

# Run all the tests
test:
  cargo test

test-transform:
  cargo run -p oxc_transform_conformance
  cargo run -p oxc_transform_conformance -- --exec

# Lint the whole project
lint:
  cargo lint -- --deny warnings

# Run all the conformance tests. See `tasks/coverage`, `tasks/transform_conformance`, `tasks/minsize`
coverage:
  cargo coverage
  cargo run --release -p oxc_transform_conformance -- --exec
  cargo run --release -p oxc_prettier_conformance
  # cargo minsize

# Get code coverage
codecov:
  cargo codecov --html

# Run the benchmarks. See `tasks/benchmark`
benchmark:
  cargo benchmark

# Generate the JavaScript global variables. See `tasks/javascript_globals`
javascript-globals:
  cargo run -p javascript_globals

# Create a new lint rule by providing the ESLint name. See `tasks/rulegen`
new-rule name:
  cargo run -p rulegen {{name}}

new-deepscan-rule name:
  cargo run -p rulegen {{name}} deepscan

new-jest-rule name:
  cargo run -p rulegen {{name}} jest

new-ts-rule name:
  cargo run -p rulegen {{name}} typescript

new-unicorn-rule name:
  cargo run -p rulegen {{name}} unicorn

new-react-rule name:
  cargo run -p rulegen {{name}} react

new-jsx-a11y-rule name:
  cargo run -p rulegen {{name}} jsx-a11y

new-oxc-rule name:
  cargo run -p rulegen {{name}} oxc

new-nextjs-rule name:
  cargo run -p rulegen {{name}} nextjs

new-jsdoc-rule name:
  cargo run -p rulegen {{name}} jsdoc

new-react-perf-rule name:
    cargo run -p rulegen {{name}} react-perf

new-n-rule name:
    cargo run -p rulegen {{name}} n

# Sync all submodules with their own remote repos (this is for Boshen updating the submodules)
sync-submodules:
  git submodule update --init --remote

# Upgrade all Rust dependencies
upgrade:
  cargo upgrade --incompatible
