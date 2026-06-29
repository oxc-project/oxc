# Coding agent guides for `crates/oxc_formatter_graphql`

## Overview

Prettier compatible GraphQL formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- This crate holds only the GraphQL-specific layer
- Two entry points:
  - `format()`: standalone files (returns a printable `Formatted`)
  - `format_to_ir()`: embedded use via the dispatcher (graphql-in-js); allocates
    from the shared `EmbeddedContext` arena, emits no BOM / trailing newline,
    and leaves `propagate_expand()` to the parent document
- Parses with [apollo-parser](https://docs.rs/apollo-parser) (rowan-based lossless CST)
  - Spec coverage: **October 2021 GraphQL spec only**
  - Prettier parses with `graphql-js`, which also accepts draft-level syntax
    (e.g. experimental fragment arguments, directives on directive definitions)
  - Such input makes `format()` return `Err`; `oxfmt` then falls back to Prettier (napi build)
- The canonical reference is Prettier's `src/language-graphql/printer-graphql.js`
  — port its layout decisions, do not invent new ones

### Error semantics

`format()` / `format_to_ir()` return `Err` whenever they cannot produce output they can stand behind:

- apollo-parser is error-tolerant (returns a CST even for invalid input),
  but any parse error bails out; never format a broken CST
- print-stage internal errors are also `Err`
- The caller (oxfmt) decides what happens next (report, or Prettier fallback)

### Comments

`graphql-js` does not attach comments to the AST;
Prettier collects them from the token stream and attaches leading/trailing/dangling per node.

This crate instead collects `COMMENT` trivia tokens from the CST into a positional cursor (`src/comments.rs`, mirrors `oxc_formatter_json`) and flushes them at sequence items, closing delimiters, and document tail.

`apollo-parser` attaches pending trivia to whichever node is open when the next significant token is consumed, so node ranges may start at a preceding comment.
**All layout decisions use significant-token positions** (`sig_start` / `sig_end` in `src/print/mod.rs`), never `text_range()` directly.

### Strings

Prettier prints `StringValue` from `graphql-js`'s _cooked_ value and re-encodes it.
apollo-parser hands us raw source, so `src/print/value.rs` reimplements:

- the GraphQL spec `BlockStringValue` algorithm (dedent + blank-line trimming)
- escape decoding for regular strings (incl. surrogate pairs)
- Prettier's re-encoding (`"`/`\` escaped, newline as `\n`, `"""` as `\"""`)

Blank-line runs inside block strings are part of the string VALUE;
the printer collapses consecutive line breaks, so they are emitted as raw `\n` text plus a `hard_line_break()` that only re-arms indentation (see `write_block_string_break`).

### Notable layout rules

- Blank-line preservation classifies the inter-token gap (`classify_gap`):
  a blank line is a whitespace-only line strictly inside the gap.
  Counting raw newlines would over-report when tokens (e.g. the `&` between two `implements` comments, or an insignificant comma) sit on their own line.
- A cooked `\r` escape in a string value is re-emitted as `\r`
  (Prettier emits a raw CR byte, which the core `text()` builder forbids; the string VALUE is identical).

## Verification

```sh
cargo c -p oxc_formatter_graphql
```

Run `clippy` and resolve all warnings.

### Fixtures tests

Snapshot tests driven by fixture files under `tests/fixtures/format/`,
covering what the Prettier conformance suite does not:
`# oxfmt-ignore` suppression, blank-line runs inside block strings,
string escape re-encoding (incl. the `\r` divergence), empty `[]` / `{}` values,
the full set of type-system extensions, insignificant-comma trivia,
trailing comments at various positions,
and width-overflowing `implements` lists (which never break).
`build.rs` auto-generates a test case from every `.graphql` file using the core
`test_support` harness. Unit tests in `tests/fixtures/mod.rs` cover parse-error
`Err` semantics and BOM preservation; `src/comments.rs` has `classify_gap` tests
(CR / CRLF endings, which `.gitattributes` keeps out of fixture files).

```sh
cargo test -p oxc_formatter_graphql
# Review / accept snapshots after intentional changes
cargo insta review -p oxc_formatter_graphql
```

Add a case by dropping a new `.graphql` file into the directory, the build script picks it up.

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes);
results live in `tasks/prettier_conformance/snapshots/prettier.graphql.snap.md`.
The `graphql` language is part of the shared conformance binary.

```sh
cargo run -p oxc_prettier_conformance
# Debug a specific test
cargo run -p oxc_prettier_conformance -- --filter graphql/<dir>/<file>
```

### Manual checks

```sh
cargo run -p oxc_formatter_graphql --example graphql_formatter [filename]
# Compare with Prettier
npx prettier --parser=graphql [filename]
```

A good large real-world stress input is GitHub's public GraphQL schema (~72k lines;
too large and third-party to commit as a fixture — its bug-catching shapes are
distilled into `tests/fixtures/format/implements-width.graphql`):

```sh
curl -sL https://docs.github.com/public/fpt/schema.docs.graphql -o /tmp/github-schema.graphql
diff <(npx prettier --parser=graphql /tmp/github-schema.graphql) \
  <(cargo run -q -p oxc_formatter_graphql --example graphql_formatter /tmp/github-schema.graphql)
```
