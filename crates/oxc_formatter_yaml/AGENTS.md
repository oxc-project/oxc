# Coding agent guides for `crates/oxc_formatter_yaml`

## Overview

Prettier compatible YAML formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- Two entry points (see their docs in `src/format.rs`):
  `format()` for standalone files, `format_to_ir()` for embedded use via the dispatcher (e.g. yaml-in-markdown)
- The canonical reference is Prettier 3.9's OUTPUT (the conformance snapshots); its source is an analysis aid, not a porting target
  - match its layout decisions — EXCEPT where they are bugs or internally inconsistent (see below)
  - never mirror its internal logic 1:1: pin behavior with fixtures, and keep implementation details of Prettier's code out of comments

### Parser

Parses with [`oxc-yaml-parser`](https://crates.io/crates/oxc-yaml-parser).
Its AST follows `yaml-unist-parser`'s node naming (the nodes Prettier's printer operates on), with spans designed for layout work (see the parser's `ast` module docs)

One caveat the printer still owns: a block scalar's `span` consumes its trailing line breaks.
The split between the scalar's own output and the inter-item gap is printer policy, kept together in `src/print/block.rs` (see `consumed_trailing_newlines` / `item_gap_anchor`).

### Error semantics

`oxc-yaml-parser` is fail-fast (no partial AST), so `format()` / `format_to_ir()` return `Err` on any syntax error and never format a broken AST.
The caller (oxfmt) decides what happens next.

Under-indented multi-line flow scalars (prettier#8602) are one such error.
Prettier 3.9.5 also rejects them since its `yaml@2` upgrade, so the string corruption reported there cannot happen in either implementation.

### Line endings

The source is normalized to `\n`-only BEFORE parsing (see `parse_root` for why).
The printer re-emits the configured `end_of_line` at the final stage.

A leading BOM is stripped before parsing and re-emitted by `format()`.

### Comments

Positional cursor (`Comments` in `src/comments.rs`), same approach as graphql/json, yaml-unist-parser's attach algorithm is NOT ported.

Placement is decided at print sites; the rules live as doc comments on the placement helpers, all in `src/comments.rs`.
Stream-tail end comments (`write_end_comments`) are the one document-layer exception, in `src/print/document.rs`.

## Known divergences

Follow Prettier by default.
The exceptions (shared policy across all the formatter crates), are the cases where consistent output beats conformance percentage:

- Prettier bugs acknowledged as open issues
- behaviors that are internally inconsistent (same construct, different output depending on node kind or context)

Affected conformance fixtures stay counted as failures.
Style debates (`status:needs discussion` issues) are still followed, do not "improve" on taste for now.

Current divergences:

- anchor/tag order (prettier#19524): source order is preserved, never reordered
- `# prettier-ignore` range (prettier#13008): suppresses exactly one node, never every following node
- anchor next-line comments (prettier#10518 / #9327): structurally avoided, the positional cursor makes them the next node's leading comments
- blank lines (prettier#15528): one unified rule:
  a blank line right after a node is preserved (normalized to one) if the source had one, never invented, identical for every node kind and context.
  Prettier's matrix (block collections only between documents; mappings only before end comments; unconditional insertion after block scalars) is not ported.
  This also keeps `proseWrap: never` idempotent where Prettier is not (prettier#10776)
- folded scalar more-indented lines (prettier#16126): never re-flowed under `proseWrap: always`, their line breaks are literal per YAML folding,
  so Prettier's wrapping at the print width changes the parsed value and breaks idempotency
- "broken but not broken" flow collections: Prettier sometimes emits a newline inside flow brackets while keeping them flat (no trailing comma, `]`/`}` on the content line).
  multiline pairs (spec-example-7-20 / 9-4) and key trailing comments.
  Here a flow collection either fits on one line or breaks normally.
- comment position (spec-example-6-1): a comment stays at its syntactic position; Prettier hoists a comment after `[` onto the `key:` line
- trailing comment width (`key: | # ...`): a same-line trailing comment is a `line_suffix` and never counts toward the `fits` measurement
  the same treatment Prettier itself gives JS/JSON line comments and yaml flow collections.
  Prettier's yaml printer measures the one after a block scalar header inline and breaks the key line (`key:\n  | # ...`); that break is not ported

## Verification

```sh
cargo c -p oxc_formatter_yaml
```

Run `clippy` and resolve all warnings.

### Fixture tests

Snapshot tests driven by fixture files under `tests/fixtures/yaml/`, covering what the Prettier conformance suite does not (`# oxfmt-ignore`, divergence shapes, etc).
`build.rs` auto-generates a test case from every `.yaml` file using the core `test_support` harness; add a case by dropping a new file into the directory.

```sh
cargo test -p oxc_formatter_yaml
# Review / accept snapshots after intentional changes
cargo insta review -p oxc_formatter_yaml
```

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes); results live in `tasks/prettier_conformance/snapshots/prettier.yaml.snap.md`.
The `yaml` language is part of the shared conformance binary.

```sh
cargo run -p oxc_prettier_conformance
# Debug a specific test
cargo run -p oxc_prettier_conformance -- --filter yaml/<dir>/<file>
```

Failures must be either fixed or classified: a new failure is acceptable only when it falls under the non-follow policy above, and it must be documented there.

### Manual checks

```sh
cargo run -p oxc_formatter_yaml --example yaml_formatter [filename]
# Dump the formatter IR
DUMP_IR=1 cargo run -p oxc_formatter_yaml --example yaml_formatter [filename]
# Compare with Prettier
npx prettier --parser=yaml [filename]
```
