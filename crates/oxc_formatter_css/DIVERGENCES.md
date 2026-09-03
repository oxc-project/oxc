# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## trailing-line-comment-print-width

- Why: uniform-rule (line_suffix is never measured)
- Pin: `tests/fixtures/format/less/trailing-inline-comment-width.less`

```scss
/* input */
@background-color-light: fade(@white, 4%); // background of header and selected item

/* ours */
@background-color-light: fade(@white, 4%); // background of header and selected item

/* prettier */
@background-color-light: fade(
  @white,
  4%
); // background of header and selected item
```

An end-of-line trailing `//` rides a `line_suffix` and never counts toward the print width,
the treatment every other formatter crate (and Prettier itself outside the CSS family) gives line comments;
Prettier's postcss printer prints `//` inline like `/* */`, measures it, and breaks the value on overflow.
Scope: END-OF-LINE positions only (statement-level trailing, SCSS map/config trailing);
a value-interior `//` (before `)`, glued to an argument, inside a fill entry) stays inline because tokens still follow on the line.
Trailing `/* */` comments still count, matching Prettier (self-delimiting comments are inline content).

## keyframe-selector-comment-list

- Why: uniform-rule (comment presence never changes layout)
- Pin: `tests/fixtures/format/css/keyframe-selector-comment.css`

```css
/* input */
@keyframes x { 60%   /* mid */  ,   70% { opacity: 1; } }

/* ours */
@keyframes x {
  60% /* mid */,
  70% {
    opacity: 1;
  }
}

/* prettier */
@keyframes x {
  60%   /* mid */  ,   70% {
    opacity: 1;
  }
}
```

Prettier keeps a COMMENTED keyframe selector list verbatim, interior spacing included;
ours prints commented and uncommented lists with the same structural layout (one selector per line, comments per the separator rule).
Same rule as `comment-preceded-map-indent`.

## nest-params-indent

- Why: uniform-rule (same construct, same output: selector-list continuation outside `@nest`)
- Pin: `tests/fixtures/format/css/nest-params-indent.css`

```css
/* input */
a {
  @nest .some-long-prefix-selector &, .another-long-prefix-selector &, .third & {
    order: 2;
  }
}

/* ours */
a {
  @nest .some-long-prefix-selector &,
  .another-long-prefix-selector &,
  .third & {
    order: 2;
  }
}

/* prettier */
a {
  @nest .some-long-prefix-selector &,
    .another-long-prefix-selector &,
    .third & {
    order: 2;
  }
}
```

`@nest <selector-list>` continuation lines indent at +2, how selector lists indent everywhere else;
Prettier lands at +4 (comma-separated selectors, above) / +6 (wrapped selector parts, also pinned), an artifact of its generic at-rule params indent.
Layout-only, deprecated syntax, triggers only on width overflow.

## nth-anb-leading-sign

- Why: semantics
- Pin: `tests/fixtures/format/css/nth-an-plus-b.css`

```css
/* input */
:nth-child(+3n - 2)

/* ours */
:nth-child(+3n - 2)

/* prettier */
:nth-child(+ 3n - 2)
```

The An+B grammar forbids whitespace between a leading sign and its term,
so Prettier's output (`postcss-selector-parser` tokenizes every `+` as a combinator) no longer parses as a selector and breaks formatter idempotency: the second pass fails to parse.

## forward-members-with-config

- Why: uniform-rule (same construct, same output: `@use ... with (...)` and config-less `@forward`)
- Pin: `tests/fixtures/format/scss/forward-members-wrap.scss`

```scss
/* input */
@forward "a" show b, c with ($a: 1);

/* ours */
@forward "a" show b, c with (
  $a: 1
);

/* prettier */
@forward "a" show b,
  c with (
    $a: 1
  );
```

