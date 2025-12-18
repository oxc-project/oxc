# Coding agent guides for `apps/oxfmt`

## Overview

`oxfmt` has multiple entry points:

- Pure Rust CLI
  - Entry point: `main()` in `src/main.rs`
- JS/Rust hybrid CLI using napi-rs
  - Entry point: `src-js/cli.ts` which uses `run_cli()` from `src/main_napi.rs`
- Node.js API using napi-rs
  - Entry point: `src-js/index.ts` which uses `format()` from `src/main_napi.rs`

When making changes, consider the impact on all paths.

## Verification

```sh
cargo c
cargo c --no-default-features
cargo c --features detect_code_removal
```

Also run clippy for the same configurations and resolve all warnings.

Run tests with:

```sh
pnpm build-test
pnpm t
cargo t
```
