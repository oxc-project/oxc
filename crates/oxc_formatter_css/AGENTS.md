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

On the other hand, Prettier operates on `postcss` + three sub-parsers (`postcss-selector-parser`, `postcss-values-parser`, `postcss-media-query-parser`) and depends on `raws` (source gaps).

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
they are collected via `ParserBuilder::comments()` into a positional cursor over `CssComment { span, inline }`
(`inline` = `//`).

- Statement-level comments: flushed before each statement
  (`flush_leading_comments`); consecutive same-line comments stay glued
  (`*/ /*!`), but a comment is always followed by a line break before a node
- Value-level comments: flushed inside fill entries before the component they
  precede (`flush_value_comments`); `//` comments expand the parent group and force a hardline after
- Trailing (`value /* c */;`): flushed by `write_declaration` with the source gap before `;` preserved
- After each statement, the sequence DISCARDS unclaimed comments inside the
  statement span (cursor must never point before a printed position)

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

#### Supported: postcss-simple-vars (auto-enabled for `CssVariant::Css`)

Covered:

- `$var: value !important;` declarations (top-level and inside rules)
- `$var` references in property values
- `$var` references inside `@media`/at-rule preludes

NOT covered: `$(var)` interpolation (`margin-$(dir): 10px`, `.icon.is-$(network)`), selector-position bare `$var` (`.$prefix`), comment substitutions (`<<$(var)>>`).

### Known divergences

Deliberate divergences from Prettier (impact does not justify the matching cost):

- Less: `func(x, + 20px)` unary gluing
  - Prettier prints `+20px`; `oxc-css-parser` ASTs `, +` as a comma-left binary operation, so matching is ad-hoc for a torture-test-only shape
- Less: Nested math in a function arg / multi-value shorthand
  - Prettier's fill fit-check breaks INSIDE the wide chunk; our core `fill` (biome semantics) breaks the SEPARATOR instead.
  - Principled fix is the shared core-fill fit-check change (needs JS-conformance impact experiment first)
- Broken `:not(...)` selector args indent at +2
  - Prettier lands at +4 (arg) / +2 (`)`)
  - Layout-only, rare trigger (selector longer than line width)
- Selector-position Sass interpolation normalizes inner spaces (`#{ $name }` → `#{$name}`)
  - We normalize BOTH positions for output consistency
  - Prettier keeps SELECTOR interpolation verbatim
- `@warn` / `@error` prelude strings re-quote per `singleQuote` option (`@error "x"` → `@error 'x'`)
  - `oxc-css-parser` parses them as `SassExpr`, so they go through the structured printer (see `at_rule.rs`)
  - Prettier keeps them as a raw string verbatim
- A function call directly after a `//` comment in nested-args position
  - Prettier double-indents it
  - We print the normal indent (prettier/prettier#19427)
- A declaration swallowed by a `;`-less css-in-js placeholder (`${m}\ncolor: red`)
  - We parse it structurally and FORMAT it (spacing/hex/number normalization)
  - Prettier keeps it verbatim, postcss swallows the run as an opaque prelude string it can't format, so `color   :   red` / `#FFFFFF` survive unformatted
- SCSS: A `;`-less custom-property rule block followed by another declaration (`--p: {color:red;} /* <- no semi */ --q: blue;`)
  - SCSS output: we treat it as two separate declarations: format the inner block, add the missing `;`, and format `--q` normally
  - Prettier behavior: keeps the whole run verbatim, postcss swallows everything past the `}` as an opaque prelude string until a source `;`
  - Why SCSS only: It falls out of the AST shape `oxc-css-parser` produces
    - SCSS parses `{...}` declaration values as `SassNestingDeclaration`, so the formatter handles them like any other nested block
    - CSS/Less do NOT structure `{...}` in declaration value position so the token-soup fallback runs, the formatter emits verbatim, and the output incidentally matches Prettier
    - Each mode is internally consistent with what its parser produces
  - The value syntax `--p: { ... }` itself is valid CSS, but its only intended consumer was the `@apply --p;` at-rule from the dropped CSS Apply Rule proposal
    - With no consumer, real-world usage is near zero, so the cross-mode behavior difference is theoretical
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

Every expected output must be verified against Prettier (3.8.4, the current submodule).
`npx prettier@3.8.4 --parser <variant>` at both `--print-width 80` and `100` (the harness snapshots both).

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

At the current version (v3.8.4), the divergences of two files has been confirmed in the SCSS conformance, but this is intentional.

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