Prettier parses the whole `@forward` prelude as ONE comma list, so the config's forced break spills into
the member commas even when the head fits, and the config body lands one level deeper (+4 body / +2 `)`).
We break members only on width overflow (fill, matching Prettier's break positions when no config is present)
and keep the config at the standalone `with (...)` indent (+2 body / 0 `)`, same as `@use`). Layout-only, rare combo.

## forward-first-member-overflow

- Why: uniform-rule (same construct, same output: `@use` members)
- Pin: `tests/fixtures/format/scss/module-head-seams.scss`

```scss
/* input */
@forward "path" show extremely-long-first-member-name-that-overflows-the-print-width, second-one;

/* ours */
@forward "path" show
  extremely-long-first-member-name-that-overflows-the-print-width, second-one;

/* prettier */
@forward "path" show
    extremely-long-first-member-name-that-overflows-the-print-width,
  second-one;
```

The prelude head is one flat fill breaking at token seams with a +2 continuation, matching Prettier's break points (incl. the trailing-`;` exclusion from the last chunk's fit).
EXCEPT when the FIRST member alone overflows: Prettier indents it at +4 and own-lines every later member at +2 (artifacts of its nested comma-chunk fill);
our flat fill packs continuation members at +2, the shape Prettier prints for `@use` members.
Heads that still never break in Prettier's value-parse (`@for` bounds, `@namespace`) would extend the same fill shape if reported.

## general-enclosed-whitespace

- Why: uniform-rule (same construct, same output: parsable `<media-condition>` preludes)
- Pin: `tests/fixtures/format/css/media-general-enclosed.css`

```css
/* input */
@media (not ( screen and ( color ) )) { }

/* ours */
@media (not (screen and (color))) {
}

/* prettier */
@media (not ( screen and ( color ) )) {
}
```

