# Coding agent guides for `crates/oxc_formatter_css`

Follow @../oxc_formatter_core/FORMATTER_POLICY.md , this file holds only the CSS/SCSS/Less-specific rules and translations.
Known divergences live in DIVERGENCES.md.

## Overview

Prettier compatible CSS/SCSS/Less formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- Entry points:
  - `format()`: standalone files, on a service-less session
  - `format_with_session()`: standalone, on the caller's `FormatSession`
  - `format_to_ir()`: embedded use via the dispatcher (`template_placeholders` = the css-in-js parse mode)

### Forked parser

Parses with [`oxc-css-parser`](https://crates.io/crates/oxc-css-parser), a `raffia` fork pinned via the workspace `Cargo.toml`.

The fork adds:

- `template_placeholder` option: backtick-delimited `` `<prefix><digits>` `` markers tokenized as one typed `Token::Placeholder`
  - the css-in-js parse mode, enabled only by `format_to_ir`
  - Rationale, gating and per-position coverage: see "css-in-js specifics" below
- Bug fixes toward the reference compilers for valid CSS/SCSS/Less syntax `raffia` miss parses or rejects;
  - unlike the leniencies below these may change the AST of input that already parsed
    - e.g. lessc's whitespace-sensitive `+`/`-`: `margin: -@a -@b` is two values, not subtraction
- Additive leniencies for syntax reference compilers reject but postcss (and so Prettier) accepts
  - e.g. the IE `*color` hack, raw-prelude rules like `sans: "Sans" { ... }`
  - Contract (additive-only, code comment + test pin per leniency) and triage:
    - see "Policy: how to take in non-spec / non-Sass dialect syntax" below

Prettier operates on `postcss` + three sub-parsers (`postcss-selector-parser`, `postcss-values-parser`, `postcss-media-query-parser`) and depends on `raws` (source gaps).

`oxc-css-parser` parses everything structurally in one pass; source gaps are recovered by comparing span boundaries (`hasEmptyRawBefore(x)` == "no gap between spans").

### Error semantics

The shared policy applies; CSS specifics:

- `oxc-css-parser` is error-tolerant via `parser.recoverable_errors()`, but any parse error still bails out
- Exception: `TopLevelDeclaration`, tolerated ONLY by `format_to_ir()`
  - The dominant css-in-js shape, `` css`display: flex;` ``
  - Standalone `format()` still rejects it as invalid CSS/SCSS/Less (Dart Sass rejects it too)

### Comments

`oxc-css-parser` does not attach comments to the AST;
they are collected via `ParserBuilder::comments()` into a positional cursor over `CssComment { span, inline }` (`inline` = `//`).

- Statement-level comments: flushed before each statement (`flush_leading_comments`);
  consecutive same-line comments stay glued (`*/ /*!`), but a comment is always followed by a line break before a node
- Value-level comments: block comments between fill runs are standalone fill items (own-line or not);
  the rest (leads before the first component, own-line `//`) flush at the next entry's head (`flush_value_comments`),
  where `//` comments expand the parent group and force a hardline after
- Trailing (`value /* c */;`): flushed by `write_declaration` with the source gap before `;` preserved
- After each statement, the sequence DISCARDS unclaimed comments inside the statement span
  (cursor must never point before a printed position)

#### Placement invariants

The shared invariants (FORMATTER_POLICY.md "Comment placement invariants") apply; this section records their CSS translation:

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
- Line-boundary rule in CSS terms: `//` comments force a hardline after;
  - own-line comments stay own-line at statement and trailing level, but a value-level own-line BLOCK comment is a plain fill item (joins the line when it fits):
    - it carries no line-based semantics, and freezing it own-line would pin a wrapped layout (= not idempotent)

### Line endings

`parse_stylesheet` normalizes `\r\n` and lone `\r` to `\n` BEFORE parsing.

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

### Front matter (yaml-in-css)

The envelope contract (host opt-in, detection, frame composition, refusal semantics) is core's (`write_front_matter` / `spec::front_matter`; boundary layer 5 in `oxc_formatter_core`'s AGENTS.md); this crate's side (the embeddable set, blank-and-compose wiring, the gap rule) lives in `format.rs`.
FM behavior is verified through oxfmt; this crate carries no dispatcher-wired FM tests.

## Prettier mapping

### Unknown at-rule params print VERBATIM

Prettier's parser hands params to sub-parsers only for a fixed allowlist (`is_value_parsed_at_rule`);
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

