# Coding agent guides for `crates/oxc_formatter_css`

## Overview

Prettier compatible CSS/SCSS/Less formatter, using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
- Parses with [raffia](https://docs.rs/raffia) 0.12 (AST with full spans + `raw` text; NOT a CST)
  - Comments are NOT in the AST: collected via `ParserBuilder::comments()` into
    a positional cursor (`src/comments.rs`, mirrors `oxc_formatter_graphql`)
  - The canonical reference is Prettier's `src/language-css/printer-postcss.js`
    (in `tasks/prettier_conformance/prettier`, v3.8.x) — port its layout
    decisions, do not invent new ones
- Two entry points: `format()` (standalone) and `format_to_ir()` (embedded/dispatcher)

### Error semantics

`format()` / `format_to_ir()` return `Err` on ANY parse error, including
raffia's recoverable ones (`parser.recoverable_errors()`); oxfmt then falls
back to Prettier (napi build).

### Architecture notes (Prettier mapping)

Prettier's CSS printer operates on postcss + three sub-parsers
(postcss-selector-parser → `selector-*`, postcss-values-parser → `value-*`,
postcss-media-query-parser → `media-*`) and depends on `raws` (source gaps).
raffia parses everything structurally in one pass; source gaps are recovered
by comparing span boundaries (`hasEmptyRawBefore(x)` == "no gap between spans").

- `src/print/mod.rs` — statement sequences (hardline-separated, one blank line
  preserved via `classify_gap`), trailing same-line comments
- `src/print/statement.rs` — qualified rules, declarations, blocks, dispatch.
  `stmt_end()` extends spans over whitespace/comments + `;` (postcss `locEnd`
  includes the semicolon; blank-line detection counts from after it)
- `src/print/value.rs` — the port of `printCommaSeparatedValueGroup` /
  `printParenthesizedValueGroup` over flat `ComponentValue` streams
  (raffia keeps `Delimiter` commas/solidi inline, like postcss-values tokens).
  Key rules ported: solidus tightness (font sizes, leading `/`), grid hardlines
  (+ leading hardline when the source breaks), `printNumber`/`printCssNumber`,
  `printString` re-quoting, CSS_UNITS canonical casing, wide-keywords/hex
  lowercase, `composes` removeLines, `progid:` verbatim, url() inner verbatim
- `src/print/selector.rs` — selectors; combinators carry the break point
  BEFORE themselves; `maybeToLowerCase` for pseudos; attribute values are
  quoted via `printString`
- `src/print/at_rule.rs` — prelude dispatch; media query port; TokenSeq
  fallback printing (gap-based separators, break AFTER math operators);
  "fused" preludes (`@page:first` stays tight when the source has no gap);
  SCSS control directives wrap `[space, prelude, line]` in a group so `{`
  drops to its own line when the prelude breaks — EXCEPT fully parenthesized
  conditions (Prettier's `hasParensAroundNode` → `{` stays on the `)` line)
- `src/print/scss.rs` — `$var` declarations, maps (always break, one item per
  line, trailing comma per option), lists, `@each`/`@for`/`@if` chains
  (`} @else` joined), mixin/include/function params, `@use`/`@forward` with
  always-broken `with (...)` configs
- `src/print/less.rs` — `@var` declarations, mixin definitions/calls, guards,
  lookups (`[@result]` tight), detached rulesets

### Comments

Positional cursor over `CssComment { span, inline }` (`inline` = `//`).

- Statement-level comments: flushed before each statement
  (`flush_leading_comments`); consecutive same-line comments stay glued
  (`*/ /*!`), but a comment is always followed by a line break before a node
- Value-level comments: flushed inside fill entries before the component they
  precede (`flush_value_comments`); `//` comments expand the parent group and
  force a hardline after
- Trailing (`value /* c */;`): flushed by `write_declaration` with the source
  gap before `;` preserved
- After each statement, the sequence DISCARDS unclaimed comments inside the
  statement span (cursor must never point before a printed position)

## Status (2026-06-11)

`cargo run -p oxc_prettier_conformance` —
**css 114/114, scss 85/85, less 39/39 — ALL 100%.**
Keep it that way: any change here must re-run the conformance suite.

Layout machinery notes discovered en route:

- Our core `fill` breaks the separator AFTER a hard-broken entry (biome
  semantics) and measures fits only up to a hardline; Prettier's fill
  fit-checks `[item, sep, next-item]` and treats a hardline-bearing chunk
  as never fitting. Where that diverges, separator breaks are SIMULATED
  with static source widths (`write_commented_value_params` /
  `write_commented_media_params` in `at_rule.rs`, the SassImport path,
  the lead-comment fill in `write_value_groups`)
- Prettier's printer counts a multi-line string doc at its FULL width (no
  newline reset), so after a multi-line `raws.between` the first trailing
  comment always wraps → `ValueContext::tail_break`
- `css-decl` prints the WHOLE trimmed `raws.between` (prop → value, colon
  and comments included) verbatim; a trailing `//` line drops the value to
  `indent([hardline, dedent(value)])`; same-line space runs before `//`
  collapse to one (postcss-less keeps inline comments out of between)
- At-rule params containing block comments are rebuilt from the source
  (postcss keeps them inside the params string): `@keyframes`-style names →
  whitespace-normalized verbatim; `@media` → media-token reconstruction
  (`)` ends a single-line paren token; spaced `feature : value` re-spaces,
  glued/multi-line stays verbatim); `@import`/`@supports` → value-token
  fill simulation with always-broken comment-bearing parens. `//` comments
  stay on the structural printers
- Selectors containing `//` comments are `selector-unknown` in Prettier:
  raw verbatim, `{` pushed to the next line after a trailing `//`
- SCSS control-directive conditions are `group(indent(parts))`, NOT a fill:
  space before every operator, breakable line after it, all-or-nothing
  (`write_condition_chain`); source-glued `$a==b` stays glued
- `isSCSSMapItemNode` is ported as two ctx flags: `map_break` (SassMap in
  `$var:`/function-arg/map-item positions always breaks) and `paren_break`
  (paren groups break ONLY as direct map-item/config values); outside those
  positions maps stay inline, preserve source blank lines between items,
  and print no trailing comma
- A function call directly after a `//` comment gets Prettier's quirky
  double indent (args +2 levels, `)` +1 — `ValueContext::after_inline_comment`)
- An interpolated string whose outer quote re-appears inside (`'#{f('a')}'`)
  splits in postcss → every piece requotes to the preferred quote
- YAML front matter: best-effort normalization in `format.rs`
  (`try_format_yaml_front_matter`) — plain mappings/sequences/comments only,
  anything else verbatim. Removed at plan Step 7 (front matter handling
  moves up to oxfmt's shared pre-pass; `oxc_formatter_yaml` formats it)
- TokenSeq mini-printer is recursive: top-level commas → fill;
  balanced paren regions → groups; `name(`/`$k: (` glue; math ops break
  after, comparisons stand alone; numbers/strings normalized
- Static-width simulations assume top-level at-rules (column 0); deeply
  nested commented at-rule params would mismeasure (not in the suite).
  `prettier@3.8.1` runs directly via `npx prettier@3.8.1 --parser css` —
  invaluable for verifying layout hypotheses against small repros

## Verification

```sh
cargo check -p oxc_formatter_css
cargo run -p oxc_prettier_conformance                          # pass rates
cargo run -p oxc_prettier_conformance -- --filter css/atrule   # diff a fixture
cargo run -p oxc_formatter_css --example css_formatter file.css
cargo run -p oxc_formatter_css --example parse_debug -- --syntax scss file.scss  # dump raffia AST
```