`<general-enclosed>` preludes (unparsable as `<media-condition>`) normalize whitespace fully: source gap → one space, paren inner edges tight.
Prettier only collapses space RUNS inside the unparsable paren;
reproducing that half-normalization is pure tokenizer-artifact matching.
Gap-based spacing never fuses tokens the source kept apart (`and (` can't become a function token `and(`).

## value-glued-bracket

- Why: uniform-rule (same construct, same output: `foo[0.50]`)
- Pin: `tests/fixtures/format/css/word-glued-bracket.css`

```css
/* input */
width: var(--x)[0];

/* ours */
width: var(--x)[0];

/* prettier */
width: var(--x) [0];
```

A source-glued value-position `[...]` stays glued to ANY typed left neighbor and prints verbatim:
one gap-based rule (never add a space the source doesn't have) for all variants.
Prettier lexes `theme(fontSize.af-md[0])` / `foo[0.50]` as ONE postcss word (matching us), but its space after `)` is a word-lexing artifact (`[` extends a word, not across `)`).
Less lookups (`@config[@key]`) are unaffected: the typed lookup rule wins and keeps printing structurally.

## custom-media-glued-prelude

- Why: uniform-rule (same construct, same output: `@media` prelude)
- Pin: `tests/fixtures/format/css/custom-media.css`

```css
/* input */
@custom-media --viewport-medium(width<=50rem);

/* ours */
@custom-media --viewport-medium (width <= 50rem);

/* prettier */
@custom-media --viewport-medium(width<=50rem);
```

`@custom-media` preludes always print structured (`--name <media-query-list>`), identical to the same query inside `@media`, where Prettier agrees.
With the name GLUED to the `(`, Prettier keeps the whole prelude verbatim (ONE `media-type` token);
with the name spaced it still only collapses whitespace RUNS (`(  width  >=500px )` → `(width >=500px)`, glue kept).

## placeholder-swallow-run

- Why: uniform-rule (same construct, same output: the same declaration outside a placeholder run)
- Pin: `tests/fixtures/embedded/scss/statement-placeholders.scss`

```scss
/* input (css-in-js: ${m} is a template placeholder) */
${m}
color   :   red;

/* ours */
${m}
color: red;

/* prettier */
${m}
color   :   red;
```

A declaration swallowed by a `;`-less css-in-js placeholder is parsed structurally and FORMATTED (spacing/hex/number normalization);
Prettier keeps it verbatim because postcss swallows the run as an opaque prelude string it can't format.

## selector-interpolation-spaces

- Why: uniform-rule (same construct, same output: `#{ }` in value position)
- Pin: `tests/fixtures/format/scss/interpolation-quotes.scss`

```scss
/* input */
.text #{ $name } { font-weight: bold; }

/* ours */
.text #{$name} {
  font-weight: bold;
}

/* prettier */
.text #{ $name } {
  font-weight: bold;
}
```

Selector-position Sass interpolation normalizes inner spaces like value-position interpolation does in both formatters;
Prettier keeps SELECTOR interpolation verbatim.

## warn-error-requote

- Why: uniform-rule (option governs: singleQuote)
- Pin: `tests/fixtures/format/scss/unknown-at-rule-edges.scss`

```scss
/* input */
@error 'single quotes get normalized';

/* ours */
@error "single quotes get normalized";

/* prettier */
@error 'single quotes get normalized';
```

`@warn` / `@error` prelude strings re-quote per the `singleQuote` option: `oxc-css-parser` parses them as `SassExpr`, so they go through the structured printer (see `at_rule.rs`);
Prettier keeps them as a raw string verbatim.
Every other string in a declaration value re-quotes per the same option in both formatters.

## call-after-line-comment-indent

- Why: uniform-rule (comment presence never changes layout; prettier/prettier#19427)
- Pin: `tests/fixtures/format/scss/inline-comment-before-call.scss`

```scss
/* input */
width: pow(2, pow(2,
// c
pow(2, 2)));

/* ours */
width: pow(
  2,
  pow(
    2,
    // c
    pow(2, 2)
  )
);

/* prettier */
width: pow(
  2,
  pow(
    2,
    // c
    pow(2, 2)
    )
);
```

Prettier double-indents a function call directly after a `//` comment in nested-args position, a forgotten leftover of the incomplete prettier/prettier#4878 fix (prettier/prettier#7844 covered only the SCSS-map / direct paren-group child case);
ours sits at the normal +1 / `)` +0 like every comment-free call.
Same rule as `comment-preceded-map-indent`.

## nested-map-context-indent

- Why: uniform-rule (same construct, same output: the same map outside a control directive)
- Pin: `tests/fixtures/format/scss/nested-map-in-block.scss`

```scss
/* input */
@if true {
  $in-if: ($k: ($n: $v));
}

/* ours */
@if true {
  $in-if: (
    $k: (
      $n: $v,
    ),
  );
}

/* prettier */
@if true {
  $in-if: (
    $k: (
        $n: $v,
      ),
  );
}
```

A nested map value prints at the SAME indent in every block context;
Prettier double-indents it (closing `)` floating between levels) when the nearest at-rule ancestor is a control directive
(`@if`/`@else`/`@for`/ `@each`/`@while`; selector blocks in between don't shield), identical source, different indent per context.

## comment-preceded-map-indent

- Why: uniform-rule (comment presence never changes layout)
- Pin: `tests/fixtures/format/scss/map-comment-block-value-comma.scss`

```scss
/* input */
$m: (
  /* c */
  b: (x: 2),
);

/* ours */
$m: (
  /* c */
  b: (
    x: 2,
  ),
);

/* prettier */
$m: (
  /* c */
  b: (
      x: 2,
    ),
);
```

A comment-preceded block map value prints at the normal nested-map indent;
Prettier double-indents it (+6 body / +4 `)`) because its dedent applies only when the pair doc is a plain `group(indent(fill))` and a leading comment changes the doc shape.
Comment presence must not change layout: same dedent-skip artifact class as `nested-map-context-indent`
(paren-block KEYS still keep the pair indent, matching Prettier: that trigger is content, not trivia).

## map-item-break-comma-lists-only

- Why: semantics
- Pin: `tests/fixtures/format/scss/map-item-parens.scss`

```scss
/* input */
@include container($foo: ($bar + $baz));

/* ours */
@include container($foo: ($bar + $baz));

/* prettier */
@include container(
  $foo: (
    $bar + $baz,
  )
);
```

The map-item break (one element per line + trailing comma) applies ONLY to parens whose contents are already a comma-separated list:
`(x,)` is a single-element list in Sass, so the added comma is a semantic no-op for a comma list and NOWHERE else.
Prettier 3.9.6 changes `key: ($a + $b)` from a number to a list, restructures `key: (a b)` (2-element space list → nested 1-element list), and emits non-compiling output for `key: 2 * ($a + $b)` inside `$var:` declarations (dart-sass: `Undefined operation "2 * (3px,)"`).
Prettier's own prettier/prettier#18530 / prettier/prettier#19091 fixed subsets of this; we extend the same rule to every non-comma-list.

## single-item-list-trailing-comma

- Why: semantics (prettier/prettier#19928)
- Pin: `tests/fixtures/format/scss/single-item-list-trailing-comma.scss`
- Drop when: upstream also keeps the paren-less form and `(fn(1),)` (dropped as a source-span artifact of its value parser)

```scss
/* input */
$list: ("a",);
$bare: "a",;
$map: (k: ("a",));

/* ours */
$list: ("a",);
$bare: "a",;
$map: (
  k: ("a",),
);

/* prettier */
$list: ("a");
$bare: "a";
$map: (
  k: ("a"),
);
```

`("a",)` and `$x: "a",` are one-element lists in Sass while `"a"` is a string (dart-sass `type-of`):
dropping the comma changes the value for `nth()` / `@each` / `list.append()`.
Prettier 3.9.6 drops it everywhere but `var()`; we keep it whenever the list has exactly one element
(multi-element lists and function arguments still drop theirs, the list is a list without it).

## trailing-comma-none-before-tail-comment

- Why: uniform-rule (option governs: trailingComma)
- Pin: `tests/fixtures/format/scss/trailing-comma-none/single-item-list.scss`

```scss
/* input, trailingComma: none */
$map: (a: 1, b: 2,
  // own
);

/* ours */
$map: (
  a: 1,
  b: 2
  // own
);

/* prettier */
$map: (
  a: 1,
  b: 2,
  // own
);
```

A trailing comma before an own-line comment is the same no-op as any other trailing comma, so `trailingComma: none` drops it.

Prettier's postcss printer prints the comment as a list member, so a source comma before it becomes the separator and survives the option
(`scss/map/15193.scss` pins that shape; the issue itself was about the comma landing INSIDE the comment).
Its JS printer follows the option (`[1, 2\n // own]`), and so do all our formatter crates.

## own-line-trailing-comment-keeps-line

- Why: invariant
- Pin: `tests/fixtures/format/scss/module-config-comments.scss`, `tests/fixtures/format/scss/paren-tail-own-line-comment.scss`

```scss
/* input */
@use "a" with (
  $e: 5
  // before close
);

/* ours */
@use "a" with (
  $e: 5
  // before close
);

/* prettier */
@use "a" with (
  $e: 5 // before close
);
```

An own-line trailing comment before a closing `)` keeps its own line:
in maps and `@use`/`@forward with (...)` configs (any comment kind),
in map-item lists (any kind, the body is already one item per line),
and in paren lists, call/`@include` arguments and `@mixin` parameters
(`//` only: an own-line block comment in a fill body is a fill item, see AGENTS.md).

Prettier's output changes line, own-line to the last item's line, a `lineSuffix` artifact of its comma-group printing.
Same-line trailing comments still glue (matching Prettier); moving an own-line comment up would destroy the author's visual grouping.

## map-leading-comment-layout

- Why: uniform-rule (comment presence never changes layout)
- Pin: `tests/fixtures/format/scss/map-comment-only.scss`

```scss
/* input */
$b: (/* c */ a: 1,);

/* ours */
$b: (
  /* c */ a: 1,
);

/* prettier */
$b: (/* c */ a: 1);
```

A map whose FIRST item is preceded by a block comment loses map-item-ness in Prettier:
it inlines when it fits and drops the trailing comma.
We print it as the same map without the comment (`$b: (a: 1,)`): one item per line, comma per `trailingComma`.
A comment before a LATER item keeps both in Prettier too.

## comment-only-map-indent

- Why: uniform-rule (same construct, same output: other `//` lines in the map)
- Pin: `tests/fixtures/format/scss/map-comment-only.scss`

```scss
/* input */
$multi: (
  // a
  // b
);

/* ours */
$multi: (
  // a
  // b
);

/* prettier */
$multi: (
  // a
   // b
);
```

Consecutive `//` comments in a comment-only map indent uniformly;
Prettier misaligns the second with a stray extra leading space, an artifact of its `join(line)` separator printing before the deferred `lineSuffix` flushes.
A meaningless glitch.

## semiless-custom-property-block

- Why: cost
- Pin: `tests/fixtures/format/scss/custom-property-semiless-block.scss` (also tracked by conformance `scss/variables/apply-rule.scss`)

```scss
/* input */
:root {
  --like-a-apply-rule: {
  color:red;} /* no semi here*/
  --another-prop: blue;
}

/* ours (SCSS mode) */
:root {
  --like-a-apply-rule: {
    color: red;
  }; /* no semi here*/
  --another-prop: blue;
}

/* prettier */
:root {
  --like-a-apply-rule: {
  color:red;} /* no semi here*/
  --another-prop: blue;
}
```

SCSS only: a `;`-less custom-property rule block followed by another declaration is two separate declarations for us
(format the inner block, add the missing `;`, format the rest normally);
Prettier keeps the whole run verbatim (postcss swallows everything past the `}` as an opaque prelude string until a source `;`).
This falls out of the AST shape: SCSS parses `{...}` declaration values as `SassNestingDeclaration`;
CSS/Less don't structure them, so their token-soup fallback incidentally matches Prettier.
The value syntax's only intended consumer was the dropped CSS Apply Rule proposal, so the cross-mode difference is theoretical.

## less-extend-statement-break

- Why: uniform-rule (same construct, same output: selector-position `&:extend(...)`; prettier/prettier#19550)
- Pin: `tests/fixtures/format/less/extend-rule.less`
- Drop when: the selector-list leak is fixed (prettier/prettier#19550 covered only the indentation) and the pin catches up

```less
/* input */
&:extend(.a, .b);

/* ours */
&:extend(.a, .b);

/* prettier */
&:extend(
  .a,
  .b
);
```

Statement-position `&:extend(...)` breaks only on overflow, like the selector-position form
(inline when it fits; parens on their own lines + one selector per line on overflow, the same shape as Prettier's break).
Prettier (3.9.5+) ALWAYS breaks multiple selectors there and never breaks a single one:
postcss-less models the statement as a rule node, so the top-level selector-list printer (hardline commas) leaks into the parens.

## unary-plus-glue

- Why: cost
- Pin: `tests/fixtures/format/css/token-soup-math-glue.css` (Css mode),
  `tests/fixtures/format/less/signed-value-args.less` (Less mode);
  also tracked by conformance `css/parens/parens.css`

```css
/* input */
prop34: func(+20px, + 20px);

/* ours */
prop34: func(+20px, + 20px);

/* prettier */
prop34: func(+20px, +20px);
```

Mode-independent (measured in Css and Less alike): a unary sign the source SPACES from its number stays spaced, ours preserves the source spacing per token;
Prettier glues the sign to the number (removing the source space, a postcss word-lexing behavior).
Matching that gluing is ad-hoc work for a torture-test-only shape.
A sign GLUED in the source never gains a space in either implementation (that direction is the parser's folded-sign handling, not a divergence).

## css-glued-minus-paren

- Why: uniform-rule (same construct, same output: the `-(` shapes Prettier keeps glued)
- Pin: `tests/fixtures/format/css/token-soup-math-glue.css` (also tracked by conformance `css/parens/parens.css`)

```css
/* input */
prop: 3px -(4px);
prop44: -(4px);

/* ours */
prop: 3px -(4px);
prop44: -(4px);

/* prettier */
prop: 3px - (4px);
prop44: -(4px);
```

Prettier splits only SOME source-glued `-(` (an operator-heuristic side effect: the split needs a binary-looking left neighbor),
an internal inconsistency for the same token pair.
Ours keeps them all glued under the one token-soup rule: never add a space the source doesn't have.
Css mode only (hence the prefix): in Less and Scss, Prettier keeps `3px -(4px)` glued too (measured), so the same shape there is not a divergence.

## fill-break-position

- Why: cost
- Pin: `tests/fixtures/format/css/fill-math-chunk-break.css` (also tracked by conformance `css/fill-value/fill.css` and oxfmt's externals suite)

```css
/* input (nested one level, print width 80) */
margin-left: sg-layout-width(logo-shopify) / 2 * -1 + sg-offset-x(page-nav) / 2;

/* ours */
margin-left: sg-layout-width(logo-shopify) / 2 * -1 + sg-offset-x(page-nav)
  / 2;

/* prettier */
margin-left: sg-layout-width(logo-shopify) / 2 * -1 +
  sg-offset-x(page-nav) / 2;
```

An over-wide math-y value run (css token soup here; nested Less math in the externals suite):
Prettier's fill fit-check breaks INSIDE the wide chunk;
our core `fill` (biome semantics) breaks the SEPARATOR instead.
Layout-only, the principled fix is the shared core-fill fit-check change (needs a JS-conformance impact experiment first).

## less-value-interpolation-rejected

- Why: uniform-rule (error semantics)
- Pin: `tests/fixtures/mod.rs::parse_error_is_err` (parser-level: the rejection is asserted directly)

```less
/* input */
.a { width: @{min-width}; }

/* ours: parse error, the input is left as-is */

/* prettier */
.a {
  width: @{min-width};
}
```

Value-position `@{var}` interpolation: `oxc-css-parser` rejects it matching `lessc`;
Prettier (postcss) accepts and prints verbatim. Per the policy's "Error semantics",
we never format what the reference compiler rejects (a parse error is the SAFE failure).

## media-query-operator-spacing

- Why: uniform-rule (same construct, same output: the same expression in a declaration value; prettier/prettier#1811)
- Pin: `tests/fixtures/format/scss/media-query-operator-spacing.scss` (also tracked by oxfmt's externals suite, e.g. gitlab `framework/diffs.scss`)

```scss
/* input */
@media (max-width: map-get($grid-breakpoints, sm)-1) { }

/* ours */
@media (max-width: map-get($grid-breakpoints, sm) - 1) {
}

/* prettier */
@media (max-width: map-get($grid-breakpoints, sm)-1) {
}
```

SCSS arithmetic in a media prelude prints through the structured `SassExpr` printer with spaced operators,
identical to the same expression in a declaration value; Prettier's postcss-media-query-parser never value-parses the prelude,
so it cannot space ANY arithmetic op there (`+` and `-` alike).
For the glued `-` Prettier also diverges in value positions, via a different artifact: see `value-glued-minus`.

## value-glued-minus

- Why: uniform-rule (same construct, same output: glued `+`)
- Pin: `tests/fixtures/format/scss/value-glued-minus.scss` (Scss),
  `tests/fixtures/format/less/signed-value-args.less` (Less)

```scss
/* input */
.a {
  width: map-get($grid-breakpoints, sm)-1;
}

/* ours */
.a {
  width: map-get($grid-breakpoints, sm) - 1;
}

/* prettier */
.a {
  width: map-get($grid-breakpoints, sm)-1;
}
```

A glued `-` after a call/paren in a value position is subtraction (dart-sass and lessc alike);
ours parses it as a binary operation (`SassBinaryExpression` / `LessBinaryOperation`) and prints spaced, like every other operator.
Prettier's value lexer keeps the glued `-` verbatim
(a sign-lexing artifact: `-` may start a number/ident, so no operator node forms) while the same glued `+` DOES get spaced;
we print both ops uniformly.

The same artifact pair exists in Less:

```less
/* input */
p7: fade(@c, 4%)-1;
p8: fade(@c, 4%)+1;

/* ours */
p7: fade(@c, 4%) - 1;
p8: fade(@c, 4%) + 1;

/* prettier */
p7: fade(@c, 4%)-1;
p8: fade(@c, 4%) + 1;
```

Scope: value positions where ours structures the glued sign as a binary operation.

- In Scss, any such position (property values, `$var:` declarations, function args)
- In Less, a call left operand only

Everywhere else BOTH implementations keep the glue, so there is nothing to diverge;
where exactly ours keeps it is printer mechanism, documented at `write_less_binary_operation` / `write_sass_binary`
(and pinned by the same fixtures plus `tests/fixtures/format/scss/calc-glued-minus.scss` for Scss calculation args).

## map-paren-value-blank-lines

- Why: uniform-rule (blank-line preservation; prettier/prettier#16824)
- Pin: `tests/fixtures/format/scss/map-paren-value-blank-lines.scss` (also tracked by oxfmt's externals suite, gitlab `highlight/conflict_colors.scss`)

```scss
/* input */
$colors: (
  a: rgba(#fff, 0.1),

  b: #111,
);

/* ours */
$colors: (
  a: rgba(#fff, 0.1),

  b: #111,
);

/* prettier */
$colors: (
  a: rgba(#fff, 0.1),
  b: #111,
);
```

A blank line after a map item is preserved per the blank-line preservation rule regardless of the value's shape,
as after a map item with a non-paren value (Prettier itself keeps that one);
Prettier drops it when the item's value is paren-ish (a call / paren group), an artifact of its comma-group splitting.
