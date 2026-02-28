## Stage 1: oxfmt JSDoc Comment Formatting (#19702)

**Goal**: Implement core JSDoc comment formatting natively in oxfmt
**Status**: Complete

### Context

Issue [oxc-project/oxc#19702](https://github.com/oxc-project/oxc/issues/19702) — implement [prettier-plugin-jsdoc](https://github.com/hosseinmd/prettier-plugin-jsdoc) functionality natively in oxfmt. Filed by Boshen, milestone: Oxfmt 1.0.

### Key Design Decision: String-based formatting

Uses `StringBuilder` (not FormatElement IR) for the comment body. Rationale:

- The formatter's IR (groups, indents, soft breaks) is designed for code layout, not comment text
- Comments are already a single text element by the time the printer sees them
- The existing non-alignable multiline comment path in `trivia.rs` already uses `StringBuilder`
- Prettier's own JSDoc plugin also works at the string level

### Files Modified

1. `crates/oxc_formatter/Cargo.toml` — Added `oxc_jsdoc` dependency
2. `crates/oxc_jsdoc/src/parser/jsdoc_parts.rs` — Added `parsed_preserving_whitespace()` and `raw()` methods
3. `crates/oxc_formatter/src/options.rs` — Added `JsdocOptions` struct and field on `FormatOptions`
4. `crates/oxc_formatter/src/formatter/mod.rs` — Registered `jsdoc` module
5. `crates/oxc_formatter/src/formatter/trivia.rs` — Integrated JSDoc formatting in `Comment::fmt()`
6. `crates/oxc_formatter/tests/fixtures/mod.rs` — Added `"jsdoc"` option parsing
7. `apps/oxfmt/src/core/oxfmtrc.rs` — Added `jsdoc: Option<bool>` config field

### Files Created

- `crates/oxc_formatter/src/formatter/jsdoc/mod.rs` — Module entry point
- `crates/oxc_formatter/src/formatter/jsdoc/normalize.rs` — Tag normalization, capitalization, type whitespace
- `crates/oxc_formatter/src/formatter/jsdoc/wrap.rs` — Word wrapping with structured content preservation
- `crates/oxc_formatter/src/formatter/jsdoc/serialize.rs` — Core formatting logic
- 9 test fixtures in `crates/oxc_formatter/tests/fixtures/js/jsdoc/`

### Features

- Tag alias normalization (`@return`→`@returns`, `@arg`→`@param`, `@yield`→`@yields`, etc.)
- Description capitalization (skips backtick-prefixed text)
- Type whitespace normalization (`{  string  |  number  }` → `{string | number}`)
- Single-line collapse for short tags
- Empty JSDoc removal
- Word wrapping to `printWidth`
- Structured content preservation (markdown lists, code fences, tables, headings)
- `@example` content preserved verbatim
- Optional param brackets preserved (`[name]`, `[name="default"]`)
- Consecutive same-kind tags grouped without blank lines
- Different tag kinds separated by blank lines

### Verification

- `cargo test -p oxc_formatter` — 202 tests pass (9 new jsdoc tests)
- `cargo test -p oxc_jsdoc` — 9 tests pass
- `cargo clippy -p oxc_formatter` — no new warnings
- `cargo check -p oxfmt --no-default-features` and `cargo check -p oxfmt` — clean

### Future Work (separate PRs)

- Tag sorting by configurable weight
- Vertical alignment option
- `@example` code formatting (recursive — reuse embedded language infrastructure)
- Additional options (`trailing_dot`, `bracket_spacing`, `jsdoc_print_width`, etc.)
- Sub-options on `JsdocOptions` (when struct is already in place, easy to extend)
