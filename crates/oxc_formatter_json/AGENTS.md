# Coding agent guides for `crates/oxc_formatter_json`

## Overview

Prettier compatible JSON/JSONC/JSON5 formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- This crate holds only the JSON-specific layer
- Parses with `oxc_parser`, not `serde_json`
  - For Prettier, JSON is not spec compliant JSON
  - They are subsets of JS expression syntax, so the comments, unquoted key, etc... are allowed as input
- A simplified, independent reimplementation that does not share `oxc_formatter`'s code
  - As a result, although Prettier's JSON (especially JSON5) behaves like JS, as a formatter implementation, they should be distinguished and kept from interfering with each other
  - So, just use `oxc_formatter` (`crates/oxc_formatter/`) as the canonical reference
    - For layout / comment / blank-line decisions when the simplified version is unclear or diverges from Prettier
  - The narrow JSON grammar pays off in speed as well
    - It is ~1.4–2.3x faster than routing the same input through `oxc_formatter` (wrapped in `(...)`)
    - Since the JS path carries expression-kind dispatch, parens, and trivia overhead
    - The gap widens for structure-heavy input and narrows for string-heavy input

### `JsonVariant`

- Json
- Jsonc
- Json5
- JsonStringify

All variants share lenient parsing (comments, trailing commas, single quotes, unquoted keys all parse regardless of variant).
What differs is the output formatting.

Parsing always uses `SourceType::default()` (JS); `variant` only gates comment validation (see `parse.rs`), never the lexis.

- So JS lexer rules including line terminators U+2028 / U+2029 apply to every variant's input, not just JSON5
- Consequence: downstream source scans (newline / blank-line detection) must be LS/PS-aware for all variants
  - Strictly LS/PS are line terminators only in JSON5, but a `json` / `jsonc` input can still carry them in inter-token gaps
  - Because the lenient JS parse accepts them (also matching Prettier, which routes every variant through its JS printer)

See the doc comments on `JsonVariant` in `src/options.rs` for the per-variant rules.

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