1. Unknown at-rule prelude verbatim (`write_verbatim_at_rule_tail`) zero-cost bucket: Tailwind, postcss-mixins, ICSS ride it for free
2. Raw fallbacks when the typed grammar rejects (raw component values, `TokenSeq`, `ImportPrelude.modifiers`, `UnknownQualifiedRule`) `[attr=;]`, weird import tails, nested config blocks
3. postcss word rules at the separator layer (`is_word_glued_number`, the `1#{$var}` glue, solidus words) variant-agnostic, fixes xstyled + `theme()` + future unknown tokens in one place
4. `ParserOptions` flag + typed node (postcss-simple-vars) ONLY when the formatter must make layout decisions INSIDE the construct.
   Promotion criteria, all three:
   (a) real user demand, (b) Prettier itself formats it structurally (not verbatim), (c) rungs 1-3 can't express it
5. A dedicated `CssVariant` only for real languages with reference compilers (css/scss/less).
   Never for a plugin.

Parser-side leniencies (in the `oxc-css-parser` fork) must be additive:
accept only input that previously errored, and never change the AST of input that already parsed (e.g. dotted words try the typed `foo.$var` / `foo.bar(...)` parse first; only dart-sass-invalid shapes take the lenient path).
Every lenient path carries a comment citing the reference-compiler vs postcss behavior, a test pinning the strict shapes, and shows up as a visible expected-error flip in the parser's conformance snapshots.

Triage order for reports follows the shared policy (don't corrupt → then accept → then pretty-print); the CSS-shaped example of the UNSAFE failure is silent token corruption like `sandstone.10` → `sandstone 0.1`.

Red flags that the approach is drifting:
specific plugin names accumulating in code, or a leniency that reinterprets previously-valid input.
Either means the fix is at the wrong rung.

#### Supported: postcss-simple-vars (auto-enabled for `CssVariant::Css`)

Covered: `$var: value !important;` declarations (top-level and inside rules), `$var` references in property values and `@media`/at-rule preludes.

NOT covered: `$(var)` interpolation (`margin-$(dir): 10px`, `.icon.is-$(network)`), selector-position bare `$var` (`.$prefix`), comment substitutions (`<<$(var)>>`).

## Verification

### Fixture tests

The harness snapshots both `--print-width 80` and `100`; verify fixtures at both widths.

### Prettier conformance

At the current version (v3.9.6), these divergences have been confirmed and are intentional (see DIVERGENCES.md):

- CSS: `css/stylefmt-repo/at-media/at-media.css`, `css/stylefmt-repo/cssnext-example/cssnext-example.css`, `css/stylefmt-repo/media-queries-ranges/media-queries-ranges.css`, `css/postcss-plugins/postcss-nesting.css`
- SCSS: `scss/comments/4878.scss`, `scss/map/function-argument/functional-argument.scss`, `scss/parens/issue-16594.scss`, `scss/variables/apply-rule.scss`, `scss/trailing-comma/comments.scss`, `scss/trailing-comma/list.scss`, `scss/trailing-comma/variable.scss`, `scss/function/arbitrary-arguments-comment.scss`, `scss/map/15193.scss`

Two more files fail with MIXED hunks; they can't pass as files (the intentional hunks alone keep them failing), so the remaining diffs are itemized here:

- `css/fill-value/fill.css` (~96% match) one hunk:
  - a fill break-point inside a math-y value (`... * -1 +` vs breaking before `/ 2`);
    - the DIVERGENCES.md "fill-break-position" class (core-fill semantics)
- `css/parens/parens.css` (~93% match) token-soup math spacing, three hunk classes:
  - intentional: Prettier splits SOME source-glued `-(` into `- (` (`prop`/`prop44`, DIVERGENCES.md "css-glued-minus-paren")
    and glues a source-spaced `+ 20px` (`prop34`, DIVERGENCES.md "unary-plus-glue")
  - normalization-direction difference (open question, low value)
    - a math operator adjacent to a function/paren boundary gets uniform `op` spacing from Prettier regardless of source (`round(1.5)+2` -> `round(1.5) + 2`, calc `*`/`/`);
    - ours preserves the source spacing per token (`prop13/14`, `prop57-60`, `prop73/74`)
  - within-a-word runs (`1+1+1+1`, `calc(100%+2px)`) match
    - glued number-ish runs are ONE postcss word and print raw (see `is_word_glued_number`)

### Manual checks

```sh
cargo run -p oxc_formatter_css --example css_formatter file.css                  # defaults to --print-width 80
cargo run -p oxc_formatter_css --example parse_debug -- --syntax scss file.scss  # dump oxc-css-parser AST
cargo run -p oxc_formatter_css --example embedded_debug file.scss                # format_to_ir entry
```

`DUMP_IR=1` prints the `FormatElement` stream before printing.
