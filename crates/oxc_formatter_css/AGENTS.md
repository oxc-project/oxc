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

Parses with a fork of [`raffia`](https://docs.rs/raffia), pinned via `rev` in the workspace `Cargo.toml`.

The fork adds:

- `tolerate_at_keyword_placeholders` option (for the css-in-js dispatcher)
  - Accepts at-keywords in declaration values (`ComponentValue::TokenWithSpan`) and selectors (type/class selector idents whose `raw` INCLUDES the leading `@`)
  - Only `format_to_ir` enables it; `format()` stays strict
- Whitespace-sensitive Less `+`/`-` operators (matching `lessc`)
  - A `+`/`-` is a `LessBinaryOperation` operator only when followed by whitespace;
  - `@a -@b` is two values (`-@b` is a `LessNegativeValue` sign
  - `margin: -@a -@b` = two values, NOT `(-@a) - @b` subtraction), `@a - @b` is subtraction
- Various bug fixes for valid CSS/SCSS/Less syntax `raffia` miss-parses or rejects
  - selector / at-rule / value-token coverage gaps

On the other hand, Prettier operates on `postcss` + three sub-parsers (`postcss-selector-parser`, `postcss-values-parser`, `postcss-media-query-parser`) and depends on `raws` (source gaps).

`raffia` parses everything structurally in one pass; source gaps are recovered by comparing span boundaries (`hasEmptyRawBefore(x)` == "no gap between spans").

### Error semantics

`format()` / `format_to_ir()` return `Err` whenever they cannot produce output they can stand behind:

- `raffia` is error-tolerant via `parser.recoverable_errors()`, but any parse error bails out;
  - Never format a broken AST
  - Exception: `TopLevelDeclaration`, the dominant css-in-js shape (postcss accepts it)
- print-stage internal errors are also `Err`
- The caller (oxfmt) decides what happens next
  (diagnostics for standalone files, template-as-is for embedded)

### Comments

`raffia` does not attach comments to the AST;
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

`format_to_ir()` accepts SCSS-like source with `@prettier-placeholder-N-id` markers in place of `${}` interpolations.
`raffia` parses them via the fork option `tolerate_at_keyword_placeholders` (`format_to_ir` passes `tolerate_placeholders: true`).

Per-position handling:

- Statement position: parses as an at-rule (`write_placeholder_at_rule` in `at_rule.rs`)
  - A `;`-less marker SWALLOWS the following statements into its prelude
- Value position: `ComponentValue::TokenWithSpan`, rides existing gap-based separator rules
  - One added rule: glued to a paren group → `Separator::SoftBreak` (`${fn}(30px)` breaks BEFORE the parens)
- Selector position: triggers "garbage mode" in `write_selector_list`
  - Emits the raw source slice with whitespace runs collapsed, never breaking
  - Mirrors `postcss-selector-parser` degrading on at-words

## Prettier mapping

### Unknown at-rule params print VERBATIM

Prettier's parser hands params to sub-parsers only for a fixed allowlist (see `parser-postcss.js`, `is_value_parsed_at_rule`);
everything else (`@apply`, `@tailwind`, `@custom-variant`, `@variant`, `@source`, ICSS `@value`, etc) stays a plain string the printer emits raw (`write_verbatim_at_rule_tail`).

Re-spacing those tokens CORRUPTS Tailwind syntax: `dark:bg-x` → `dark: bg-x`, `py-1.5` → `py-10.5`, `@custom-variant dark (&:is(...))` → `dark(&: is(...))`.

We also follow this to keep Prettier compatibility.

### Tailwind `@apply` sorting (`CssFormatOptions::sort_tailwindcss`)

Ports prettier-plugin-tailwindcss's `transformCss`: with the option on, `@apply` params become `FormatElement::TailwindClass(index)` elements, and a host-supplied `TailwindSorter` performs the actual ordering/dedup outside this crate.

See `write_apply_prelude` in `at_rule.rs` (collection + `!important` / Less `~"..."` extraction) and `format.rs` (sorter dispatch).

### Intentionally unsupported: postcss plugin syntax

These syntax that only "works" in Prettier because `postcss` parses without validation and a build-time plugin interprets it later is mostly NOT supported.
(`postcss` is permissive with any syntax it doesn't understand, while we parse strictly with `raffia`.)

These plugins were once common but are now mostly legacy. Representative examples we do NOT support:

- `postcss-simple-vars`: `$blue: #056ef0;` declarations, value-position `$blue`, `$(dir)` interpolation
- `postcss-mixins`: parametered `@define-mixin x $a, $b` and `$var` inside body
  - Plain `@define-mixin icon {}` / `@mixin icon;` / `@mixin icon a, b;` DO parse fine
- `postcss-nested-props` (`font: { ... }`), ICSS nested `:export { nest: {} }`, `--element(...)` (CSS Extensions, zero implementations)

Failures emit a LOUD diagnostic; ignore-listing is the escape hatch.

We DO support, however, the following popular plugin-flavored syntaxes:

- Tailwind v3/v4 at-rules: `@tailwind`, `@apply`, `@layer`, `@theme`, `@utility`, `@variant`, `@config`, `@custom-media`
- CSS Modules constructs: `@value` (incl. `from`), `:global`/`:local`, `composes`, plain `:import`/`:export`
- Standard CSS nesting

Less also rejects (matching `lessc`):

- Value-position `@{var}` interpolation
- Whitespace inside a Less lookup (`@config   [   option1]`)

If there is high demand, we can also consider making some parts acceptable by updating the `raffia` parser side.

### Known divergences

Deliberate divergences from Prettier (impact does not justify the matching cost):

- Less `func(x, + 20px)` unary gluing
  - Prettier prints `+20px`; `raffia` ASTs `, +` as a comma-left binary operation, so matching is ad-hoc for a torture-test-only shape
- Nested Less math in a function arg / multi-value shorthand
  - Prettier's fill fit-check breaks INSIDE the wide chunk; our core `fill` (biome semantics) breaks the SEPARATOR instead.
  - Principled fix is the shared core-fill fit-check change (needs JS-conformance impact experiment first)
- Broken `:not(...)` selector args indent at +2
  - Prettier lands at +4 (arg) / +2 (`)`)
  - Layout-only, rare trigger (selector longer than line width)
- Selector-position Sass interpolation normalizes inner spaces (`#{ $name }` → `#{$name}`)
  - We normalize BOTH positions for output consistency
  - Prettier keeps SELECTOR interpolation verbatim
- A function call directly after a `//` comment in nested-args position
  - Prettier double-indents it
  - We print the normal indent (prettier/prettier#19427)

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
cargo run -p oxc_formatter_css --example parse_debug -- --syntax scss file.scss  # dump raffia AST
cargo run -p oxc_formatter_css --example embedded_debug file.scss                # format_to_ir entry
```

## Roadmap (TODO: Follow Prettier main)

The guiding axis is Prettier compatibility, matching what is in Prettier's unreleased changelog (main has them, next stable will).

- [#18605](https://github.com/prettier/prettier/blob/main/changelog_unreleased/css/18605.md):
  Don't break a selector when its attribute value contains an escaped literal newline (`foo="long\\<newline>continuation"`).
  We currently break before the long span; Prettier main keeps the selector on one line.
