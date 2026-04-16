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
# Update snapshots
pnpm t -u
# Run conformance test for xxx-in-js and js-in-xxx
pnpm conformance
# Run unit test in Rust
cargo t
```

To manually verify the CLI behavior after building:

```sh
pnpm build-test

# Show help
node ./dist/cli.js --help
# Stdin (`npx prettier --config=<cfg> <file>` equivalent)
cat <file> | node ./dist/cli.js --config=<cfg> --stdin-filepath=<file>
# With log
OXC_LOG=debug node ./dist/cli.js --threads=1 <file>
```

NOTE: `pnpm build-test` combines `pnpm build-js` and `pnpm build-napi`, so you don't need to run them separately.

To compare formatting output with Prettier:

```sh
# Use a shared config file (e.g., fmt.json) because oxfmt and Prettier have different default printWidth
# Example fmt.json: { "printWidth": 80 }
cat <file> | node ./dist/cli.js --config=fmt.json --stdin-filepath=<file>
npx prettier --config=fmt.json <file>
```

## Test Organization (`test/` directory)

Tests are organized into specific domains, each with its own structure.

### `test/api/`: Formatting result tests

Focuses on verifying formatting output. Use the Node.js API. No fixtures, test inputs are inline in each test file.

- Multiple `*.test.ts` files coexist in a flat directory (no subdirectories)
- Snapshots are colocated in `__snapshots__/` by Vitest

### `test/cli/`: CLI fixture-driven tests

A single `cli.test.ts` auto-discovers and runs all fixture directories via `utils.ts`.

- Each fixture directory contains:
  - `options.json` — array of test cases (args, cwd, env, stdin, etc.)
  - `fixtures/` — input files for the test cases
  - `*.snap.md` — file snapshots (one per test case, named `0.snap.md`, `1.snap.md`, …)
- Adding a new CLI test: create a new directory with `options.json` and `fixtures/`, then run the test to generate snapshots
- If exceptional test cases are required, place a separate `*.test.ts` file for them

### `test/lsp/`: LSP integration tests

Each test directory follows the 1:1:1 rule:

- 1 test file (`*.test.ts` with the same name as the directory)
- 0 or 1 `fixtures/` directory
- Snapshots are colocated in `__snapshots__/` by Vitest

Shared helpers are in `utils.ts` at the `test/lsp/` level.

## After updating `Oxfmtrc` (Under `src/core/oxfmtrc`)

When modifying the `Oxfmtrc` struct (and configuration options):

- Run `just formatter-schema-json` to update `npm/oxfmt/configuration_schema.json`
- Run `just formatter-config-ts` to regenerate `src-js/config.generated.ts` from the schema
- Run `cargo test -p website_formatter` to update schema markdown snapshots
  - Then, `cargo insta accept`
