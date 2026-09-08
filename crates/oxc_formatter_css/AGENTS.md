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
  - `parse_for_format()`: the parse `format()` runs, exposed for callers that inspect the AST (e.g. the fixture harness's fingerprint)

### Forked parser

Parses with [`oxc-css-parser`](https://crates.io/crates/oxc-css-parser), a `raffia` fork pinned via the workspace `Cargo.toml`.

The fork adds:

- `template_placeholder` option: backtick-delimited `` `<prefix><digits>` `` markers tokenized as one typed `Token::Placeholder`
  - the css-in-js parse mode, enabled only by `format_to_ir`
  - Rationale, gating and per-position coverage: see "css-in-js specifics" below
- Bug fixes toward the reference compilers for valid CSS/SCSS/Less syntax `raffia` miss parses or rejects;
- The acceptance line: the fork's README "Acceptance", see "Acceptance" below

Prettier operates on `postcss` + three sub-parsers (`postcss-selector-parser`, `postcss-values-parser`, `postcss-media-query-parser`) and depends on `raws` (source gaps).

`oxc-css-parser` parses everything structurally in one pass; source gaps are recovered by comparing span boundaries (`hasEmptyRawBefore(x)` == "no gap between spans").

### Error semantics

The shared policy applies; CSS specifics:

- `oxc-css-parser` is error-tolerant via `parser.recoverable_errors()`, but any parse error still bails out
- `TopLevelDeclaration`: a root declaration in Css and Scss, so standalone `format()` rejects it (README "Acceptance")
  - In the css-in-js parse mode (`template_placeholder`, Scss-only) the parser treats it as a statement and emits nothing
    - `` css`display: flex;` `` is the dominant css-in-js shape
  - Less parses and formats it: less.js accepts it at parse time and fails only at eval

### Comments

`oxc-css-parser` does not attach comments to the AST;
they are collected via `ParserBuilder::comments()` into a positional cursor over `CssComment { span, inline }` (`inline` = `//`).

Ownership is a claim discipline, not attachment: each printer claims the comments inside what it prints, in source order.

- Statement level: `flush_leading_comments` before the statement, `write_terminator_tail_comments` before its `;`
- Value level: a comma group claims its own head (`write_comma_group`); a bare value has no group to do so,
  its CALLER claims (`write_list_element` / `write_top_level_list_element`), never `write_component_value` itself (a `//` hardline would land inside the indent an arm opens)
- After each statement, the sequence DISCARDS unclaimed comments inside the statement span:
  a monotonicity guard (the cursor must never point before a printed position), not a placement rule.
  A comment reaching it is LOST (the lossless contract), so every new value position must claim its leads

#### Placement invariants

The shared invariants (FORMATTER_POLICY.md "Comment placement invariants") apply; this section records their CSS translation:

- `,` is a list SEPARATOR: a comment between an element and its comma stays BEFORE the comma
  - `a /* c */, b`; comments after it lead the next element, except a `//` on the comma's line (line-boundary rule below)
  - Declaration value lists (`write_value_groups`) and function arguments (`write_function`) route every comma through `write_group_comma` with the comma offset paired to its group
  - `split_comma_groups` returns `(group, Option<comma_start>)`; SCSS/Less lists pair `comma_spans`
  - A new comma site must take the pair, the shape makes taking the groups without the commas a visible choice, not an accident
  - Adopted by every comma writer (function/include args, maps, paren/sass lists, `@mixin` params, `@each` bindings, keyframe selectors)
- `;` is a declaration TERMINATOR, but unlike JS statements Prettier does NOT move comments behind it:
  - `value /* c */;` keeps the comment before `;` (measured behavior, not principle, may change in the future)
  - `oxc_formatter` prints `1; /* c */` (comment behind the terminator); CSS keeps it before, uniformly for declarations, `$var` / `@var` values and `!flag`s (`write_terminator_tail_comments`).
    - Revisit together with JS if the policy ever picks one side
  - The gap before the `;` is the formatter's and is dropped (`/* c */ ;` -> `/* c */;`), see DIVERGENCES.md "terminator-gap-normalized"
- The positional cursor makes ownership a bounds discipline, not an attachment one:
  - a flush's upper bound must never extend past the next piece of user content,
  - and a declaration's `tail_bound` may only be consumed by the LAST comma group (`write_value_groups` clears it for every other group)
- Line-boundary rule in CSS terms: `//` comments force a hardline after;
  - a `//` on a list `,`'s line stays there (`1, // c`, like JS), together with the block comments glued before it on that line;
    Prettier's CSS moves it below as the next element's leading comment (DIVERGENCES.md "line-comment-after-comma")
  - a `//` before a list `,` rides past it (`a // c\n, b` -> `a, // c`), an own-line comment there leads the next element;
    a `//` glued to a prelude's end stays there and `{` starts the next line (DIVERGENCES.md "line-comment-before-block")
  - a structured prelude printer places its own comments, word by word (`value::write_with_comments`: own-line ones lead the word, glued ones trail it),
    and `write_at_rule` writes whatever is still pending before the `;` / `{`, so a prelude comment is never lost
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

### Acceptance

What the parser takes in is the oxc-css-parser's README "Acceptance": one grammar owner per variant.
Never "postcss keeps the bytes", never "Prettier prints it".

The formatter's side: postcss gives Prettier statements whose selectors, values and params are strings, and Prettier re-tokenizes those with its own sub-parsers and word rules.
We have a typed grammar instead, so Prettier is the oracle for the layout of what both sides structure the same way;
where its string handling produces a shape our grammar does not (a re-spaced raw run, a re-split word), DIVERGENCES.md records the case and we do not follow.
Below a statement the typed grammar is the only structure; raw is an admission (a hole in our grammar, or bytes that ARE the value).

What the formatter relies on per shape:

- postcss property names: printed verbatim, lowercased
- raw-prelude rules (`x: { ... }`, numeric-led statements): `UnknownQualifiedRule`, the prelude prints verbatim
- root declarations: see "Error semantics"
- `.scss` / `.less` must compile; a shape only postcss-scss / postcss-less accepts is a report to close, not a gap
- Tailwind (`@tailwindcss/postcss`), postcss-simple-vars, nested-config dialects all sit inside the lines above

### Printing raw vs typed

- Raw is verbatim where the grammar owner says the bytes ARE the value:
  a raw name (with the usual lowercasing), a raw prelude, unknown at-rule params, and a custom property value the typed grammar could not read.
  A postcss-simple-vars value the typed grammar could not read is verbatim too: the plugin substitutes those bytes textually.
  "Raw plus a little normalization" is the failure mode: re-spacing unknown params corrupts Tailwind syntax (`dark:bg-x` → `dark: bg-x`, `py-1.5` → `py-10.5`).
  A normal property's `<any-value>` fallback is not raw in this sense; it rides the value writer
- The print-layer word heuristics (`value.rs`) rebuild postcss's word model from spec tokens.
  A new one is admitted when its scope is bounded and it improves how structured output is laid out;
  never to match a fixture, and never to reproduce Prettier's string handling

### Absorbing dialect tokens

Plugin dialects (`xstyled` dotted tokens, Tailwind `theme()` paths, postcss plugin at-rules, ...) are absorbed at the highest rung that covers them, cheapest first:

1. Unknown at-rule params verbatim: Tailwind, postcss-mixins, ICSS ride it for free
2. Raw fallbacks when the typed grammar rejects
3. A word heuristic, under the conditions above
4. A typed node, no flag, ONLY when the formatter must make layout decisions INSIDE the construct and Prettier formats it structurally (`PostcssSimpleVar`)
5. A dedicated `CssVariant` only for real languages with reference compilers. Never for a plugin

A report is first translated into "which line, which shape or which rung is this?", never "how do we support plugin Xxx?".
Triage follows the shared policy (don't corrupt → then accept → then pretty-print); the CSS-shaped UNSAFE failure is silent token corruption like `sandstone.10` → `sandstone 0.1`.
Specific plugin names accumulating in code is the red flag.

### Tailwind `@apply` sorting (`CssFormatOptions::sort_tailwindcss`)

Ports prettier-plugin-tailwindcss's `transformCss`: `@apply` params become `FormatElement::TailwindClass(index)` elements
and a host-supplied `TailwindSorter` does the ordering and dedup outside this crate.

### postcss-simple-vars (Css only)

Covered: `$var: value !important;` declarations (root and inside rules), `$var` references in property values and at-rule preludes,
`$(var)` interpolation in property names.

NOT covered: `$(var)` interpolation inside values and selectors (`.icon.is-$(network)`), selector-position bare `$var` (`.$prefix`), comment substitutions (`<<$(var)>>`).

## Verification

### Fixture tests

The harness snapshots both `--print-width 80` and `100`; verify fixtures at both widths.

### Prettier conformance

At the current version (v3.9.6), these divergences have been confirmed and are intentional (see DIVERGENCES.md):

- CSS: `css/stylefmt-repo/at-media/at-media.css`, `css/stylefmt-repo/cssnext-example/cssnext-example.css`, `css/stylefmt-repo/media-queries-ranges/media-queries-ranges.css`, `css/postcss-plugins/postcss-nesting.css`, `css/comments/declaration.css` (terminator-gap-normalized),
  `css/postcss-8-improment/test.css` (custom-property-raw-verbatim), `css/parens/empty-lines.css` (postcss-simple-var-raw-verbatim)
- SCSS: `scss/comments/4878.scss`, `scss/map/function-argument/functional-argument.scss`, `scss/parens/issue-16594.scss`, `scss/trailing-comma/comments.scss`, `scss/trailing-comma/list.scss`, `scss/trailing-comma/variable.scss`, `scss/function/arbitrary-arguments-comment.scss`, `scss/map/15193.scss`, `scss/comments/variable-declaration.scss` (terminator-gap-normalized), `scss/variables/postcss-8-improment.scss` (custom-property-raw-verbatim),
  `scss/comments/4594.scss`, `scss/comments/lists.scss`, `scss/comments/maps.scss`, `scss/trailing-comma/issue-6920.scss` and one more hunk of `scss/trailing-comma/comments.scss` (line-comment-after-comma)
- Less: `less/comments/value-lists.less` (line-comment-after-comma), `less/postcss-8-improment/test.less` (custom-property-raw-verbatim)

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
