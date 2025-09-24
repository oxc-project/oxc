#!/usr/bin/env -S just --justfile

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["bash", "-cu"]

_default:
  @just --list -u

# ==================== ALIASES ====================
alias r := ready
alias c := conformance
alias f := fix

# ==================== SETUP & INITIALIZATION ====================

# Initialize the project by installing all necessary tools
init:
  # Rust related init
  cargo binstall watchexec-cli cargo-insta typos-cli cargo-shear dprint -y
  # Node.js related init
  pnpm install

# Clone or update submodules
submodules:
  just clone-submodule tasks/coverage/test262 https://github.com/tc39/test262 baa48a416c9e9abd698a9010378eccf3d1f4ed1e
  just clone-submodule tasks/coverage/babel https://github.com/babel/babel 41d96516130ff48f16eca9f387996c0272125f16
  just clone-submodule tasks/coverage/typescript https://github.com/microsoft/TypeScript 261630d650c0c961860187bebc86e25c3707c05d
  just clone-submodule tasks/prettier_conformance/prettier https://github.com/prettier/prettier 7584432401a47a26943dd7a9ca9a8e032ead7285
  just clone-submodule tasks/coverage/acorn-test262 https://github.com/oxc-project/acorn-test262 090bba4ab63458850b294f55b17f2ca0ee982062
  just clone-submodule tasks/coverage/node-compat-table https://github.com/williamkapke/node-compat-table ed0d6ba55790519d9ad3f6f776ca2cd303cc1e0b
  just update-transformer-fixtures

# Install git pre-commit hook to format files
install-hook:
  echo -e "#!/bin/sh\njust fmt" > .git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit

# ==================== CORE DEVELOPMENT ====================

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

# Fix all auto-fixable format and lint issues
fix:
  cargo clippy --fix --allow-staged --no-deps
  just fmt
  typos -w
  git status

# ==================== DEVELOPMENT TOOLS ====================

watch *args='':
  watchexec --no-vcs-ignore {{args}}

watch-check:
  just watch "'cargo check; cargo clippy'"

watch-example *args='':
  just watch 'just example {{args}}'

# Run examples in parser, formatter, linter
example tool *args='':
  cargo run -p oxc_{{tool}} --example {{tool}} -- {{args}}

# Run the benchmarks
benchmark:
  cargo benchmark

# Run benchmarks for a single component
benchmark-one *args:
  cargo benchmark --bench {{args}} --no-default-features --features {{args}}

# ==================== TESTING & CONFORMANCE ====================

# Run all conformance tests
coverage:
  cargo coverage
  cargo run -p oxc_transform_conformance -- --exec
  cargo run -p oxc_prettier_conformance

# Run Test262, Babel and TypeScript conformance suite
conformance *args='':
  cargo coverage -- {{args}}

# Test ESTree
test-estree *args='':
  cargo run -p oxc_coverage --profile coverage -- estree {{args}}

# Get code coverage
codecov:
  cargo codecov --html

# ==================== AST & CODEGEN ====================

# Generate AST related boilerplate code
ast:
  cargo run -p oxc_ast_tools
  just check

# ==================== PARSER ====================

# Parser-specific commands will be added here as needed

# ==================== LINTER ====================

# Build oxlint in release build
oxlint:
  pnpm -C apps/oxlint run build

watch-oxlint *args='':
  just watch 'pnpm run -C apps/oxlint build-dev && node apps/oxlint/dist/cli.js --disable-nested-config {{args}}'

# Create a new lint rule for any plugin
new-rule name plugin='eslint':
  cargo run -p rulegen {{name}} {{plugin}}

# Legacy aliases for backward compatibility
new-jest-rule name: (new-rule name "jest")
new-ts-rule name: (new-rule name "typescript")
new-unicorn-rule name: (new-rule name "unicorn")
new-import-rule name: (new-rule name "import")
new-react-rule name: (new-rule name "react")
new-jsx-a11y-rule name: (new-rule name "jsx-a11y")
new-oxc-rule name: (new-rule name "oxc")
new-nextjs-rule name: (new-rule name "nextjs")
new-jsdoc-rule name: (new-rule name "jsdoc")
new-react-perf-rule name: (new-rule name "react-perf")
new-n-rule name: (new-rule name "n")
new-promise-rule name: (new-rule name "promise")
new-vitest-rule name: (new-rule name "vitest")
new-regexp-rule name: (new-rule name "regexp")
new-vue-rule name: (new-rule name "vue")

# Alias for backward compatibility
alias new-typescript-rule := new-ts-rule

# ==================== FORMATTER ====================

# Formatter-specific commands will be added here as needed

# ==================== TRANSFORMER ====================

# Test Transform
test-transform *args='':
  cargo run -p oxc_transform_conformance -- --exec {{args}}

# Update transformer conformance test fixtures
update-transformer-fixtures:
  cd tasks/coverage/babel; git reset --hard HEAD; git clean -f -q
  node tasks/transform_conformance/update_fixtures.mjs

# ==================== MINIFIER ====================

# Update minifier size snapshots
minsize:
  cargo minsize
  just allocs

# Update memory allocation snapshots
allocs:
  cargo allocs

# Generate minifier size comparison
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

# ==================== PLAYGROUND ====================

# Install wasm32-wasip1-threads for playground
install-wasm:
  rustup target add wasm32-wasip1-threads

build-playground:
  pnpm --filter oxc-playground build

watch-playground:
  just watch 'pnpm --filter oxc-playground build-dev'

# ==================== UTILITIES & ADVANCED ====================

# Generate website documentation
website path:
  cargo run -p website -- linter-rules --table {{path}}/src/docs/guide/usage/linter/generated-rules.md --rule-docs {{path}}/src/docs/guide/usage/linter/rules --git-ref $(git rev-parse HEAD)
  cargo run -p website -- linter-cli > {{path}}/src/docs/guide/usage/linter/generated-cli.md
  cargo run -p website -- linter-schema-markdown > {{path}}/src/docs/guide/usage/linter/generated-config.md

# Automatically DRY up Cargo.toml manifests in a workspace
autoinherit:
  cargo binstall cargo-autoinherit
  cargo autoinherit

# ==================== PLATFORM HELPERS ====================

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
