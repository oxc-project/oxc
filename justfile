#!/usr/bin/env -S just --justfile

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["bash", "-cu"]

_default:
  @just --list -u

alias r := ready
alias c := conformance
alias f := fix
alias new-typescript-rule := new-ts-rule

# Make sure you have cargo-binstall and pnpm installed.
# You can download the pre-compiled binary from <https://github.com/cargo-bins/cargo-binstall#installation>
# or install via `cargo install cargo-binstall`
# Initialize the project by installing all the necessary tools.
init:
  # Rust related init
  cargo binstall watchexec-cli cargo-insta typos-cli cargo-shear dprint -y
  # Node.js related init
  pnpm install

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
  git status

# Clone or update submodules
# Make sure to update `.github/actions/clone-submodules/action.yml` too
submodules:
  just clone-submodule tasks/coverage/test262 https://github.com/tc39/test262.git 4b5d36ab6ef2f59d0a8902cd383762547a3a74c4
  just clone-submodule tasks/coverage/babel https://github.com/babel/babel.git 1d4546bcb80009303aab386b59f4df1fd335c1d5
  just clone-submodule tasks/coverage/typescript https://github.com/microsoft/TypeScript.git 81c951894e93bdc37c6916f18adcd80de76679bc
  just clone-submodule tasks/prettier_conformance/prettier https://github.com/prettier/prettier.git 7584432401a47a26943dd7a9ca9a8e032ead7285
  just clone-submodule tasks/coverage/acorn-test262 https://github.com/oxc-project/acorn-test262 d9ba02ddea22800a285c7ad24e3fbfbb00ccbb02
  just update-transformer-fixtures

# Install git pre-commit to format files
install-hook:
  echo -e "#!/bin/sh\njust fmt" > .git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit

watch *args='':
  watchexec --no-vcs-ignore {{args}}

watch-check:
  just watch "'cargo check; cargo clippy'"

# Run the example in `parser`, `formatter`, `linter`
example tool *args='':
  cargo run -p oxc_{{tool}} --example {{tool}} -- {{args}}

watch-example *args='':
  just watch 'just example {{args}}'

# Build oxlint in release build; Run with `./target/release/oxlint`.
oxlint :
  cargo oxlint

# Watch oxlint
watch-oxlint *args='':
  just watch 'cargo run -p oxlint -- {{args}}'

# Run cargo check
check:
  cargo ck

# Run all the tests
test:
  cargo test --all-features

# Lint the whole project
lint:
  cargo lint -- --deny warnings

# Format all files
fmt:
  -cargo shear --fix # remove all unused dependencies
  cargo fmt --all
  dprint fmt

[unix]
doc:
  RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items

[windows]
doc:
  $Env:RUSTDOCFLAGS='-D warnings'; cargo doc --no-deps --document-private-items

# Fix all auto-fixable format and lint issues. Make sure your working tree is clean first.
fix:
  cargo clippy --fix --allow-staged --no-deps
  just fmt
  typos -w
  git status

# Run all the conformance tests. See `tasks/coverage`, `tasks/transform_conformance`
coverage:
  cargo coverage
  cargo run -p oxc_transform_conformance -- --exec
  cargo run -p oxc_prettier_conformance

# Run Test262, Babel and TypeScript conformance suite
conformance *args='':
  cargo coverage -- {{args}}

# Generate AST related boilerplate code.
# Run this when AST definition is changed.
ast:
  cargo run -p oxc_ast_tools
  just check

# Get code coverage
codecov:
  cargo codecov --html

# Run the benchmarks. See `tasks/benchmark`
benchmark:
  cargo benchmark

# Run the benchmarks for a single component.
# e.g. `just benchmark-one parser`.
# See `tasks/benchmark`.
benchmark-one *args:
  cargo benchmark --bench {{args}} --no-default-features --features {{args}}

# Update memory allocation snapshots.
allocs:
  cargo allocs

# Automatically DRY up Cargo.toml manifests in a workspace.
autoinherit:
  cargo binstall cargo-autoinherit
  cargo autoinherit

# Test Transform
test-transform *args='':
  cargo run -p oxc_transform_conformance -- --exec {{args}}

# Update transformer conformance test fixtures
update-transformer-fixtures:
  cd tasks/coverage/babel; git reset --hard HEAD; git clean -f -q
  node tasks/transform_conformance/update_fixtures.mjs

# Test ESTree
test-estree *args='':
  cargo run -p oxc_coverage --profile coverage -- estree {{args}}

# Install wasm32-wasip1-threads for playground
install-wasm:
  rustup target add wasm32-wasip1-threads

watch-playground:
  just watch 'pnpm --filter oxc-playground dev'

build-playground mode="release":
  pnpm --filter oxc-playground build

# Create a new lint rule by providing the ESLint name. See `tasks/rulegen`
new-rule name:
  cargo run -p rulegen {{name}}

new-jest-rule name:
  cargo run -p rulegen {{name}} jest

new-ts-rule name:
  cargo run -p rulegen {{name}} typescript

new-unicorn-rule name:
  cargo run -p rulegen {{name}} unicorn

new-import-rule name:
  cargo run -p rulegen {{name}} import

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

new-regexp-rule name:
    cargo run -p rulegen {{name}} regexp

new-vue-rule name:
    cargo run -p rulegen {{name}} vue

[unix]
clone-submodule dir url sha:
  cd {{dir}} || git init {{dir}}
  cd {{dir}} && git remote add origin {{url}} || true
  cd {{dir}} && git fetch --depth=1 origin {{sha}} && git reset --hard {{sha}} && git clean -f -q

[windows]
clone-submodule dir url sha:
  if (-not (Test-Path {{dir}}/.git)) { git init {{dir}} }
  cd {{dir}} ; if ((git remote) -notcontains 'origin') { git remote add origin {{url}} } else { git remote set-url origin {{url}} }
  cd {{dir}} ; git fetch --depth=1 origin {{sha}} ; git reset --hard {{sha}} ; git clean -f -q

website path:
  cargo run -p website -- linter-rules --table {{path}}/src/docs/guide/usage/linter/generated-rules.md --rule-docs {{path}}/src/docs/guide/usage/linter/rules --git-ref $(git rev-parse HEAD)
  cargo run -p website -- linter-cli > {{path}}/src/docs/guide/usage/linter/generated-cli.md
  cargo run -p website -- linter-schema-markdown > {{path}}/src/docs/guide/usage/linter/generated-config.md

minifier-diff:
  #!/usr/bin/env bash
  cargo minsize --compress-only pr
  git checkout main
  cargo minsize --compress-only main
  for file in antd bundle.min d3 echarts jquery lodash moment react.development three typescript victory vue
  do
      echo $file.js >> diff
      diff target/minifier/main/$file.js target/minifier/pr/$file.js >> diff
  done
  git checkout -
