#!/usr/bin/env -S just --justfile

set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

_default:
  @just --list -u

alias r := ready
alias c := coverage
alias f := fix
alias new-typescript-rule := new-ts-rule

# Make sure you have cargo-binstall installed.
# You can download the pre-compiled binary from <https://github.com/cargo-bins/cargo-binstall#installation>
# or install via `cargo install cargo-binstall`
# Initialize the project by installing all the necessary tools.
init:
  cargo binstall cargo-watch cargo-insta typos-cli taplo-cli wasm-pack cargo-llvm-cov cargo-shear -y

# When ready, run the same CI commands
ready:
  git diff --exit-code --quiet
  typos
  just fmt
  just check
  just test
  just lint
  just doc
  just ast
  cargo shear
  git status

# Clone or update submodules
submodules:
  just clone-submodule tasks/coverage/test262 git@github.com:tc39/test262.git a15874163e6a4f19ee7cd3e47592af382af0f5fd
  just clone-submodule tasks/coverage/babel git@github.com:babel/babel.git 12619ffe5b0777edb0223304da1fdf8770d93e7c
  just clone-submodule tasks/coverage/typescript git@github.com:microsoft/TypeScript.git d8086f14b6b97c0df34a0cc2f56d4b5926a0c299
  just clone-submodule tasks/prettier_conformance/prettier git@github.com:prettier/prettier.git 7142cf354cce2558f41574f44b967baf11d5b603

# Install git pre-commit to format files
install-hook:
  echo "#!/bin/sh\njust fmt" > .git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit

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

# Lint the whole project
lint:
  cargo lint -- --deny warnings

doc:
  RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items

# Fix all auto-fixable format and lint issues. Make sure your working tree is clean first.
fix:
  cargo clippy --fix --allow-staged --no-deps
  just fmt
  typos -w
  git status

# Run all the conformance tests. See `tasks/coverage`, `tasks/transform_conformance`, `tasks/minsize`
coverage:
  cargo coverage
  cargo run -p oxc_transform_conformance -- --exec
  cargo run -p oxc_prettier_conformance
  # cargo minsize

# Get code coverage
codecov:
  cargo codecov --html

# Run the benchmarks. See `tasks/benchmark`
benchmark:
  cargo benchmark

# Removed Unused Dependencies
shear:
  cargo shear --fix

# Automatically DRY up Cargo.toml manifests in a workspace.
autoinherit:
  cargo binstall cargo-autoinherit
  cargo autoinherit

# Test Transform
test-transform *args='':
  cargo run -p oxc_transform_conformance -- {{args}}
  cargo run -p oxc_transform_conformance -- --exec  {{args}}

# Build oxlint in release build
oxlint:
  cargo oxlint

watch-wasm:
  cargo watch --no-vcs-ignores -i 'npm/oxc-wasm/**' -- just build-wasm

build-wasm:
  wasm-pack build --out-dir ../../npm/oxc-wasm --target web --dev --scope oxc crates/oxc_wasm

# Generate the JavaScript global variables. See `tasks/javascript_globals`
javascript-globals:
  cargo run -p javascript_globals

# Create a new lint rule by providing the ESLint name. See `tasks/rulegen`
new-rule name:
  cargo run -p rulegen {{name}}

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

new-promise-rule name:
    cargo run -p rulegen {{name}} promise

new-vitest-rule name:
    cargo run -p rulegen {{name}} vitest

clone-submodule dir url sha:
  git clone --depth=1 {{url}} {{dir}} || true
  cd {{dir}} && git fetch origin {{sha}} && git reset --hard {{sha}}

website path:
  cargo run -p website -- linter-rules --table {{path}}/src/docs/guide/usage/linter/generated-rules.md --rule-docs {{path}}/src/docs/guide/usage/linter/rules
  cargo run -p website -- linter-cli > {{path}}/src/docs/guide/usage/linter/generated-cli.md
  cargo run -p website -- linter-schema-markdown > {{path}}/src/docs/guide/usage/linter/generated-config.md

# sync ast changes
ast:
  cargo run -p oxc_ast_tools
  just check
