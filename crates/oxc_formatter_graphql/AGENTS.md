# Coding agent guides for `crates/oxc_formatter_graphql`

Follow @../oxc_formatter_core/FORMATTER_POLICY.md , this file holds only the GraphQL-specific rules and translations.

## Overview

Prettier compatible GraphQL formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- Two entry points:
  - `format()`: standalone files (returns a printable `Formatted`)
  - `format_to_ir()`: embedded use via the dispatcher (e.g. graphql-in-js)

### Forked parser

Prettier uses `graphql-js`, which is also the reference implementation of the GraphQL specification.
This means it is not locked to a specific GraphQL version and can parse a wide range of syntaxes, including draft syntax.

On the other hand, `apollo-parser`, the upstream we selected, strictly follows its versioning and currently only supports syntax up to Oct2021.
Therefore, we forked it as [`oxc-graphql-parser`](https://crates.io/crates/oxc-graphql-parser) and added some support ourselves, pinned via the workspace `Cargo.toml`.

The fork aligns with graphql-js v17.x, which Prettier 3.9 targets:

- executable descriptions (Sep2025 spec) and directive extensions (`extend directive`) are always-on
- fragment arguments (`...F(size: $size)`) are behind the `experimental_fragment_arguments` flag
  - which also covers the legacy `fragment F($x: Int) on T` definition-side syntax

### Error semantics

The shared policy applies; GraphQL specifics:

- `oxc-graphql-parser` is error-tolerant (returns an AST even for invalid input), but any parse error still bails out
  - Several printer shortcuts (e.g. `close_delim_start`) are sound only under this guarantee

### Comments

`graphql-js` does not attach comments to the AST;
Prettier collects them from the token stream and attaches leading/trailing/dangling per node.

This crate instead collects comment spans into a positional cursor, drained in source order by claim points spread through the printers
(the `flush_*` helpers in `src/comments.rs`; behavior pinned by `tests/fixtures/graphql/comments-inside-node-spans.graphql`).

The shared placement invariants apply: a comment stays between the source tokens it sat between, and a same-line trailing comment stays on its line.

Two bounded exceptions:

- an own-line comment claimed right after a printed literal (`type`, `:`, `=`) inlines on that literal's line
  - `type` + break + `# c` + break + `A` prints as `type # c` + break + `A`
  - identical to Prettier and to `oxc_formatter`'s `const // c` + break + `a = 1`; keeping it own-line would need a column-conditional break the IR does not have
- positions no printer claims fall back to an own-line trailing comment after the node, which may cross remaining in-span tokens (e.g. a type's `!`)
  - `flush_overlooked_inside_comments`

Where Prettier relocates a comment across tokens instead, we diverge, see "Known divergences".

Two constraints the code cannot show:

- the cursor is monotonic: every claim point must drain everything inside the span it just printed
  - or a later flush point's gap range inverts and panics (issue #24927)
- trailing claims are bounded at the literal's SOURCE position
  - or re-formatting is not idempotent (`g: # c` must not become `g # c` + `:`)

Node spans are significant-token spans (trivia is never included), so layout decisions use them directly.
All span bridging (conversion, closing-delimiter derivation, the bare-token source scan) lives in `src/print/span.rs`.

### Strings

Prettier prints `StringValue` from `graphql-js`'s _cooked_ value and re-encodes it.
`oxc-graphql-parser`'s `StringValue.value` is cooked but not to spec (no block-string dedent / blank-line trimming, no surrogate pairing), so `src/print/string.rs` cooks from `raw` itself:

- the GraphQL spec `BlockStringValue` algorithm (dedent + blank-line trimming)
- escape decoding for regular strings (incl. surrogate pairs)
- Prettier's re-encoding (`"`/`\` escaped, newline as `\n`, `"""` as `\"""`)

Blank-line runs inside block strings are part of the string VALUE and are emitted with `exact_line_breaks()`. Values are written pre-trimmed, the core printer never trims.

### Notable layout rules

- Blank-line preservation classifies the inter-token gap (`classify_gap`):
  a blank line is a whitespace-only line strictly inside the gap.
  Counting raw newlines would over-report when tokens (e.g. the `&` between two `implements` comments, or an insignificant comma) sit on their own line.
- A cooked `\r` escape in a string value is re-emitted as `\r`
  (Prettier emits a raw CR byte, which the core `text()` builder forbids; the string VALUE is identical).

## Known divergences

Admission reasons and rules: see FORMATTER_POLICY.md "Known divergences".
All current entries are one class: Prettier relocates a comment (an attachment artifact of `graphql-js` node boundaries, not a layout rule), we keep it between its source tokens on its source line.

- `"desc" type # c`: Prettier pulls the comment backwards across the keyword onto the description's line
- `"""d"""` + break + `# c` + break + `type A`: Prettier pushes the comment forward across the keyword (`type # c` + break + `A`)
- `type A # c` + break + `implements B`: Prettier scatters it to the line end (`type A implements B { # c`);
  - same class: `f(x) # c` + break + `: T` is pulled inside the parens
- `{ # c` after an opening delimiter: Prettier moves it own-line as the first child's leading (asymmetric: `test # c` / `} # c` stay inline)

## Verification

Manual checks:

```sh
cargo run -p oxc_formatter_graphql --example graphql_formatter [filename]
```

A good large real-world stress input is GitHub's public GraphQL schema (~72k lines).
It is too large and third-party to commit as a fixture, bug-catching shapes are distilled into `tests/fixtures/graphql/implements-width.graphql`):

```sh
curl -sL https://docs.github.com/public/fpt/schema.docs.graphql -o /tmp/github-schema.graphql
diff <(node apps/oxfmt/node_modules/prettier/bin/prettier.cjs --parser=graphql /tmp/github-schema.graphql) \
  <(cargo run -q -p oxc_formatter_graphql --example graphql_formatter /tmp/github-schema.graphql)
```
