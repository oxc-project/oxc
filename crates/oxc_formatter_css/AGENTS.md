# Coding agent guides for `crates/oxc_formatter_css`

## Overview

Prettier compatible CSS/SCSS/Less formatter, using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
- Parses with [raffia](https://docs.rs/raffia) 0.12 (AST with full spans + `raw` text; NOT a CST)
  - A FORK (git dep on `leaysgur/raffia` in workspace deps): adds
    `ParserOptions::tolerate_at_keyword_placeholders`, which accepts at-keywords
    in declaration values (`ComponentValue::TokenWithSpan`) and selectors
    (type/class selector idents whose `raw` INCLUDES the leading `@`).
    Only `format_to_ir` enables it; `format()` stays strict
  - Comments are NOT in the AST: collected via `ParserBuilder::comments()` into
    a positional cursor (`src/comments.rs`, mirrors `oxc_formatter_graphql`)
  - The canonical reference is Prettier's `src/language-css/printer-postcss.js`
    (in `tasks/prettier_conformance/prettier`, v3.8.x) — port its layout
    decisions, do not invent new ones
- Two entry points: `format()` (standalone) and `format_to_ir()` (embedded/dispatcher)

### Error semantics

`format()` / `format_to_ir()` return `Err` on ANY parse error, including
raffia's recoverable ones (`parser.recoverable_errors()`) — except
`TopLevelDeclaration`, which postcss accepts (the dominant css-in-js shape).
What oxfmt does with the `Err` differs by entry point: standalone files
report it as a diagnostic (NO Prettier fallback), while the css-in-js
dispatcher falls back to Prettier. Since the raffia fork (plan Step 6),
value/selector-position `${}` placeholders parse on the Rust path, so that
fallback is a pure safety net: what still `Err`s is garbage Prettier can't
format either (e.g. `foo\n${a}\n${b}` bare words — Prettier's embed throws
too and the template prints as-is).

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

Wired into oxfmt: standalone css/scss/less files AND the
css-in-js dispatcher route (`apps/oxfmt`). The embedded suite
(`pnpm conformance` in `apps/oxfmt`) also exercises this crate via
`format_to_ir`; re-run it for printer changes too.

Keep them that way: any change here must re-run the conformance suite.

### css-in-js specifics

- `format_to_ir` input contains `@prettier-placeholder-N-id` markers for
  `${}` interpolations. raffia parses statement-position markers as
  at-rules (a `;`-less one SWALLOWS the following statements into its
  prelude); value/selector-position markers parse via the fork option
  (see Overview) — `format_to_ir` passes `tolerate_placeholders: true`
- Value-position placeholders are `ComponentValue::TokenWithSpan` and mostly
  ride the existing gap-based separator rules (glued → `Tight`, spaced →
  `Line`). ONE added rule: placeholder glued to a paren group separates with
  `Separator::SoftBreak` (Prettier's `isAtWordPlaceholderNode +
isParenGroupNode → softline`: `${fn}(30px)` breaks BEFORE the parens)
- Selector-position placeholders trigger "garbage mode"
  (`write_selector_list`): postcss-selector-parser degrades on at-words, so
  from the selector containing the FIRST placeholder onwards Prettier prints
  near-verbatim — our port emits the raw source slice with whitespace runs
  collapsed to single spaces, never breaking. Commas BEFORE the first
  placeholder still split selectors normally. Detection is a source-text
  scan for `@prettier-placeholder-`
- Ignored (`prettier-ignore`) `;`-less placeholder at-rule at EOF VANISHES
  (`write_statement_sequence`): postcss leaves no `source.end` on it, so
  Prettier's `printIgnored` slices an empty string. Reproduced for
  placeholder at-rules ONLY — the resulting placeholder-count mismatch makes
  the embed fall back to plain template printing, like Prettier. For real
  code (`@foobar`) we deliberately DIVERGE and keep the verbatim text
  (Prettier silently deletes it; that's a data-loss bug, not a behavior to
  port)
- Placeholder at-rules get Prettier's `isTemplatePlaceholderNode`
  treatment (`write_placeholder_at_rule` in `at_rule.rs`): verbatim
  prelude (newlines stay literal), gap comments printed not discarded,
  name-glued `:` collapses following whitespace to one space, `;` only
  when the source has one
- `TopLevelDeclaration` is the ONE recoverable error format() accepts
  (postcss accepts it; it is the dominant css-in-js shape)
- Custom property values arrive as raw token streams; they are re-parsed
  as a plain declaration at the same source offsets (prefix blanked
  BYTE-wise — multi-byte chars! — prop replaced by `a`s) so the value
  gets the normal group/break layout (`reparse_custom_property_value`)
- raffia's `Calc` spans EXCLUDE operand parens; `write_calc_operand`
  recovers them from the source (children account for unbalanced parens
  inside the span — see the `need_left`/`need_right` math)
- A source trailing comma in function args survives for `var()` ONLY
  (Prettier's `printTrailingComma`)
- Trailing same-line comments are plain content, NOT `line_suffix`:
  Prettier counts them towards the line width, so they can break the
  preceding value group
- Known remaining diffs (webawesome suite, layout-only): Prettier's fill
  fit-check (`[item, sep, next]`) breaks inside `var(...)` args in long
  `calc()`s and inside `::slotted()` after a long `:not(...)`, where our
  fill breaks after the operator / inside `:not(...)` instead

Layout machinery notes discovered en route:

- Our core `fill` breaks the separator AFTER a hard-broken entry (biome
  semantics) and measures fits only up to a hardline; Prettier's fill
  fit-checks `[item, sep, next-item]` and treats a hardline-bearing chunk
  as never fitting. Where that diverges, separator breaks are SIMULATED
  with static source widths (`write_commented_value_params` /
  `write_commented_media_params` in `at_rule.rs`, the SassImport path,
  the lead-comment fill in `write_value_groups`)
  - VERDICT on Prettier's fill semantics (assessed 2026-06-11, keep the
    sims as-is): it is really TWO separate rules with different merit.
    (A) "a hardline-bearing chunk never fits → it starts on its own line"
    is RATIONAL — it keeps a comment visually attached to the item it
    annotates (`// Comment` ends up on its own line BEFORE the next
    import path, not dangling after the previous one). The sims reproduce
    rule A, so they are reasoned behavior, not quirk-for-bytes emulation.
    (B) the pairwise lookahead's side effect of breaking INSIDE short
    paren groups (`var(\n --x\n ) * 2`, `::slotted(\n *\n )` — the two
    accepted webawesome diffs) is IRRATIONAL and we deliberately do NOT
    follow it. Removing the sims was measured (3 fixtures fail: css
    112/114, scss 84/85 — all comment-torture tests) and rejected: it
    would drop rational behavior A for consistency's sake. The principled
    long-term fix, if ever needed, is to teach rule A ALONE to the core
    fill fit-check (NOT full Prettier fill) — that retires all three sims
    and keeps the webawesome layout; it requires a JS-conformance impact
    experiment first since core fill is shared with `oxc_formatter`.
    Note the sims sit INSIDE the source-rebuild layer for comment-bearing
    params; the rebuild itself can never be removed (raffia's AST drops
    params-embedded comments — removing it loses comments, not layout)
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
cargo test -p oxc_formatter_css                                # fixture snapshots (see below)
cargo run -p oxc_prettier_conformance                          # pass rates
cargo run -p oxc_prettier_conformance -- --filter css/atrule   # diff a fixture
cargo run -p oxc_formatter_css --example css_formatter file.css
cargo run -p oxc_formatter_css --example parse_debug -- --syntax scss file.scss  # dump raffia AST
```

### Fixture tests (`tests/fixtures/format/` and `tests/fixtures/embedded/`)

Cases the Prettier conformance suite does NOT cover live here as insta
snapshots (same harness as `oxc_formatter_json`/`_graphql`): placeholder
at-rules, custom-property value re-parsing (incl. the multi-byte-comment
span regression), calc operand parens, An+B normalization, `:has(>)`
relative combinators, `var()` trailing commas, group-breaking trailing
comments, top-level declarations. Parse-error fallback triggers are unit
tests in `tests/fixtures/mod.rs` (`parse_error_is_err`).

Fixtures under `embedded/` route through `format_to_ir` (the css-in-js
dispatcher entry, placeholders tolerated) instead of `format()`: value /
selector / paren-softline / ignore-vanish placeholder cases. The
`embedded_debug` example formats a file the same way for quick comparison
(`cargo run -p oxc_formatter_css --example embedded_debug file.scss`).

**Every expected output was verified against `prettier@3.8.3` by hand;
do the same when adding fixtures** (`npx prettier@3.8.3 --parser <variant>`,
at both `--print-width 80` and `100` — the harness snapshots both).
Update snapshots with `cargo insta review` (or `INSTA_UPDATE=always cargo test`).
