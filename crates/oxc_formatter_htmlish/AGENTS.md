# Coding agent guides for `crates/oxc_formatter_htmlish`

Follow @../oxc_formatter_core/FORMATTER_POLICY.md , this file holds only the htmlish-specific rules.
Known divergences live in DIVERGENCES.md.

## Overview

Prettier compatible formatter for HTML and the template languages built on it (Vue, Angular, ...), on `oxc_formatter_core`.

### Parser

Parses with [`oxc-htmlish-parser`](https://github.com/oxc-project/oxc-htmlish-parser).
Parse behavior and its own divergences are that repo's `AGENTS.md` / `DIVERGENCES.md`.

### Dialects

What a dialect changes in the shared rules is a `DialectRules` field (`src/rules.rs`), resolved once per document;
a dialect adds a field there, not a `Dialect` check at the use site.

### Preprocess

The facts Prettier's `print-preprocess` computes (CSS display, whitespace sensitivity, kept children, borrowed tag markers)
are a table keyed by `NodeId` (`src/tree/`).
The AST is never rewritten.

### Tags

An end tag the source omits is never added (DIVERGENCES.md#unclosed-at-end, DIVERGENCES.md#omitted-end-tag).
A normalization is applied only where it is known safe for every reader, as in `oxc_formatter_css` (its AGENTS.md "Case"):
a void element prints `/>`, a doctype `<!doctype html>` but in XHTML, which an XML parser may read (DIVERGENCES.md#doctype-case),
and the tag and attribute names Prettier knows print lowercase in a `.html` file only (DIVERGENCES.md#tag-name-case).

The borrow rules (`src/print/tag.rs`) are shaped like Prettier's `print/tag.js` against the policy, kept deliberately (why: its module doc).

## Verification

```sh
cargo run -p oxc_formatter_htmlish --example htmlish_formatter [filename]
DUMP_IR=1 cargo run -p oxc_formatter_htmlish --example htmlish_formatter [filename]

# The oracle. The parser follows the file name as the example's variant does (`*.component.html` is angular).
node apps/oxfmt/node_modules/prettier/bin/prettier.cjs [filename]
```

### Prettier conformance

`tests/conformance.rs` runs `tests/format/html`.
Files whose output depends on embedded formatting (`<script>` / `<style>` bodies, `style` / `on*` attributes, a YAML front matter body) are ignored here and covered by oxfmt's E2E conformance;
the ignore list names the reason for each.

Every failing file is accounted for by a DIVERGENCES.md entry's `Conformance:` line; there are no unclassified failures.

Fixtures and the example read `.vue` as Vue, `.xhtml` / `.xht` as XHTML and `*.component.html` as Angular.
