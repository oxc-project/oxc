# Coding agent guides for `crates/oxc_formatter_json`

## Overview

Prettier compatible JSON/JSONC/JSON5 formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- This crate holds only the JSON-specific layer
- Parses with `oxc_parser`, not `serde_json`
  - For Prettier, JSON is not spec compliant JSON
  - They are subsets of JS expression syntax, so the comments, unquoted key, etc... are allowed as input

### `JsonVariant`

All variants share lenient parsing (comments, trailing commas, single quotes, unquoted keys all parse regardless of variant).

What differs is the output formatting. See the doc comments on `JsonVariant` in `src/options.rs` for the per-variant rules.

## Verification

```sh
cargo c -p oxc_formatter_json
```

Run `clippy` and resolve all warnings.

### Fixtures tests

Snapshot tests driven by fixture files under `tests/fixtures/json/`.
`build.rs` auto-generates a test case from every `.{json,jsonc,json5}` file using the core `test_support` harness.

```sh
cargo test -p oxc_formatter_json
# Review / accept snapshots after intentional changes
cargo insta review -p oxc_formatter_json
```

Add a case by dropping a new file into `tests/fixtures/json/`, the build script picks it up.

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes); results live in `tasks/prettier_conformance/snapshots/`. The `json` language is part of the shared conformance binary.

```sh
cargo run -p oxc_prettier_conformance
```
