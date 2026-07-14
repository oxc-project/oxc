# Coding agent guides for `crates/oxc_formatter_css`

## Overview

Prettier compatible CSS/SCSS/Less formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- Two entry points:
  - `format()`: standalone files (returns a printable `Formatted`)
  - `format_to_ir()`: embedded use via the dispatcher (e.g. css-in-js);
    tolerates `${}` placeholders and `TopLevelDeclaration`
- The canonical reference is Prettier's `src/language-css/printer-postcss.js`
  - port its layout decisions, do not invent new ones
  - ONE exception: never reproduce output that changes program semantics
    - See "Known divergences" section

### Forked parser

Parses with [`oxc-css-parser`](https://crates.io/crates/oxc-css-parser), a `raffia` fork pinned via the workspace `Cargo.toml`.

The fork adds:

- `template_placeholder` option (for the css-in-js dispatcher)
  - Backtick-delimited marker `` `<prefix><digits>` `` with a parameterized inner affix (`TemplatePlaceholder { prefix }`);
  - Backtick is invalid CSS/SCSS/Sass (only Less's inline-JS delimiter), so the marker is
    unmistakably out-of-band, not a real `@var`/`$var` or at-rule
  - Only `format_to_ir` enables it (with the option unset a backtick is a syntax error)
  - MUST be used with `Syntax::Scss`; css-in-js is `CssVariant::Scss`-hardcoded
  - Tokenized as one typed `Token::Placeholder { index, suffix }` accepted in value / selector / statement / declaration-name positions
    - Per-position layout and coverage: see "css-in-js specifics" below
- Whitespace-sensitive Less `+`/`-` operators (matching `lessc`)
  - A `+`/`-` is a `LessBinaryOperation` operator only when followed by whitespace;
  - `@a -@b` is two values (`-@b` is a `LessNegativeValue` sign
  - `margin: -@a -@b` = two values, NOT `(-@a) - @b` subtraction), `@a - @b` is subtraction
- Various bug fixes for valid CSS/SCSS/Less syntax `oxc-css-parser` miss-parses or rejects
  - selector / at-rule / value-token coverage gaps
- Additive leniencies for syntax reference compilers reject but postcss (and so Prettier) accepts
  - the IE `*color` hack, Scss dotted words (`foo.bar`, xstyled / tailwind-theme tokens), plain-CSS `%placeholder` selectors (postcss-extend-rule), non-standard `@import` tails, ...
  - Each is additive: it only accepts previously-erroring input, never changes the AST of input that already parsed
  - See "Policy: how to take in non-spec / non-Sass dialect syntax" below

Prettier operates on `postcss` + three sub-parsers (`postcss-selector-parser`, `postcss-values-parser`, `postcss-media-query-parser`) and depends on `raws` (source gaps).

`oxc-css-parser` parses everything structurally in one pass; source gaps are recovered by comparing span boundaries (`hasEmptyRawBefore(x)` == "no gap between spans").

### Error semantics

`format()` / `format_to_ir()` return `Err` whenever they cannot produce output they can stand behind:

- `oxc-css-parser` is error-tolerant via `parser.recoverable_errors()`, but any parse error bails out;
  - Never format a broken AST
  - Exception: `TopLevelDeclaration`, tolerated ONLY by `format_to_ir()`
    - The dominant css-in-js shape, `` css`display: flex;` ``)
    - Standalone `format()` still rejects it as invalid CSS/SCSS/Less (Dart Sass rejects it too)
- print-stage internal errors are also `Err`
- The caller (oxfmt) decides what happens next
  (diagnostics for standalone files, template-as-is for embedded)

### Comments

`oxc-css-parser` does not attach comments to the AST;
they are collected via `ParserBuilder::comments()` into a positional cursor over `CssComment { span, inline }` (`inline` = `//`).

- Statement-level comments: flushed before each statement
  (`flush_leading_comments`); consecutive same-line comments stay glued
  (`*/ /*!`), but a comment is always followed by a line break before a node
- Value-level comments: flushed inside fill entries before the component they
  precede (`flush_value_comments`); `//` comments expand the parent group and force a hardline after
- Trailing (`value /* c */;`): flushed by `write_declaration` with the source gap before `;` preserved
- After each statement, the sequence DISCARDS unclaimed comments inside the
  statement span (cursor must never point before a printed position)

#### Placement invariants

The placement rules follow `crates/oxc_formatter/AGENTS.md` "Comment placement invariants"
(the invariant / compat-table two-layer split, and the terminator vs separator ownership rule);

this section records their CSS translation:

- A comment never crosses user content (other values, other comments):
  - it stays on its source side of every value/argument
- `,` is a list SEPARATOR: a comment between an element and its comma stays BEFORE the comma
  - `a /* c */, b`; comments after it lead the next element
  - Declaration value lists (`write_value_groups`) and function arguments (`write_function`) route every comma through `write_group_comma` with the comma offset paired to its group
  - `split_comma_groups` returns `(group, Option<comma_start>)`; SCSS/Less lists pair `comma_spans`
  - A new comma site must take the pair, the shape makes taking the groups without the commas a visible choice, not an accident
  - Adopted by every comma writer (function/include args, maps, paren/sass lists, `@mixin` params, `@each` bindings, keyframe selectors)
- `;` is a declaration TERMINATOR, but unlike JS statements Prettier does NOT move comments behind it:
  - `value /* c */;` keeps the comment before `;` (measured behavior, not principle, may change in the future)
- The positional cursor makes ownership a bounds discipline, not an attachment one:
  - a flush's upper bound must never extend past the next piece of user content,
  - and a declaration's `tail_bound` may only be consumed by the LAST comma group (`write_value_groups` clears it for every other group)
- Never let a comment cross a line boundary
  - `//` comments force a hardline after; own-line comments stay own-line, and never move a suppression comment off its target

### Line endings

`parse_stylesheet` normalizes `\r\n` / lone `\r` to `\n` BEFORE parsing.

Unlike other formatters that normalize locally where needed, CSS has too many verbatim slices to handle case by case.
And without this, raw `\r` reaching the core `text()` builder would panic.
Parse and print both use the normalized arena copy, so spans stay consistent.

The configured `end_of_line` option still applies, the printer emits the chosen line ending when materializing multiline `Text` IR.

### css-in-js specifics

`format_to_ir()` accepts SCSS-like source with `` `PLACEHOLDER-N` `` markers in place of `${}` interpolations.
`oxc-css-parser` tokenizes each as a typed `Token::Placeholder` via the fork option `template_placeholder` (`format.rs` passes the inner affix). Backtick is invalid SCSS, so the marker can never be confused with real syntax (see the `TEMPLATE_PLACEHOLDER_PREFIX` / `_SUFFIX` consts in `lib.rs`).

Each parses into a typed node that the printer emits as a `FormatElement::EmbedPlaceholder(N)` marker (`print/mod.rs::write_placeholder`), plus a `Text` for any glued suffix; the JS host (`oxc_formatter/embed/css.rs`) substitutes `${exprN}` back. No output-side string protocol — the index is carried structurally.

Per-position layout, the non-obvious rules below;
the exact set of supported positions (incl. id / attribute-value / class selector) is whatever the `embedded/scss/*-placeholders.scss` fixtures exercise, not this list:

- Statement position: a `Statement::Placeholder` (`write_statement`) source-driven layout:
  - The `;` is kept only when the source has one; consecutive placeholders preserve the source whitespace (`${a} ${b}` on one line, `${a}\n${b}` on two)
  - A `;`-less placeholder opens a postcss "swallow" run: following declarations keep a source-driven `;` until a source `;` ends the run (`write_statement_sequence_bounded`)
  - `${foo}: ${bar}` parses as a declaration whose property NAME is a placeholder
- Value position: `ComponentValue::Placeholder`; rides existing gap-based separator rules
  - One added rule: glued to a paren group → `Separator::SoftBreak` (`${fn}(30px)` breaks BEFORE the parens)
- Selector position: `InterpolableIdent::Placeholder`; a placeholder mid-selector still triggers "garbage mode" in `write_selector_list`
  - Emits the raw source slice with whitespace runs collapsed (sentinels split back out to `EmbedPlaceholder` via `write_text_with_placeholders`), never breaking
  - Mirrors `postcss-selector-parser` degrading on at-words
  - A statement-position placeholder-led selector must not absorb across a newline (oxc-css-parser `placeholder_starts_qualified_rule`): `${mixin}\n& > .x {}` is two statements
- String / `url()` position: the CSS lexer keeps these opaque, so a sentinel inside them stays in a verbatim `Text` (no `EmbedPlaceholder`)
  - The JS host (`oxc_formatter`) counts these and substitutes them inline through its `Text`-sentinel branch, a deliberate string-scan fallback at the edges of the typed path

`tests/fixtures/embedded/scss/*-placeholders.scss` is the source of truth for which positions parse and how they print (the `embedded/` harness runs `format_to_ir` with the option on); add a fixture there when extending coverage.

## Prettier mapping

### Unknown at-rule params print VERBATIM

Prettier's parser hands params to sub-parsers only for a fixed allowlist (see `parser-postcss.js`, `is_value_parsed_at_rule`);
everything else (`@apply`, `@tailwind`, `@custom-variant`, `@variant`, `@source`, ICSS `@value`, etc) stays a plain string the printer emits raw (`write_verbatim_at_rule_tail`).

Re-spacing those tokens CORRUPTS Tailwind syntax: `dark:bg-x` → `dark: bg-x`, `py-1.5` → `py-10.5`, `@custom-variant dark (&:is(...))` → `dark(&: is(...))`.

We also follow this to keep Prettier compatibility.

### Tailwind `@apply` sorting (`CssFormatOptions::sort_tailwindcss`)

Ports prettier-plugin-tailwindcss's `transformCss`: with the option on, `@apply` params become `FormatElement::TailwindClass(index)` elements, and a host-supplied `TailwindSorter` performs the actual ordering/dedup outside this crate.

See `write_apply_prelude` in `at_rule.rs` (collection + `!important` / Less `~"..."` extraction) and `format.rs` (sorter dispatch).

### postcss plugin syntax

`postcss` parses everything permissively and lets plugins interpret syntax at runtime; `oxc-css-parser` parses strictly, so plugin-specific syntax is rejected by default. Failures emit a LOUD diagnostic.

However, some plugin-flavored constructs work anyway, because:

- 1: Now standard CSS
  - CSS nesting
  - Tailwind v3/v4 at-rules (`@tailwind`, `@apply`, `@layer`, `@theme`, `@utility`, `@variant`, `@config`, `@custom-media`)
  - CSS Modules (`@value` incl. `from`, `:global`/`:local`, `composes`, plain `:import`/`:export`)
- 2: CSS forward-compat
  - Unknown at-rules (`postcss-mixins`'s `@define-mixin`, `@mixin`, etc.) round-trip as `UnknownAtRule` with the prelude held as a verbatim `TokenSeq`
  - `@media`/`@supports` preludes `oxc-css-parser` can't structure fall through to `<general-enclosed>` as a verbatim `TokenSeq`

Beyond those, we add support per-plugin when there's real demand.

#### Policy: how to take in non-spec / non-Sass dialect syntax

Plugin dialects (`xstyled` dotted tokens, Tailwind `theme()` paths, postcss plugin at-rules, ...) look like an unbounded support surface.
They are not — the oracle is never "the dialect", it is what Prettier does with the bytes, and Prettier's answer is almost always "preserve verbatim".
`postcss` is a token-soup preserver, not a grammar: everything it doesn't positively recognize is a "word" that round-trips untouched.
So the target behavior is finite: never destroy tokens Prettier wouldn't destroy.

When a dialect report comes in, first translate it: "which GENERAL postcss behavior are we missing?" Not "how do we support plugin Xxx?".
Then absorb it at the highest possible rung of the escape-hatch hierarchy (top = cheapest, each rung covers whole classes of dialects at once):

1. Unknown at-rule prelude verbatim (`write_verbatim_at_rule_tail`)
   zero-cost bucket: Tailwind, postcss-mixins, ICSS ride it for free
2. Raw fallbacks when the typed grammar rejects (raw component values, `TokenSeq`, `ImportPrelude.modifiers`)
   `[attr=;]`, weird import tails
3. postcss word rules at the separator layer (`is_word_glued_number`, the `1#{$var}` glue, solidus words)
   variant-agnostic, fixes xstyled + `theme()` + future unknown tokens in one place
4. `ParserOptions` flag + typed node (postcss-simple-vars)
   ONLY when the formatter must make layout decisions INSIDE the construct.
   Promotion criteria, all three:
   (a) real user demand, (b) Prettier itself formats it structurally (not verbatim), (c) rungs 1-3 can't express it
5. A dedicated `CssVariant`
   only for real languages with reference compilers (css/scss/less).
   Never for a plugin.

Parser-side leniencies (in the `oxc-css-parser` fork) must be additive:
accept only input that previously errored, and never change the AST of input that already parsed
(e.g. dotted words try the typed `foo.$var` / `foo.bar(...)` parse first; only dart-sass-invalid shapes take the lenient path).
Every lenient path carries a comment citing the reference-compiler vs postcss behavior, a test pinning the strict shapes, and shows up as a visible expected-error flip in the parser's conformance snapshots.

Triage order for reports (failure modes are asymmetric, a parse error is a SAFE failure, oxfmt leaves the file/template as-is; silent token corruption like `sandstone.10` → `sandstone 0.1` is the UNSAFE one):
don't corrupt (verbatim paths) → then accept (leniency) → then pretty-print (structure).

Red flags that the approach is drifting:
specific plugin names accumulating in code, or a leniency that reinterprets previously-valid input.
Either means the fix is at the wrong rung.

#### Supported: postcss-simple-vars (auto-enabled for `CssVariant::Css`)

Covered:

- `$var: value !important;` declarations (top-level and inside rules)
- `$var` references in property values
- `$var` references inside `@media`/at-rule preludes

NOT covered: `$(var)` interpolation (`margin-$(dir): 10px`, `.icon.is-$(network)`), selector-position bare `$var` (`.$prefix`), comment substitutions (`<<$(var)>>`).

### Known divergences

Deliberate divergences from Prettier. Two admission reasons:

1. Prettier's output would change program semantics (formatting must never do that)
2. The impact does not justify the matching cost

Notable divergences are:

- A COMMENTED keyframe selector list is formatted structurally (one selector per line, comments per the separator rule: `60% /* mid */,`)
  - Prettier keeps the whole list verbatim on one line, interior spacing included (`60%   /* mid */  ,   70%` survives untouched)
  - Ours prints commented and uncommented lists with the same layout; layout-only, rare trigger
- Broken `:not(...)` selector args indent at +2
  - Prettier lands at +4 (arg) / +2 (`)`)
  - Layout-only, rare trigger (selector longer than line width)
- `@nest <selector-list>` continuation lines indent at +2 (same class as the `:not(...)` entry above)
  - Prettier lands at +4 (comma-separated selectors) / +6 (wrapped selector parts)
    - An artifact of its generic at-rule params indent
  - Ours matches how selector lists indent everywhere else
    - This is layout-only, deprecated syntax, triggers only on width overflow
- SCSS: `@forward` with `show`/`hide` members AND a `with (...)` config
  - Prettier parses the whole prelude as ONE comma list, so the config's forced break spills into the member commas
    (`show b,\n  c with (` even when the head fits) and the config body lands one level deeper (+4 body / +2 `)`)
  - We break members only on width overflow (fill, matching Prettier's break positions when no config is present)
    and keep the config at the standalone `with (...)` indent (+2 body / 0 `)`, same as `@use`)
  - Layout-only, rare combo
- SCSS: `@forward` members after an over-wide FIRST member pack at +2
  - The prelude head (`path`, `as <ns>`/`as <prefix>-*`, `show`/`hide`, members) is one flat fill,
    so overflow breaks at the token seams with a +2 continuation
    - Matching Prettier's break points (its params are a comma list of `line`-joined words in a fill), incl. the trailing-`;` exclusion from the last chunk's fit
  - The one difference: when the FIRST member alone overflows, Prettier indents it at +4 and puts
    every later member on its own line at +2 (artifacts of its nested comma-chunk fill);
    our flat fill packs the continuation members at +2 (`show\n  <wide-member>, second;`)
  - Same class as the `:not(...)` indent entry; pinned in `module-head-seams.scss`
  - Remaining printers of this overflow class (heads that still never break): `@for` bounds
    and `@namespace` — Prettier value-parses both, so it breaks their word seams too;
    extend the same fill shape if reported
- `<general-enclosed>` media preludes (`@media (not all)`, `(screen and (color))` unparsable as `<media-condition>`) normalize whitespace fully
  - Source gap → one space, paren inner edges tight
  - Prettier only collapses space RUNS inside the unparsable paren, leaving `(not ( screen and ( color ) ))`
    - We print `(not (screen and (color)))`
  - Reproducing the half-normalization is pure tokenizer-artifact matching;
    - Gap-based spacing never fuses tokens the source kept apart (`and (` can't become a function token `and(`)
- A source-glued value-position `[...]` stays glued to ANY typed left neighbor and prints verbatim
  - `theme(fontSize.af-md[0])`, `foo[0.50]`: matching Prettier, which lexes the run as ONE postcss word;
    But also `var(--x)[0]`, where Prettier prints `var(--x) [0]`)
    - Prettier's space there is a word-lexing artifact (`[` extends a word, but not across `)`)
    - Ours is one gap-based rule for all variants: never add a space the source doesn't have
  - Less lookups (`@config[@key]`) are unaffected: the typed lookup rule wins and keeps printing structurally
  - With the name GLUED to the `(` (`--viewport-medium(width<=50rem)`)
  - Prettier keeps the whole prelude verbatim (ONE `media-type` token)
- A declaration swallowed by a `;`-less css-in-js placeholder (`${m}\ncolor: red`)
  - We parse it structurally and FORMAT it (spacing/hex/number normalization)
  - Prettier keeps it verbatim, postcss swallows the run as an opaque prelude string it can't format, so `color   :   red` / `#FFFFFF` survive unformatted
- SCSS: Selector-position Sass interpolation normalizes inner spaces (`#{ $name }` → `#{$name}`)
  - We normalize BOTH positions for output consistency
  - Prettier keeps SELECTOR interpolation verbatim
- SCSS: `@warn` / `@error` prelude strings re-quote per `singleQuote` option (`@error "x"` → `@error 'x'`)
  - `oxc-css-parser` parses them as `SassExpr`, so they go through the structured printer (see `at_rule.rs`)
  - Prettier keeps them as a raw string verbatim
- SCSS: A function call directly after a `//` comment in nested-args position
  - Prettier double-indents it
  - We print the normal indent (prettier/prettier#19427)
- SCSS: The map-item break (one element per line + trailing comma) applies ONLY to parens whose contents are already a comma-separated list (semantics)
  - `(x,)` is a single-element list in Sass, so the added comma is a semantic no-op for a comma list and NOWHERE else
  - Prettier 3.9.5 changes `key: ($a + $b)` from a number to a list,
    restructures `key: (a b)` (2-element space list → nested 1-element list),
    and emits non-compiling output for `key: 2 * ($a + $b)` inside `$var:` declarations (dart-sass: `Undefined operation "2 * (3px,)"`)
  - Prettier's own #18530 (math siblings in args) / #19091 (single-node scalars) fixed subsets of this;
    we extend the same rule to every non-comma-list, so these stay inline
- SCSS: An own-line trailing comment before a list's closing `)` keeps its own line
  - Applies to maps AND `@use`/`@forward with (...)` configs (`$e: 5\n  // c\n)` stays as-is;
    for maps the trailing comma is also kept)
  - Prettier pulls the comment up onto the last item's line (`$e: 5 // c`, a `lineSuffix`
    artifact of its comma-group printing) and drops the map's trailing comma
  - Same-line trailing comments still glue (matching Prettier);
    moving an own-line comment up would destroy the author's visual grouping
- SCSS: A map whose FIRST item is preceded by a block comment always breaks one-per-line
  - Prettier stops treating it as a map item (the comment becomes `groups[0]`, so `isKeyValuePairInParenGroupNode` fails) and inlines it when it fits: `$b: (/* c */ a: 1);`
  - We reproduce the map-item-ness loss for the trailing comma (dropped, like Prettier)
    - But keep the forced break; matching the inline layout needs comment support in the soft map path
- SCSS: Consecutive `//` comments in a comment-only map indent uniformly
  - Prettier misaligns the second one with a stray extra leading space (`   // b`), an artifact of its `join(line)` separator printing before the deferred `lineSuffix` flushes
  - A meaningless glitch (may well be fixed upstream); we print the normal indent
- SCSS: A `;`-less custom-property rule block followed by another declaration (`--p: {color:red;} /* <- no semi */ --q: blue;`)
  - SCSS output: we treat it as two separate declarations: format the inner block, add the missing `;`, and format `--q` normally
  - Prettier behavior: keeps the whole run verbatim, postcss swallows everything past the `}` as an opaque prelude string until a source `;`
  - Why SCSS only: It falls out of the AST shape `oxc-css-parser` produces
    - SCSS parses `{...}` declaration values as `SassNestingDeclaration`, so the formatter handles them like any other nested block
    - CSS/Less do NOT structure `{...}` in declaration value position so the token-soup fallback runs, the formatter emits verbatim, and the output incidentally matches Prettier
    - Each mode is internally consistent with what its parser produces
  - The value syntax `--p: { ... }` itself is valid CSS, but its only intended consumer was the `@apply --p;` at-rule from the dropped CSS Apply Rule proposal
    - With no consumer, real-world usage is near zero, so the cross-mode behavior difference is theoretical
- Less: statement-position `&:extend(...)` breaks only on overflow, like the selector-position form
  - Prettier (3.9.5+) ALWAYS breaks multiple selectors one per line there and never breaks a single one:
    postcss-less models the statement as a rule node, so the top-level selector-list printer
    (hardline commas) leaks into the parens (prettier#19550 only fixed the indentation)
  - Ours prints BOTH positions with the same pseudo-args layout for consistency
    (inline when it fits; parens on their own lines + one selector per line on overflow, the same shape as Prettier's break)
- Less: `func(x, + 20px)` unary gluing
  - Prettier prints `+20px`; `oxc-css-parser` ASTs `, +` as a comma-left binary operation, so matching is ad-hoc for a torture-test-only shape
- Less: Nested math in a function arg / multi-value shorthand
  - Prettier's fill fit-check breaks INSIDE the wide chunk; our core `fill` (biome semantics) breaks the SEPARATOR instead.
  - Principled fix is the shared core-fill fit-check change (needs JS-conformance impact experiment first)
- Less: Value-position `@{var}` interpolation
  - `oxc-css-parser` rejects it matching `lessc`; Prettier (postcss) accepts and prints verbatim
- Less: Lookup with whitespace inside (`@config   [   option1]`)
  - `oxc-css-parser` rejects matching `lessc`; Prettier accepts

## Verification

```sh
cargo c -p oxc_formatter_css
```

Run `clippy` and resolve all warnings.

### Fixture tests

Snapshot tests driven by fixture files; covers what the Prettier conformance suite does not (placeholder at-rules, custom-property re-parsing, embedded css-in-js, etc.).

Fixtures are grouped per language (`format/{css,scss,less}/`; test modules mirror the directories), with the shared `options.json` at the `format/` / `embedded/` level (the harness walks up to the nearest one).
`embedded/scss/` is explicit about the dispatcher's variant=Scss hardcoding.

Unit tests in `tests/fixtures/mod.rs` cover parse-error `Err` semantics (`parse_error_is_err`).
Fixtures under `embedded/` route through `format_to_ir` instead of `format()`; the `embedded_debug` example formats files the same way for quick comparison.

Every expected output must be verified against Prettier (3.9.5, the current submodule).
`npx prettier@3.9.5 --parser <variant>` at both `--print-width 80` and `100` (the harness snapshots both).

Exception: a fixture may pin an entry from "Known divergences" (e.g. `map-item-parens.scss`);
its comments must say which lines deviate from Prettier and why.

```sh
cargo test -p oxc_formatter_css
# Review / accept snapshots after intentional changes
cargo insta review -p oxc_formatter_css
```

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes);
results live in `tasks/prettier_conformance/snapshots/prettier.css.snap.md` / `prettier.scss.snap.md` / `prettier.less.snap.md`.

```sh
cargo run -p oxc_prettier_conformance
# Debug a specific test
cargo run -p oxc_prettier_conformance -- --filter css/atrule
```

At the current version (v3.9.5), the divergences of six files have been confirmed and are intentional (see "Known divergences"):

- CSS: `css/stylefmt-repo/at-media/at-media.css`, `css/stylefmt-repo/cssnext-example/cssnext-example.css`, `css/postcss-plugins/postcss-nesting.css`
- SCSS: `scss/comments/4878.scss`, `scss/map/function-argument/functional-argument.scss`, `scss/variables/apply-rule.scss`

Two more files fail with MIXED hunks; they can't pass as files (the intentional hunks alone keep them failing), so the remaining diffs are itemized here:

- `css/fill-value/fill.css` (~96% match) one hunk:
  - a fill break-point inside a math-y value (`... * -1 +` vs breaking before `/ 2`);
    - the "Less: nested math fill fit-check" divergence class (core-fill semantics)
- `css/parens/parens.css` (~93% match) token-soup math spacing, three hunk classes:
  - intentional (Prettier artifact): Prettier splits SOME source-glued `-(` into `- (`
    - `prop`/`prop44`, an operator-heuristic side effect; ours keeps them all glued and consistent
    - and glues a source-spaced `+ 20px` (`prop34`, the documented Less `func(x, + 20px)` divergence in Css mode)
  - normalization-direction difference (open question, low value)
    - a math operator adjacent to a function/paren boundary gets uniform `op` spacing from Prettier regardless of source (`round(1.5)+2` -> `round(1.5) + 2`, calc `*`/`/`);
    - ours preserves the source spacing per token (`prop13/14`, `prop57-60`, `prop73/74`)
  - within-a-word runs (`1+1+1+1`, `calc(100%+2px)`) match
    - glued number-ish runs are ONE postcss word and print raw (see `is_word_glued_number`)

### Embedded conformance (`apps/oxfmt`)

The embedded-language features (css-in-js) are validated end-to-end through the Oxfmt.

Requires a dev build first.

```sh
pnpm --dir apps/oxfmt build-dev
pnpm --dir apps/oxfmt conformance
```

### Manual checks

```sh
cargo run -p oxc_formatter_css --example css_formatter file.css
cargo run -p oxc_formatter_css --example parse_debug -- --syntax scss file.scss  # dump oxc-css-parser AST
cargo run -p oxc_formatter_css --example embedded_debug file.scss                # format_to_ir entry
```

## Roadmap (TODO: Follow Prettier main)

The guiding axis is Prettier compatibility, matching what is in Prettier's unreleased changelog (main has them, next stable will).
