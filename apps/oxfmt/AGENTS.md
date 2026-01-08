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
