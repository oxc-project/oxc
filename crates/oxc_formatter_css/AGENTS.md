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
  - The fork also fixes valid-syntax coverage gaps found in the skipped-fixture
    survey (rev `711e372`): dash-prefixed ID selectors (`#-a-b-c-`),
    Sass interpolation in media-in-parens (`@media x and #{...}` → `MediaInParensKind::SassInterpolation`),
    `@import url()` with SassScript (`ImportPreludeHref::Function`),
    interpolated strings in progid / custom-property values, progid on any
    property name, and nameless vendor `@keyframes {}`
  - The fork also makes Less `+`/`-` binary operators WHITESPACE-SENSITIVE
    (matching lessc): a `+`/`-` is a `LessBinaryOperation` operator only when
    followed by whitespace; `@a -@b` is two values (`-@b` is a
    `LessNegativeValue` sign — `margin: -@a -@b` = two values, NOT `(-@a) - @b`
    subtraction), `@a - @b` is subtraction. Without this the AST miss-attaches
    shorthand signs and reprinting it would corrupt `margin`/`padding`
    (prettier/prettier#6082, #10399). This is what makes the value-position
    `LessBinaryOperation` structured printer (`write_less_binary_operation`)
    safe — every remaining operator is a real whitespace-delimited one
  - Comments are NOT in the AST: collected via `ParserBuilder::comments()` into
    a positional cursor (`src/comments.rs`, mirrors `oxc_formatter_graphql`)
  - The canonical reference is Prettier's `src/language-css/printer-postcss.js`
    (in `tasks/prettier_conformance/prettier`, v3.8.x) — port its layout
    decisions, do not invent new ones
- Two entry points: `format()` (standalone) and `format_to_ir()` (embedded/dispatcher)
- Line endings: `parse_stylesheet` normalizes `\r\n` / lone `\r` to `\n` BEFORE
  parsing (like Prettier's endOfLine pre-pass) — verbatim slices reach the core
  `text()` builder, which panics on raw `\r`. Parse and print both use the
  normalized arena copy, so spans stay consistent. Output is always LF

### Tailwind `@apply` sorting (`CssFormatOptions::sort_tailwindcss`)

Ports prettier-plugin-tailwindcss's `transformCss`: the ONLY CSS construct it
sorts is `@apply` params (`name == "apply"`, case-sensitive). This crate only
COLLECTS — `write_apply_prelude` (at_rule.rs) splits off the `!important` tail
(`/\s+(?:!important|#{(['"]*)!important\1})\s*$/`, see `split_important_tail`)
and Less `~"..."` wrappers, then emits the class list as one
`FormatElement::TailwindClass(index)`. Sorting (order, dedup, whitespace
collapse, `{{` skip) is the host-supplied sorter's job: `format()` takes an
`Option<TailwindSorter>` and bakes the result into the `Document`;
`format_to_ir()` returns an `EmbeddedIr` and its classes travel to the
parent in `DispatchResult::tailwind_classes`, where the parent merges them
with `DispatchResult::remap_tailwind_into` (a dangling index trips a printer
debug_assert). Params containing comments fall back to the normal printers
(sorting would corrupt them).

### Error semantics

`format()` / `format_to_ir()` return `Err` on ANY parse error, including
raffia's recoverable ones (`parser.recoverable_errors()`).
Except `TopLevelDeclaration`, which postcss accepts (the dominant css-in-js shape).

NOTHING falls back to Prettier: standalone files report the
`Err` as a diagnostic, and a css-in-js dispatch `Err` makes the parent print
the template as-is.
Since the raffia fork, value/selector-position `${}` placeholders parse on the Rust path,
so what still `Err`s is garbage Prettier's embed throws on too (e.g. `foo\n${a}\n${b}` bare words).

### Intentionally unsupported: postcss plugin syntax

Syntax that only "works" in Prettier because postcss parses without validation
and a build-time plugin interprets it later is NOT supported in `.css` files —
the parse-error diagnostic is the correct behavior, not a coverage gap:

- postcss-simple-vars: `$blue: #056ef0;` declarations, value-position `$blue`,
  `$(dir)` interpolation (the plugin's entire core)
- postcss-mixins: parametered `@define-mixin x $a, $b` and `$var` inside the
  body. Plain `@define-mixin icon {}` / `@mixin icon;` / `@mixin icon a, b;`
  DO parse fine
- postcss-nested-props (`font: { ... }`), ICSS nested `:export { nest: {} }`,
  `--element(...)` (CSS Extensions, zero implementations)

Verified UNAFFECTED (parse clean, recoverable errors empty — 2026-06-12):
every Tailwind v3/v4 at-rule (`@tailwind` `@apply` `@layer` `@theme` `@utility`
`@variant` `@config` `@custom-media`), every CSS Modules construct (`@value`
incl. `from`, `:global`/`:local`, `composes`, plain `:import`/`:export`), and
standard CSS nesting. The big postcss-ecosystem user bases are all safe;
the unsupported plugins are ~1% of Prettier's npm weekly downloads and
fail LOUD (diagnostic), with ignore-listing as the escape hatch.
If demand materializes, a raffia-fork tolerate option (same pattern as
`tolerate_at_keyword_placeholders`) could accept `$var` tokens.

Value-position `@{var}` (Less) also stays unsupported: lessc itself rejects
it (verified) — raffia's "interpolation is disallowed in declaration values"
is lessc-accurate, and the fixture exercising it (postcss-less PR #159) is a
tolerance test, not valid Less. Same for whitespace inside a Less lookup
(`@config   [   option1]` — gluing works, spacing is near-zero-use).

NOTE: the genuine coverage gaps found in the same survey were FIXED in the
raffia fork (2026-06-12, rev `711e372`; the six fixes are listed under
Overview above). Remaining skipped conformance fixtures are all
intentional-invalid / mixed-language / plugin-syntax, plus `---lang` front
matter (plan Step 6-2, oxfmt pre-pass) and `>>>` (reassess at plan Step 8-1,
Vue scoped styles). Upstreaming the fork fixes to g-plane/raffia is planned
(see the plan's cross-cutting items); minimal repros exist for all of them.

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
  lowercase, `composes` removeLines, `progid:` verbatim, url() inner verbatim.
  Interpolation rules (2026-06-12, from the un-skipped fixtures):
  `SassInterpolatedIdent` reprints structurally (`#{ $a + $b }` →
  `#{$a + $b}` — postcss-values tokenizes through `#{}`); a neighbor glued
  to an interpolated ident stays glued (`1#{$var}` is one word); a unary
  `+`/`-` glues to its operand even across a gap EXCEPT before a function
  call; in `write_sass_binary`, `*` always spaces and word-like `+`/`-`
  operands space on both sides (each ends the glued run) — EXCEPT an
  asymmetric `+`/`-` (whitespace BEFORE, glued AFTER), which is a signed
  operand in postcss-values lexing and stays glued (`$a -$b` → `$a -$b`, NOT
  `$a - $b`; matters for the ambiguous Sass `margin: -$a -$b` list/subtraction
  case dart-sass deprecates). Fixture: `scss/binary-operation-spacing.scss`
- `src/print/selector.rs` — selectors; combinators carry the break point
  BEFORE themselves; `maybeToLowerCase` for pseudos; attribute values are
  quoted via `printString`
- `src/print/at_rule.rs` — prelude dispatch; media query port;
  **unknown at-rule params print VERBATIM** (2026-06-12, ecosystem-CI):
  Prettier's parser hands params to sub-parsers only for a fixed allowlist
  (parser-postcss.js; see `is_value_parsed_at_rule`) and everything else —
  `@apply`, `@tailwind`, `@custom-variant`, `@variant`, `@source`, ICSS
  `@value`, plus `@warn`/`@error` (media-unknown) — stays a plain string the
  printer emits raw (`write_verbatim_at_rule_tail`, also used by
  `UnknownSassAtRule`; slice runs from the NAME so gap comments stay
  embedded; a trailing `//` line pushes `{` down à la
  `lastLineHasInlineComment`). Re-spacing those tokens CORRUPTS Tailwind
  syntax (`dark:bg-x` → `dark: bg-x`, `py-1.5` → `py-10.5`,
  `@custom-variant dark (&:is(...))` → `dark(&: is(...))`).
  The TokenSeq mini-printer (gap-based separators, break AFTER math
  operators) now only serves SCSS-family names parsed AS CSS
  (`@include` etc., raffia: Unknown / Prettier: parseValue);
  "fused" preludes (`@page:first` stays tight when the source has no gap);
  SCSS control directives wrap `[space, prelude, line]` in a group so `{`
  drops to its own line when the prelude breaks — EXCEPT fully parenthesized
  conditions (Prettier's `hasParensAroundNode` → `{` stays on the `)` line);
  a no-comment `@import` path list fills (long lists wrap at the width)
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
  preceding value group — EXCEPT a single interpolated component
  (`--p: #{fn(...)}; // c`): Prettier's value parser splits `#{` into
  multiple fill chunks and a fill chunk's fit ignores the rest of the line,
  so the comment never breaks it (2026-06-12, mastodon; routed through a
  single fill entry in `write_comma_group` + an extra `indent` in the
  `SassInterpolated` arm for the +2/+1 broken layout — both derived from
  `--debug-print-doc`)
- The 2026-06-12 ecosystem-CI sweep (fixture
  `format/scss/layout-major-diffs.scss` covers all of it) also fixed:
  grid separator = plain SPACE (single-line grid values NEVER re-wrap,
  Prettier pushes `" "` not `line`); custom property `!important` lives on
  the REPARSED declaration (`reparse_custom_property_value`) — the printer
  reads it from there or it silently vanishes; media feature values are
  flat text (`media-value` = `adjustNumbers(adjustStrings(...))` →
  `ValueContext::no_break`, honored by
  `write_function`/`write_calc`/`write_sass_binary` — e.g. a media feature
  value `map-get(...) - 1px` never breaks, however long the `@media` line);
  **calc is a FLAT operator fill** (`write_calc` flattens nested
  unparenthesized `Calc` nodes into chunks — operator glued to its LEFT
  operand, break after, ONE uniform indent; parenthesized sub-expressions
  stay single chunks). The calc flattening also fixed the webawesome
  `page.styles.ts` diff at printWidth 100; the printWidth-80 webawesome
  diffs REMAIN — they are the deliberate rule-B divergence (see the fill
  VERDICT below), not a calc-layout artifact. (A first conformance run
  seemed to clear them all, but `pnpm conformance` imports `dist/index.js`
  WITHOUT rebuilding — it had measured a stale binary. Always run
  `pnpm --dir apps/oxfmt build-dev` first.)

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
    webawesome diffs at printWidth 80, still accepted as of 2026-06-12;
    the flat-calc rewrite resolved only the printWidth-100 instance)
    is IRRATIONAL and we deliberately do NOT follow it. Removing the sims
    was measured (3 fixtures fail: css 112/114, scss 84/85 — all
    comment-torture tests) and rejected: it would drop rational behavior A
    for consistency's sake. The principled long-term fix, if ever needed,
    is to teach rule A ALONE to the core fill fit-check (NOT full Prettier
    fill) — that retires all three sims; it requires a JS-conformance
    impact experiment first since core fill is shared with `oxc_formatter`.
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
  `prettier@3.8.4` runs directly via `npx prettier@3.8.4 --parser css` —
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

Fixtures are grouped per language (`format/{css,scss,less}/`; test modules
mirror the directories, e.g. `fixtures::format::scss::case_normalize`).
`embedded/scss/` is explicit about the dispatcher's variant=Scss hardcoding.
The shared `options.json` sits at `format/` / `embedded/` level (the harness
walks up to the nearest one).

Cases the Prettier conformance suite does NOT cover live here as insta
snapshots (same harness as `oxc_formatter_json`/`_graphql`): placeholder
at-rules, custom-property value re-parsing (incl. the multi-byte-comment
span regression), calc operand parens, An+B normalization, `:has(>)`
relative combinators, `var()` trailing commas, group-breaking trailing
comments, top-level declarations, `@import ... supports(...)` (the prelude
`supports` arm was a data-loss stub emitting empty `supports()`; now routed
through the structured `@supports` printers — `at-rule-import-supports.css`).
Parse-error fallback triggers are unit
tests in `tests/fixtures/mod.rs` (`parse_error_is_err`).

The skipped-fixture survey (2026-06-12) added 11 fixtures extracted from
the valid parts of permanently-skipped conformance files (`@import`/
`@charset`/`@supports` variants, selector variants, ICSS, SCSS control
directives / case / values / CRLF / comma imports, Less case / operations /
lookups) — every block machine-compared against prettier@3.8.4 (the
pre-survey fixtures also match 3.8.4, re-verified after the per-language
reorganization). The survey
exposed 10 divergences; 9 are FIXED (comment-bearing selectors verbatim,
ICSS `@value`/property casing, `URL(` casing, keyframe interpolations,
`shouldBreakList` multi-node chunks incl. non-initial `-`-led idents,
`@import`/`@supports` break points, division spacing per postcss-values
lexing — each rule is documented at its implementation site). Known
divergences, all deliberate (impact does not justify the matching cost):

- Less `func(x, + 20px)` unary gluing → Prettier prints `+20px`; raffia
  ASTs the `, +` as a comma-left binary operation, so matching it would be
  ad-hoc for a torture-test-only shape. Revisit if hit by real code
- Nested Less math in a function arg / multi-value shorthand
  (`max((round(...) / 10) - @bw, 0)`, `margin: (a + b) -@x -@y`): Prettier's
  fill fit-check breaks INSIDE the wide chunk (the leading paren goes to its
  own lines); our core `fill` (biome semantics) breaks the SEPARATOR instead
  (between the paren and the next value). Same root as the webawesome
  `*.styles.ts` divergences — the principled fix is the shared core-fill
  fit-check change (needs a JS-conformance impact experiment first). Standalone
  `@var:`/value-position Less math (incl. a parenthesized op that breaks onto
  its own lines) DOES match Prettier now — see below
- Broken `:not(...)` selector args indent at +2 relative to the selector;
  Prettier lands at +4 (arg) / +2 (`)`) via its selector-printer indent
  stack. Layout-only, rare trigger (selector longer than the line width)
- Selector-position Sass interpolation normalizes inner spaces
  (`#{ $name }` → `#{$name}`, `selector.rs` `write_interpolable_ident` routes
  through `write_sass_interpolated_ident`). Prettier keeps SELECTOR
  interpolation verbatim while normalizing VALUE-position interpolation
  (postcss-selector-parser treats it as an opaque token; same token-stream
  limitation as the `#1811` operator spacing). We normalize BOTH positions so
  oxfmt's output is internally consistent — at the cost of one conformance
  fixture (`scss/map/function-argument/functional-argument.scss`, the only
  `.text #{ $name }` spaced case). String literals inside the interpolation
  re-quote either way (`#{'x'}` → `#{"x"}`), which DOES match Prettier;
  see `tests/fixtures/format/scss/interpolation-quotes.scss`

(The former second divergence — `@foo 'one'` requoting — was RESOLVED by
the unknown-at-rule verbatim contract (2026-06-12): ALL unknown at-rule
params now print raw, so the `raw_at_rule_strings` ICSS special-case cell
is gone too.)

The 2026-06-12 ecosystem-CI sweep added `format/css/tailwind-at-rules.css`
(unknown at-rule verbatim: `@apply`/`@custom-variant`/`@variant`/`@source`),
`format/css/custom-property-important.css` (custom property `!important`
survival), `format/scss/layout-major-diffs.scss` (grid no-rewrap,
`@import` fill, interpolation fit/indent — custom AND normal props —, flat
calc, media-value no-break, `@warn`/`@error` raw) and
`format/scss/unknown-at-rule-edges.scss` (fused `@a:b`, glued `@foo (x)`,
comments embedded in verbatim params, trailing `//` pushing `{` down,
single-quote preservation in `@error`/`@warn`, interpolated-name
`@#{$name}` = the `UnknownSassAtRule` statement path) — all
machine-compared against prettier@3.8.4 at both widths.

The 2026-06-13 Less ecosystem sweep (ng-zorro-antd 409 / vant 107 `.less`
files vs Prettier; vant now diffs ZERO, ng-zorro only the two deliberate
divergences above plus its `insert_final_newline=false` editorconfig)
established the **Less selector-side verbatim contract** (`less.rs`):
Prettier re-parses mixin-definition preludes, statement-position mixin
calls and `when` guards with postcss-selector-parser and prints the raw
text — so spacing/newlines survive, nothing width-breaks, and the ONLY
transforms are `adjustNumbers` + `adjustStrings`
(`value::adjust_numbers_and_strings`). `LessConditionalQualifiedRule` is a
`css-rule`: NO trailing `;` after the block (it used to hit the verbatim
catch-all and gain one). Also fixed: function-argument source parens that
raffia drops are restored by bounded balance-scan
(`group_own_paren_layers` — `max(((a - b) / 2), 0)`, `min(((@a)), @b)`,
`calc((a))`); Less `@var:` values get NO softline after the colon
(`ValueContext::no_leading_softline` — Prettier's
`shouldPrecededBySoftline` matches `css-decl` only, not atrule-variables);
`~'...'` re-quotes like a plain string AND counts as a multi-node
comma_group for `shouldBreakList`. Fixtures: `less/guards.less`,
`less/mixin-verbatim.less`, `less/variable-values.less`,
`css/calc-parens.css` — all machine-compared against prettier@3.8.4 at
both widths.

The 2026-06-18 oxfmt-conformance sweep (apps/oxfmt ng-zorro 409 `.less`)
fixed three quote-normalization gaps where a Less string variant fell into a
verbatim catch-all and skipped `printString` re-quoting (Prettier
`adjustStrings` always normalizes the quote): (1) `@import (options) '...'`
parses as `AtRulePrelude::LessImport` — it had no prelude arm and hit the
`_ => verbatim` catch-all; now `write_less_import_prelude` prints
`(names)` + href via the shared `write_import_href`; (2) an interpolated
import path (`@import './@{var}.less'`) was verbatim in
`write_import_prelude_inner`; now both plain and Less imports route the href
through `write_import_href`, which `write_requoted_verbatim`s the
interpolated case; (3) attribute-selector `[class^=~'...']`
(`AttributeSelectorValue::LessEscapedStr`) was verbatim — now `~` +
`write_str` like the value-position handler. Fixture:
`less/import-quotes.less`.

The same sweep replaced the verbatim `LessBinaryOperation` printer with a
structured one (`write_less_binary_operation` + `write_less_parenthesized_operation`
in `value.rs`): a flat operator fill (break AFTER the operator, one uniform
indent; nested unparenthesized ops flatten into the same fill; a parenthesized
sub-expression is its own group that drops `(`/`)` onto their own lines when it
breaks). This is only safe because of the raffia whitespace-sensitive `+`/`-`
fix above (signs never reach this printer as operands). It matches Prettier for
value-position math — `line-height: @a - 2*@b - (@c/2)` (was overflowing
printWidth), `@c-y: (long + expr)` paren-on-own-lines — and is output-neutral
when the expression fits. Fixture: `less/math-operations.less`. The remaining
ng-zorro divergences are: `:not()` wrap indent (above), and nested math in
function-arg / multi-value-shorthand contexts (the core-fill fit-check
divergence — see the Known-divergences list).

Fixtures under `embedded/` route through `format_to_ir` (the css-in-js
dispatcher entry, placeholders tolerated) instead of `format()`: value /
selector / paren-softline / ignore-vanish placeholder cases. The
`embedded_debug` example formats a file the same way for quick comparison
(`cargo run -p oxc_formatter_css --example embedded_debug file.scss`).

**Every expected output was verified against Prettier (3.8.4, the current submodule);
do the same when adding fixtures** (`npx prettier@3.8.4 --parser <variant>`,
at both `--print-width 80` and `100` — the harness snapshots both).
Update snapshots with `cargo insta review` (or `INSTA_UPDATE=always cargo test`).
