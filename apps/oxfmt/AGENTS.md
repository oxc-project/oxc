# Coding agent guides for `apps/oxfmt`

## Overview

The `oxfmt` implemented under this directory serves several purposes.

- Pure Rust CLI
  - Minimum feature set, CLI usage only
  - Build with `cargo build --no-default-features`
  - Entry point: `main()` in `src/main.rs`
- JS/Rust hybrid CLI using `napi-rs`
  - Full feature set like CLI, Stdin, LSP, and more
  - Build with `pnpm build`
  - Entry point: `src-js/cli.ts` which uses `run_cli()` from `src/main_napi.rs`
- Node.js API using napi-rs
  - Build with `pnpm build`
  - Entry point: `src-js/index.ts` which uses `format()` from `src/main_napi.rs`

When making changes, consider the impact on all paths.

## Verification

```sh
cargo c
cargo c --no-default-features
cargo c --features detect_code_removal
```

Also run `clippy` for the same configurations and resolve all warnings.

Run tests with:

```sh
# Run E2E
pnpm build-test && pnpm t

# Run unit test in Rust
cargo t
```

## Test Organization (`test/` directory)

Tests are organized by domain and colocated with strict structural rules.

- 1:1:1 Rule: Each test directory contains exactly
  - 1 test file (`*.test.ts` with the same name with directory)
  - 0 or 1 `fixtures/` directory (if needed)
  - Snapshots are colocated automatically by Vitest
- No Upward References (except `utils.ts` and `oxfmt` binary)
  - Test files may only reference:
    - Files within their own directory
    - Shared `utils.ts` in parent directories

When adding new tests:

- Place test in the appropriate domain directory
- If the test needs fixtures, create a `fixtures/` subdirectory
- If multiple test cases share a fixture structure, use subdirectories within `fixtures/` (e.g., `fixtures/basic/`, `fixtures/nested/`)

## `Oxfmtrc` Configuration (`src/core/oxfmtrc.rs`)

When modifying the `Oxfmtrc` struct (configuration options):

1. Update `src-js/index.ts` types to match the Rust struct
2. Run `just formatter-schema-json` to update `npm/oxfmt/configuration_schema.json`
3. Run `cargo test -p website_formatter` to update schema markdown snapshots
