# Coding agent guides for `crates/oxc_formatter_yaml`

Follow @../oxc_formatter_core/FORMATTER_POLICY.md , this file holds only the YAML-specific rules and translations.
Known divergences live in DIVERGENCES.md.

## Overview

Prettier compatible YAML formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- Entry points (see their docs in `src/format.rs`):
  - `format()` for standalone files, `format_to_ir()` for embedded use via the dispatcher (e.g. CSS front matter, JSDoc fenced blocks)
  - `parse_for_format()`: the parse `format()` runs, exposed for callers that inspect the AST (e.g. the fixture harness's fingerprint)

### Parser

Parses with [`oxc-yaml-parser`](https://crates.io/crates/oxc-yaml-parser).
Its AST follows `yaml-unist-parser`'s node naming (the nodes Prettier's printer operates on), with spans designed for layout work (see the parser's `ast` module docs)

One caveat the printer still owns: a block scalar's `span` consumes its trailing line breaks.
The split between the scalar's own output and the inter-item gap is printer policy, kept together in `src/print/block.rs` (see `consumed_trailing_newlines` / `item_gap_anchor`).

### Error semantics

The shared policy applies; YAML specifics:

- `oxc-yaml-parser` is fail-fast (no partial AST), so any syntax error is an `Err`
- Under-indented multi-line flow scalars (prettier#8602) are one such error
  - Prettier 3.9.6 also rejects them since its `yaml@2` upgrade, so the string corruption reported there cannot happen in either implementation

### Line endings

The source is normalized to `\n`-only BEFORE parsing (see `parse_root` for why).
The printer re-emits the configured `end_of_line` at the final stage.

A leading BOM is stripped before parsing and re-emitted by `format()`.

### Comments

Positional cursor (`Comments` in `src/comments.rs`), same approach as graphql/json, yaml-unist-parser's attach algorithm is NOT ported.

Placement is decided at print sites; the rules live as doc comments on the placement helpers, all in `src/comments.rs`.
Stream-tail end comments (`write_end_comments`) are the one document-layer exception, in `src/print/document.rs`.

## Verification

Manual checks:

```sh
cargo run -p oxc_formatter_yaml --example yaml_formatter [filename]
# Dump the formatter IR
DUMP_IR=1 cargo run -p oxc_formatter_yaml --example yaml_formatter [filename]
```
