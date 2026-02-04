# Coding agent guides for `apps/oxfmt`

## Overview

The `oxfmt` implemented under this directory serves several purposes.

- Pure Rust CLI
  - Minimum feature set, CLI usage only, no LSP, no Stdin support
  - Formats JS/TS and TOML files, no xxx-in-js support
  - Entry point: `main()` in `src/main.rs`
  - Build with `cargo build --no-default-features`
- JS/Rust hybrid CLI using `napi-rs`
  - Full feature set like CLI, Stdin, LSP, and more
  - Format many file types with embedded language formatting support
  - Entry point: `src-js/cli.ts` which uses `run_cli()` from `src/main_napi.rs`
  - Build with `pnpm build`
- Node.js API using napi-rs
  - Entry point: `src-js/index.ts` which uses `format()` from `src/main_napi.rs`
  - Build with `pnpm build`

When making changes, consider the impact on all paths.

## Platform Considerations

Oxfmt is built for multiple platforms (Linux, macOS, Windows) and architectures.

When working with file paths in CLI code, be aware of Windows path differences:

- Use `std::path::Path` / `PathBuf` instead of manual string manipulation with `/`
- Be cautious with path comparisons and normalization across platforms
  - Avoid hardcoding `/` as a path separator; prefer `Path::join()`
  - Windows uses `\` as a path separator and has drive letter prefixes (e.g., `C:\`)

## Verification

```sh
cargo c
cargo c --no-default-features
cargo c --features detect_code_removal
```

Also run `clippy` for the same configurations and resolve all warnings.

Run tests with:

```sh
# Run E2E test
pnpm build-test && pnpm t
# Run unit test in Rust
cargo t
```

To manually verify the CLI behavior after building:

```sh
pnpm build-test

# Show help
node ./dist/cli.js --help
# Stdin
cat <file> | node ./dist/cli.js --stdin-filepath=<file>
```

NOTE: `pnpm build-test` combines `pnpm build-js` and `pnpm build-napi`, so you don't need to run them separately.

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

## After updating `Oxfmtrc` (`src/core/oxfmtrc.rs`)

When modifying the `Oxfmtrc` struct (configuration options):

- Also update `src-js/index.ts` types to match the Rust struct if needed
- Run `just formatter-schema-json` to update `npm/oxfmt/configuration_schema.json`
- Run `cargo test -p website_formatter` to update schema markdown snapshots
  - Then, `cargo insta accept`
