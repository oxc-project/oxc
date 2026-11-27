# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## trailing-line-comment-print-width

- Why: uniform-rule (line_suffix is never measured)
- Pin: `tests/fixtures/format/less/trailing-inline-comment-width.less`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-config/globalVars/extended.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/color-functions/rgba.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/extend-selector/extend-selector.less`
  - `externals/ng-zorro-antd/components/style/themes/default.less`
  - `externals/ng-zorro-antd/components/style/themes/variable.less`
  - `externals/ng-zorro-antd/components/style/themes/dark.less`
  - `externals/gitlab/stylesheets/pages/profile.scss`

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

## commented-as-uncommented

- Why: uniform-rule (comment presence never changes layout)
- Pin: `tests/fixtures/format/css/keyframe-selector-comment.css`, `tests/fixtures/format/css/keyframes-name-comment.css`, `tests/fixtures/format/css/media-commented-feature.css`
- Conformance:
  - `externals/prettier/css/comments/at-rules.css`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/comments/comments2.less`

```css
/* input */
@keyframes x { 60%   /* mid */  ,   70% { opacity: 1; } }
@keyframes /* c */  foo {}
@media all and/* c */(max-width:1024px) {}

/* ours */
@keyframes x {
  60% /* mid */,
  70% {
    opacity: 1;
  }
}
@keyframes /* c */ foo {
}
@media all and /* c */ (max-width: 1024px) {
}

/* prettier */
@keyframes x {
  60%   /* mid */  ,   70% {
    opacity: 1;
  }
}
@keyframes /* c */  foo {
}
@media all and/* c */(max-width:1024px) {
}
```

A keyframe selector list, a `@keyframes` name and a `@media` query print the same with or without a comment in them, each comment kept between the tokens it sits between,
a `@keyframes` string re-quoted as without one (DIVERGENCES.md#prelude-string-requote).
Prettier keeps each of them as written once it holds a comment, source spacing and quotes included, and normalizes the uncommented ones.
Same rule as DIVERGENCES.md#comment-preceded-map-indent.

## layer-comma-list

- Why: uniform-rule (same construct, same output: a comma list in `@media` / `@custom-selector`)
- Pin: `tests/fixtures/format/css/layer-comma-list.css`

```css
/* input */
@layer a,b;
@layer reset, base,
  components;
@layer /* c */ a ,  b;

/* ours */
@layer a, b;
@layer reset, base, components;
@layer /* c */ a, b;

/* prettier */
@layer a,b;
@layer reset, base,
  components;
@layer /* c */ a ,  b;
```

A `@layer` name list prints `, `-separated on one line, with or without a comment in it,
as Prettier prints a `@media` query list or a `@custom-selector` list (`screen,print` -> `screen, print`).
Prettier has no `@layer` case and keeps its params as written.
A `//` still breaks the list (`tests/fixtures/format/scss/at-rule-prelude-comments.scss`).

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
@media (foo bar:baz , qux) { }

/* ours */
@media (not (screen and (color))) {
}
@media (foo bar:baz, qux) {
}

/* prettier */
@media (not ( screen and ( color ) )) {
}
@media (foo bar: baz , qux) {
}
```

`<general-enclosed>` preludes (unparsable as `<media-condition>`) normalize whitespace fully: source gap → one space, paren inner edges tight.
So does a `@supports` one, and a `selector()` whose argument is not a selector (`selector(A < B)`).
Prettier only collapses space RUNS inside the unparsable paren;
reproducing that half-normalization is pure tokenizer-artifact matching.
Gap-based spacing never splits or fuses tokens: `and (` can't become a function token `and(`,
and no space is added where the source has none, so a pseudo-class keeps its `:` (`selector(:before)`), unlike Prettier's `: ` after any colon.

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

## placeholder-property-name

- Why: uniform-rule (same construct, same output: `${a}-${b}: 1rem`)
- Pin: `tests/fixtures/embedded/scss/placeholder-property-name.scss`

```scss
/* input (css-in-js: ${a} is a template placeholder) */
x-${a}:    1rem;

/* ours */
x-${a}: 1rem;

/* prettier */
x-${a}:    1rem;
```

A property name with a placeholder formats the same wherever the placeholder sits, and its case is kept.
Prettier formats the template only when a placeholder leads the name (postcss reads it as an at-rule);
otherwise it fails and the whole template stays verbatim.

## placeholder-important-space

- Why: uniform-rule (same construct, same output: `red !important`)
- Pin: `tests/fixtures/embedded/scss/value-placeholders.scss`

```scss
/* input (css-in-js: ${m} is a template placeholder) */
color: ${m}!important;

/* ours */
color: ${m} !important;

/* prettier */
color: ${m}!important;
```

A `!important` after a placeholder is spaced as after any other value, valid whatever the placeholder expands to;
Prettier keeps it glued because it reads the placeholder text and the flag as one word.

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
Prettier keeps the spaces inside SELECTOR interpolation, only collapsing each run to one as it does across the selector (`#{   $name   }` -> `#{ $name }`).

## prelude-string-requote

- Why: uniform-rule (option governs: singleQuote)
- Pin: `tests/fixtures/format/scss/unknown-at-rule-edges.scss`, `tests/fixtures/format/css/prelude-string-requote.css`
- Conformance:
  - `externals/mdn-css-guides/values_and_units/textual_data_types/index.md`

```scss
/* input */
@error 'single quotes get normalized';
@keyframes 'validString' {}

/* ours */
@error "single quotes get normalized";
@keyframes "validString" {
}

/* prettier */
@error 'single quotes get normalized';
@keyframes 'validString' {
}
```

A string in an at-rule prelude we parse typed re-quotes per the `singleQuote` option:
the `@warn` / `@error` prelude (a `SassExpr`) and a `@keyframes` name (`at_rule.rs` `write_keyframes_name`) go through the structured printer;
Prettier keeps the at-rule params as a raw string verbatim.
Every other string in a declaration value re-quotes per the same option in both formatters.

## less-escape-hugs-string

- Why: uniform-rule (same construct, same output: a declaration value)
- Pin: `tests/fixtures/format/less/escaped-string.less`

```less
/* input */
a { value: ~ 'x'; }
@media (min-width: ~ '768px') {}
[class^=~ 'col-'] {}
@keyframes ~ 'name' {}

/* ours */
a {
  value: ~"x";
}
@media (min-width: ~"768px") {
}
[class^=~"col-"] {
}
@keyframes ~"name" {
}

/* prettier */
a {
  value: ~"x";
}
@media (min-width: ~ "768px") {
}
[class^=~ "col-"] {
}
@keyframes ~ 'name' {
}
```

A Less escape (`LessEscapedStr`) prints as `~` hugging its re-quoted string wherever it is typed (`value.rs` `write_less_escaped_str`);
Prettier does so in a declaration value and `url()`, but keeps the space in a media feature and an attribute value, and the `@keyframes` name verbatim (see DIVERGENCES.md#prelude-string-requote).
The Prettier-compat raw paths keep the source as-is: a selector with a comment, a commented `@page` / `@font-feature-values` / `@counter-style` / `@property` prelude, and Less mixin call / definition arguments and `when` guards.

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
Same rule as DIVERGENCES.md#comment-preceded-map-indent.

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
- Pin: `tests/fixtures/format/scss/variable-inline-comment.scss`

```scss
/* input */
$m: // c
  (k: v);

/* ours */
$m: // c
  (
    k: v,
  );

/* prettier */
$m: // c
  (
  k: v,
);
```

A map value after a trailing comment on the `$var:` line keeps its shape under the continuation indent;
Prettier dedents it, the body level with the `(` and the `)` at the declaration's column,
because its dedent applies only when the pair doc is a plain `group(indent(fill))` and the comment changes the doc shape.
Same dedent-skip artifact class as DIVERGENCES.md#nested-map-context-indent.

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
Prettier changes `key: ($a + $b)` from a number to a list, restructures `key: (a b)` (2-element space list → nested 1-element list), and emits non-compiling output for `key: 2 * ($a + $b)` inside `$var:` declarations (dart-sass: `Undefined operation "2 * (3px,)"`).
Prettier's own prettier/prettier#18530 / prettier/prettier#19091 fixed subsets of this; we extend the same rule to every non-comma-list.

## single-item-list-trailing-comma

- Why: semantics (prettier/prettier#19928)
- Pin: `tests/fixtures/format/scss/single-item-list-trailing-comma.scss`
- Drop when: upstream also keeps the paren-less form

```scss
/* input */
$bare: "a",;

/* ours */
$bare: "a",;

/* prettier */
$bare: "a";
```

`("a",)` and `$x: "a",` are one-element lists in Sass while `"a"` is a string (dart-sass `type-of`):
dropping the comma changes the value for `nth()` / `@each` / `list.append()`.
Prettier keeps it in parens since #19928 and still drops it from the paren-less form (`$x: "a",`, `@return "a",`);
we keep it whenever the list has exactly one element
(multi-element lists and function arguments still drop theirs, the list is a list without it).

## bracket-list-layout

- Why: uniform-rule (same construct, same output: paren comma list `(1, 2)`)
- Pin: `tests/fixtures/format/scss/bracket-list.scss`

```scss
/* input */
$a: [1, /* c */ 2];
$b: [1, 2] 3;

/* ours */
$a: [1, /* c */ 2];
$b: [1, 2] 3;

/* prettier */
$a:
  [1,
  /* c */ 2];
$b:
  [1,
  2] 3;
```

A bracketed Sass list is a list like `(1, 2)`; we print it with the paren list's layout
(fits on one line or breaks one element per line, comments stay inside the brackets).
Prettier's value parser has no bracket node: `[1,` and `2]` are words of the OUTER comma list,
so a comment, a sibling token or an overflow hard-breaks that outer list around the brackets
(and `[1, 2, ]` keeps a trailing comma with a space).
`[1,]` keeps its comma for the same reason as `(1,)` (see DIVERGENCES.md#single-item-list-trailing-comma).

## terminator-gap-normalized

- Why: uniform-rule (the formatter owns the trivia up to a terminator)
- Pin: `tests/fixtures/format/scss/terminator-gap.scss`
- Conformance:
  - `externals/prettier/css/stylefmt-repo/at-media/at-media.css`
  - `externals/prettier/css/stylefmt-repo/cssnext-example/cssnext-example.css`
  - `externals/prettier/css/stylefmt-repo/media-queries-ranges/media-queries-ranges.css`
  - `externals/prettier/css/postcss-plugins/postcss-nesting.css`
  - `externals/prettier/css/comments/declaration.css`
  - `externals/prettier/scss/comments/4878.scss`
  - `externals/prettier/scss/map/function-argument/functional-argument.scss`
  - `externals/prettier/scss/parens/issue-16594.scss`
  - `externals/prettier/scss/trailing-comma/comments.scss`
  - `externals/prettier/scss/trailing-comma/list.scss`
  - `externals/prettier/scss/trailing-comma/variable.scss`
  - `externals/prettier/scss/function/arbitrary-arguments-comment.scss`
  - `externals/prettier/scss/map/15193.scss`
  - `externals/prettier/scss/comments/variable-declaration.scss`

```scss
/* input */
a { color: red !important /* c */ ; }

/* ours */
a {
  color: red !important /* c */;
}

/* prettier */
a {
  color: red !important /* c */ ;
}
```

A comment before the `;` keeps its place; the gap between it and the `;` is the formatter's (FORMATTER_POLICY.md "Comment placement invariants": terminators and the trivia up to them),
so it is dropped as after any other value (`color: red /* c */ ;` -> `red /* c */;`, matching Prettier).
Prettier keeps the source gap only after `!important` / `!default` / `!global`: postcss stores the text after the flag verbatim in `raws.important`.

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
- Conformance:
  - `externals/prettier/scss/trailing-comma/issue-6356.scss`

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

An own-line trailing comment before a closing `)` keeps its own line
(so does an own-line comment before a list comma: it leads the next element, `a,\n  // c\n  b`, pinned in `line-comment-before-comma.scss`):
- maps
- `@use`/`@forward with (...)` configs (any comment kind)
- map-item lists (any kind, the body is already one item per line)
- paren lists
- call/`@include` arguments
- `@mixin` parameters

`//` only: an own-line block comment in a fill body is a fill item, see AGENTS.md.

Prettier's output changes line, own-line to the last item's line, a `lineSuffix` artifact of its comma-group printing.
Same-line trailing comments still glue (matching Prettier); moving an own-line comment up would destroy the author's visual grouping.

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

## supports-selector-inline-comment-list

- Why: uniform-rule (same construct, same output: `selector(a // c\n)`)
- Pin: `tests/fixtures/format/scss/supports-selector-inline-comments.scss`

```scss
/* input */
@supports selector(a, // c
 b) {}
@supports selector(a // c
 // d
) {}

/* ours */
@supports selector(
  a, // c
  b
) {
}
@supports selector(
  a // c
  // d
) {
}

/* prettier */
@supports selector(a, // c
 b) {
}
@supports selector(
  a // c
 // d
) {
}
```

A `//` inside `selector()` opens the parens and ends its line, the shape Prettier itself prints for a single selector (`selector(a // c\n)`);
a comma list holding a `//` it keeps verbatim instead, source whitespace included.
A second `//` indents like the first; Prettier prints it with a stray leading space and no indent,
the same `join(line)` + deferred `lineSuffix` artifact as DIVERGENCES.md#comment-only-map-indent.

## line-comment-after-comma

- Why: invariant
- Pin: `tests/fixtures/format/scss/line-comment-after-comma.scss`, `tests/fixtures/format/less/line-comment-after-comma.less`
- Conformance:
  - `externals/prettier/scss/comments/4594.scss`
  - `externals/prettier/scss/comments/lists.scss`
  - `externals/prettier/scss/comments/maps.scss`
  - `externals/prettier/scss/trailing-comma/issue-6920.scss`
  - `externals/prettier/scss/trailing-comma/comments.scss`
  - `externals/prettier/scss/map/empty-line-after-function.scss`
  - `externals/prettier/less/comments/value-lists.less`

```scss
/* input */
$my-map: (
  "foo": 1, // Comment
  "bar": 2, // Comment
);

/* ours */
$my-map: (
  "foo": 1, // Comment
  "bar": 2, // Comment
);

/* prettier */
$my-map: (
  "foo": 1,
  // Comment
  "bar": 2, // Comment
);
```

A `//` on a comma's line stays on that line (the block comments glued before it come along);
Prettier moves it below as the next element's leading comment, across the line boundary (`a, // stylelint-disable-line` loses its target).
Prettier keeps the same comment after the LAST comma (`"bar": 2, // Comment`) and its JS printer keeps `1, // c` everywhere:
the move is where postcss-value-parser hands the comment to the next comma group, not a rule.
Applies at every comma site: values, function and `@include` arguments, maps, paren lists, `@use ... with`, `@forward` members, `@each`,
`@import` paths and modifiers, `@layer`, `@custom-selector`, `@media` query lists and `selector()` lists.
For `@media` Prettier's move also swallows the next query (`@media a, // c b {`), a semantics bug on its side.

## line-comment-before-block

- Why: invariant
- Pin: `tests/fixtures/format/scss/at-rule-comment-before-block.scss`

```scss
/* input */
@supports (a: b) // c
{
  color: red;
}

/* ours */
@supports (a: b) // c
{
  color: red;
}

/* prettier */
@supports (a: b) { // c
  color: red;
}
```

A `//` glued to the end of an at-rule prelude stays on that line and the `{` starts the next;
Prettier moves it past the `{` as the block's first comment for `@media` / `@supports` / `@mixin` / `@include`
(and keeps it before the `{` for `@page` / `@keyframes` / `@font-face` / `@if`):
the comment crosses the `{`, a grammar-fixed delimiter.
For `@media screen // c {` and `@else // c` Prettier's move also swallows the `{` / the comment text, bugs on its side.

## own-line-comment-before-block

- Why: invariant
- Pin: `tests/fixtures/format/scss/at-rule-comment-before-block.scss`
- Conformance:
  - `externals/prettier/css/comments/at-rules.css`

```css
/* input */
@media screen
  /* c */{
}

/* ours */
@media screen
/* c */ {
}

/* prettier */
@media screen /* c */ {
}
```

A comment starting a line before an at-rule's `{` keeps its line, the `{` on it or on the next as in the source, as before a JS block (`if (a)\n/* c */ {`);
Prettier moves it onto the prelude's line: the comment changes line.

## at-root-line-comment-indent

- Why: uniform-rule (same construct, same output: an `@include` prelude after a `//`)
- Pin: `tests/fixtures/format/scss/at-rule-prelude-comments.scss`
- Conformance:
  - `externals/prettier/scss/comments/at-rule.scss`

```scss
/* input */
@include // c
foo;
@at-root // c
.foo {}

/* ours */
@include // c
  foo;
@at-root // c
  .foo {
}

/* prettier */
@include // c
  foo;
@at-root // c
.foo {
}
```

A `//` before an at-rule prelude breaks it, and the prelude continues one level in, as Prettier prints `@include`;
Prettier keeps an `@at-root` selector after a `//` as written instead.

## line-comment-before-comma-fill-head

- Why: uniform-rule (same construct, same output: `@each $k in a, // c` with the `//` after the comma)
- Pin: `tests/fixtures/format/scss/line-comment-before-comma.scss`

```scss
/* input */
@each $k in a // c
  , b {
}

/* ours */
@each $k in a, // c
  b
{
}

/* prettier */
@each $k
    in a, // c
  b
{
}
```

A `//` before a list comma rides past the comma and ends the line there; the fill entries before it stay on their line,
exactly as when the `//` follows the comma in the source.
Prettier attaches a `breakParent` to the deferred comment, which its fill measures as never fitting,
so the entry BEFORE the comment (`in a,`) breaks away from `$k` too; the same source with the `//` after the comma keeps `$k in a,`.

## less-variable-value-comments

- Why: invariant
- Pin: `tests/fixtures/format/less/variable-value-comments.less`, `tests/fixtures/format/less/important-comments.less`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/comments/comments2.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/property-name-interp/property-name-interp.less`

```less
/* input */
@a: /* c */ 1;
@c: 1 /* c */;

/* ours */
@a: /* c */ 1;
@c: 1 /* c */;

/* prettier */
@a: 1;
@c: 1/* c */ ;
```

A comment inside a variable's value stays in place, as in a declaration (`color: /* c */ red`).
Prettier loses it (the lossless contract): postcss-less parses `@a: 1` as an at-rule with the value as a params string,
and its at-rule printer drops the block comments in there (a trailing one survives glued to the value, in front of the `;` gap).

## less-extend-statement-break

- Why: uniform-rule (same construct, same output: selector-position `&:extend(...)`; prettier/prettier#19550)
- Pin: `tests/fixtures/format/less/extend-rule.less`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/extend-chaining/extend-chaining.less`
  - `externals/prettier/less/less/less.less` (the `extend #7977` rules)
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
Prettier ALWAYS breaks multiple selectors there and never breaks a single one:
postcss-less models the statement as a rule node, so the top-level selector-list printer (hardline commas) leaks into the parens.

## unary-plus-glue

- Why: cost
- Pin: `tests/fixtures/format/css/token-soup-math-glue.css` (Css mode),
  `tests/fixtures/format/less/signed-value-args.less` (Less mode)
- Conformance:
  - `externals/prettier/css/parens/parens.css` (`prop34`; the file also has unclassified diffs)

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

## less-keyword-minus

- Why: semantics
- Pin: `tests/fixtures/format/less/keyword-minus.less`

```less
/* input */
v: -  1;

/* ours */
v: - 1;

/* prettier */
v: -1;
```

less.js negates only a glued `@var`, `$prop` or `(`, so a lone `-` before a number is a keyword, not a sign.
lessc compiles `- 1` as written and `-1` as the number, so Prettier's glue changes the output.

## scss-at-rule-name-case

- Why: semantics
- Pin: `tests/fixtures/format/scss/at-rule-name-case.scss`
- Conformance:
  - `externals/prettier/scss/case/case.scss` (the at-rule name lines only; the interpolated property name line has no entry)

```scss
/* input */
@IMPORT Keep;
@USE "x";
@MEDIA (MIN-WIDTH: 700PX) {}

/* ours */
@IMPORT Keep;
@USE "x";
@MEDIA (MIN-WIDTH: 700PX) {}

/* prettier */
@import Keep;
@use "x";
@media (min-width: 700px) {}
```

dart-sass matches at-rule names case-sensitively: `@IMPORT`, `@USE` and `@MEDIA` are unknown at-rules it compiles as written,
while `@import Keep;` fails to compile and `@use "x"` loads a module.
So an SCSS at-rule with an uppercase name prints as written, its params included,
as every unknown at-rule's (AGENTS.md "Printing raw vs typed").

## signed-value-resplit

- Why: uniform-rule (same construct, same output: the item in first position, an unsigned item)
- Pin: `tests/fixtures/format/css/signed-value-resplit.css`
- Conformance:
  - `externals/prettier/css/fill-value/fill.css`
  - `externals/gitlab/stylesheets/framework/variables.scss`
  - `externals/ng-zorro-antd/components/table/style/index.less`

```css
/* input */
font-family: Arial, -apple-system;
-webkit-mask-position-x: 50px, 25%, -3em;
background: url("/shared-assets/images/examples/web-animations/cat_sprite.png") -600px 0 no-repeat;

/* ours */
font-family: Arial, -apple-system;
-webkit-mask-position-x: 50px, 25%, -3em;
background: url("/shared-assets/images/examples/web-animations/cat_sprite.png")
  -600px 0 no-repeat;

/* prettier */
font-family:
  Arial,
  -apple-system;
-webkit-mask-position-x:
  50px,
  25%,
  -3em;
background: url("/shared-assets/images/examples/web-animations/cat_sprite.png") -600px
  0 no-repeat;
```

postcss-values splits a signed word (`-apple-system`, `-3em`, `-@var`) past the first position into an operator + word,
the re-split word AGENTS.md's "Acceptance" does not follow:
- as a two-node item, it breaks the comma list one item per line (`shouldBreakList`), though `-apple-system, Arial` and `50px, 25%, 3em` stay on one line
- as a math operator, it glues to the word before it, overflowing the line

Ours prints a signed item as one word, like the same item in first position or unsigned.

## css-glued-minus-paren

- Why: uniform-rule (same construct, same output: the `-(` shapes Prettier keeps glued)
- Pin: `tests/fixtures/format/css/token-soup-math-glue.css`
- Conformance:
  - `externals/prettier/css/parens/parens.css` (`prop`/`prop44`; the file also has unclassified diffs)

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
- Pin: `tests/fixtures/format/scss/fill-break-position.scss`
- Conformance:
  - `externals/webawesome/number-input/number-input.styles.ts`
  - `externals/webawesome/page/page.styles.ts`
  - `externals/ng-zorro-antd/components/style/themes/default.less`
  - `externals/ng-zorro-antd/components/style/themes/variable.less`
  - `externals/ng-zorro-antd/components/table/style/rtl.less`
  - `externals/gitlab/stylesheets/components/content_editor.scss`
  - `externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss`
  - `externals/gitlab/stylesheets/framework/sidebar.scss`

```scss
/* input (nested three levels, print width 80) */
height: calc(#{$calc-application-viewport-height} - #{$mr-sticky-header-height} - var(--mr-review-bar-height));

/* ours */
height: calc(
  #{$calc-application-viewport-height} - #{$mr-sticky-header-height} -
    var(--mr-review-bar-height)
);

/* prettier */
height: calc(
  #{$calc-application-viewport-height} -
    #{$mr-sticky-header-height} - var(--mr-review-bar-height)
);
```

A math-y value run (token soup here):
Prettier's fill breaks at an earlier separator, though the next chunk still fits;
our core `fill` (biome semantics) breaks only at the last fitting separator.
Layout-only, the principled fix is the shared core-fill fit-check change (needs a JS-conformance impact experiment first).
A spaced `/` is not part of this: it glues to its left operand and breaks after, as in Prettier (`tests/fixtures/format/css/fill-math-chunk-break.css`).

## grammar-owner-rejected

- Why: uniform-rule (acceptance: the grammar owner decides)
- Pin: `tests/fixtures/mod.rs::parse_error_is_err` (parser-level: the rejection is asserted directly)
- Conformance:
  - `externals/prettier/css/atrule/charset.css`
  - `externals/prettier/css/atrule/import.css`
  - `externals/prettier/css/attribute/quotes.css`
  - `externals/prettier/css/combinator/combinator.css`
  - `externals/prettier/css/comments/selectors.css`
  - `externals/prettier/css/custom-properties/emoji.css`
  - `externals/prettier/css/no-semicolon/url.css`
  - `externals/prettier/css/postcss-plugins/postcss-mixins.css`
  - `externals/prettier/css/postcss-plugins/postcss-simple-vars.css`
  - `externals/prettier/css/quotes/quotes.css`
  - `externals/prettier/css/selector-list/selectors.css`
  - `externals/prettier/css/selector-string/string.css`
  - `externals/prettier/scss/atrule/for.scss`
  - `externals/prettier/scss/comments/CRLF.scss`
  - `externals/prettier/scss/math/3945.scss`
  - `externals/prettier/scss/no-semicolon/url.scss`
  - `externals/prettier/scss/parens/parens.scss`
  - `externals/prettier/less/case/case.less`
  - `externals/prettier/less/comments/in-value.less`
  - `externals/prettier/less/interpolation/interpolation.less`
  - `externals/prettier/less/lookup/lookup-1.less`
  - `externals/prettier/less/no-semicolon/url.less`
  - `externals/prettier/less/parens/parens.less`

```less
/* input */
.a { width: @{min-width}; }

/* ours: parse error, the input is left as-is */

/* prettier */
.a {
  width: @{min-width};
}
```

`oxc-css-parser` rejects what the variant's grammar owner rejects (oxc-css-parser AGENTS.md "Acceptance"),
so per the policy's "Error semantics" we leave it as-is (a parse error is the SAFE failure);
Prettier (postcss) accepts these and prints them, mostly verbatim.

- CSS, a browser drops the rule (Chrome; lightningcss, built on `cssparser`):
  - invalid selectors: `a[id=func("foo")]`, `-Option/root .a`, `>>>`, `.is-$(var)`, `:not(".a")`, `one "two" three`
  - invalid `@import` preludes: `@import (multiple) "a";`, `@import "a" {}`
  - root declarations, `--foo: { }` included (a custom property, not the raw-prelude `x: { }` rule)
  - `@charset UTF-8;`: only the exact `@charset "...";` form means anything (css-syntax-3); a browser drops both forms, so this one rests on the spec
- SCSS, dart-sass rejects: `@for $i from 1 end 3`, a Less variable (`@var`) in a value, root declarations
- Less, lessc rejects: value-position `@{var}`, SCSS interpolation (`#{$var}`), `@IMPORT a;`, a spaced lookup (`@c [ a ]`), `++1`
- Every owner rejects `@import ur l(a.css);`

## media-query-operator-spacing

- Why: uniform-rule (same construct, same output: the same expression in a declaration value; prettier/prettier#1811)
- Pin: `tests/fixtures/format/scss/media-query-operator-spacing.scss`
- Conformance:
  - `externals/gitlab/stylesheets/framework/diffs.scss`
  - `externals/gitlab/stylesheets/page_bundles/editor.scss`
  - `externals/gitlab/stylesheets/page_bundles/issuable_list.scss`
  - `externals/gitlab/stylesheets/page_bundles/labels.scss`
  - `externals/gitlab/stylesheets/page_bundles/environments.scss`
  - `externals/gitlab/stylesheets/page_bundles/merge_requests.scss`
  - `externals/gitlab/stylesheets/page_bundles/settings.scss`
  - `externals/gitlab/stylesheets/page_bundles/projects.scss`
  - `externals/gitlab/stylesheets/pages/settings.scss`

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
For the glued `-` Prettier also diverges in value positions, via a different artifact: see DIVERGENCES.md#value-glued-minus.

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

## slash-spacing

- Why: uniform-rule (same construct, same output: a `/` between numbers, glued `1rem/1` or spaced `1rem / 1`; prettier/prettier#10831)
- Pin: `tests/fixtures/format/scss/slash-spacing.scss`, `tests/fixtures/format/less/slash-spacing.less`
- Conformance:
  - `externals/prettier/css/font/font.css`
  - `externals/prettier/css/parens/parens.css`
  - `externals/prettier/less/less-test-suite/snippet: tests-config/namespacing/namespacing-3.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/variables/variables.less`

```scss
/* input */
.a {
  width: f(1px)/2;
  width: 1px/($b);
  font: 12px/ 1.5 serif;
  font: 12px  /1.5 serif;
  width: $a/ 2;
}

/* ours */
.a {
  width: f(1px)/2;
  width: 1px/($b);
  font: 12px / 1.5 serif;
  font: 12px / 1.5 serif;
  width: $a / 2;
}

/* prettier */
.a {
  width: f(1px) / 2;
  width: 1px/ ($b);
  font: 12px/1.5 serif;
  font: 12px /1.5 serif;
  width: $a/ 2;
}
```

```less
/* input */
.a { width: @a/2; }
@media @phone and (aspect-ratio: 16/ 9) {}

/* ours */
.a {
  width: @a/2;
}
@media @phone and (aspect-ratio: 16 / 9) {
}

/* prettier */
.a {
  width: @a / 2;
}
@media @phone and (aspect-ratio: 16/ 9) {
}
```

A `/` outside a math function keeps the source's choice, made symmetric:
glued on both sides stays glued, a gap on either side prints one space on both.
The spacing never changes the meaning: dart-sass and lessc compile `x/y`, `x/ y` and `x / y` alike for every operand kind.
Prettier decides each gap on its own and by the neighbor's kind:

- a word or function neighbor forces spaces (`f(1px)/2`), numbers and parens keep each gap as written (`1rem/ 1` stays lopsided)
- `font` and custom properties tighten a `/` after a font-size value (prettier/prettier#18114, whose review asked to "preserve space")
- a `/` glued after a word is lexed INTO the word by postcss-values-parser 2 (`$a/`, `center/`), so no operator node is left to space and `$a/ 2` stays lopsided

Its own numeric `/` keeping tight or spaced is the rule we apply to every operand.
Scope: declaration values, Sass / Less operations, media query ratios (a substituted raw prelude's too) and Sass control conditions;
a Less mixin call / definition / guard prints its source slice (`adjust_numbers_and_strings`, Prettier does too), and a `@container` prelude prints verbatim.

## math-function-operator-spacing

- Why: uniform-rule (same construct, same output: `calc()`'s `*` and `/`)
- Pin: `tests/fixtures/format/scss/slash-spacing.scss`
- Conformance:
  - `externals/prettier/css/font/font.css`

```scss
/* input */
.a {
  width: min(1px/2, 3px);
  font: calc(var(--size)/2)/2;
}

/* ours */
.a {
  width: min(1px / 2, 3px);
  font: calc(var(--size) / 2)/2;
}

/* prettier */
.a {
  width: min(1px/2, 3px);
  font: calc(var(--size)/2)/2;
}
```

Inside a math function (the `Calc` node: `calc`, `min`, `max`, `clamp`, `round`, `abs`, ...) `*` and `/` always print spaced;
`+` / `-` keep the source, they require whitespace and a glued one is invalid, left as written.
There a `/` is always a division, never a separator, so it is laid out as math:
css-values-4 serializes a calculation tree with `" * "` / `" / "`, and dart-sass prints `min(var(--x)/2, 3px)` as `min(var(--x) / 2, 3px)`.
Prettier spaces them inside `calc()` only, and not after a font-size value in `font` and custom properties (its #18114 rule above).

## paren-group-glued-word

- Why: semantics
- Pin: `tests/fixtures/format/css/paren-group-glued-word.css`
  (`css/postcss-plugins/postcss-simple-vars.css` carries the case too, but its root declarations fail to parse)

```css
/* input */
a {
  color: $(style)color;
  color: @@(style)color123;
}

/* ours */
a {
  color: $(style)color;
  color: @@(style)color123;
}

/* prettier */
a {
  color: $(style) color;
  color: @@(style) color123;
}
```

`$(style)color` is a postcss-simple-vars interpolation glued to a word, and the glue carries meaning:
after substitution it is one word (`redcolor`), the way `margin-$(dir)` becomes `margin-top`.
postcss-values-parser lexes it as a paren node followed by a word and Prettier prints a space between them,
changing the substituted value (`red color`); Prettier keeps `$$(style)Color` in the same conformance file glued, so it is not even consistent.
The value is the `<any-value>` raw fallback for us, and the value writer keeps source-glued tokens glued (`Separator::Tight`); it never inserts a space.

## glued-number-head

- Why: uniform-rule (same construct, same output: `sandstone.10`)
- Pin: `tests/fixtures/format/css/glued-number-head.css`

```css
/* input */
a {
  version: 1.20.3;
  b: f(1.20.3) 1.50em.5;
}

/* ours */
a {
  version: 1.20.3;
  b: f(1.20.3) 1.50em.5;
}

/* prettier */
a {
  version: 1.2.3;
  b: f(1.2.3) 1.5em.5;
}
```

Glued number-ish values can never be valid CSS, so the run is one word (`is_word_glued_number`):
Prettier prints a word-led glued run raw as a whole (`sandstone.10`) and every glued continuation raw (`.3`),
but normalizes a number-led run's head (`1.20`).
Ours prints that head raw as well, instead of normalizing a part of a word.
The value writer is shared, so SCSS and Less print the same (Prettier normalizes the head in all three).

## escaped-custom-property-case

- Why: semantics
- Pin: `tests/fixtures/format/css/custom-property-raw-verbatim.css`

```css
/* input and ours */
.a {
  \-\-CamelCase: red;
  color: var(\-\-CamelCase);
}

/* prettier */
.a {
  \-\-camelcase: red;
  color: var(\-\-CamelCase);
}
```

The escaped identifier `\-\-CamelCase` decodes to the custom property name `--CamelCase`.
Custom property names are case-sensitive, so lowercasing the declaration while preserving the reference makes `var(\-\-CamelCase)` unresolved.
Escaped custom property names therefore preserve their source spelling, like names written with a literal `--` prefix.

## custom-property-raw-verbatim

- Why: uniform-rule (raw is verbatim; AGENTS.md "Printing raw vs typed")
- Pin: `tests/fixtures/format/css/custom-property-raw-verbatim.css`, `tests/fixtures/format/scss/custom-property-raw-verbatim.scss`, `tests/fixtures/format/scss/custom-property-text.scss`
- Conformance:
  - `externals/prettier/css/postcss-8-improment/test.css`
  - `externals/prettier/scss/variables/postcss-8-improment.scss`
  - `externals/prettier/less/postcss-8-improment/test.less`

```css
/* input */
:root {
  --z: */;
  --x:   1px   !foo;
  --JSON: [1, "2", {"three": {"a":1}}, [4]];
  --javascript: function(rule) { console.log(rule) };
}

/* ours */
:root {
  --z: */;
  --x: 1px   !foo;
  --JSON: [1, "2", {"three": {"a":1}}, [4]];
  --javascript: function(rule) { console.log(rule) };
}

/* prettier */
:root {
  --z: * /;
  --x: 1px !foo;
  --JSON: [1, "2", {"three": {"a": 1}}, [4]];
  --javascript: function(rule) {console.log(rule)};
}
```

A custom property value is its token stream (css-variables-1) and text to dart-sass.
We lay it out only when the typed `<declaration-value>` grammar reads all of it;
when it does not (`Declaration::value_is_raw`), the value prints verbatim, in every variant.
Prettier hands such values to postcss-values-parser, its regular value grammar: it collapses inner whitespace,
adds a space after a `:` inside `{}`, strips the spaces inside a JS block, and splits `*/` because `/` is a `div` node to it.
Consistent for Prettier, and the same token stream for css-syntax (a whitespace run is one token), so admissible.
We print it verbatim because raw is verbatim everywhere else too (raw names, `UnknownQualifiedRule` and `TokenSeq` preludes, unknown at-rule params);
re-spacing a value we could not read is not a layout rule of ours.

## postcss-simple-var-raw-verbatim

- Why: semantics
- Pin: `tests/fixtures/format/css/postcss-simple-vars/vars.css`
- Conformance:
  - `externals/prettier/css/parens/empty-lines.css`

```css
/* input and ours */
$x: */;
a { --fragment: $x; }

/* prettier */
$x: * /;
a { --fragment: $x; }
```

postcss-simple-vars substitutes a variable's value textually.
With the plugin, the input produces `--fragment: */`, while Prettier's output produces `--fragment: * /`;
the added whitespace changes the custom property's preserved token stream.
A `$var` value the typed grammar cannot read therefore prints verbatim, like a raw custom-property value.

## important-comment-run

- Why: uniform-rule (same construct, same output: `!IMPORTANT`, which Prettier prints `!important`)
- Pin: `tests/fixtures/format/css/important-comments.css`

```css
/* input */
a {
  y: red ! /* a */ IMPORTANT;
  v: red !/* glued */important;
}

/* ours */
a {
  y: red ! /* a */ important;
  v: red ! /* glued */ important;
}

/* prettier */
a {
  y: red ! /* a */ IMPORTANT;
  v: red !/* glued */important;
}
```

`!important` is normalized the same way whether or not a comment sits between `!` and the keyword:
lowercase keyword, one space around the comment.
Prettier normalizes only the plain shape (`raws.important` is replaced when it matches `\s*!\s*important`)
and prints any other run verbatim, so a comment inside freezes the keyword's case and the glue.

## less-javascript-verbatim

- Why: semantics
- Pin: `tests/fixtures/format/less/javascript-verbatim.less`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/javascript/javascript.less`

```less
/* input */
.a {
  js: `"hello world"`;
  ary: `@{ary}.join(', ')`;
}

/* ours */
.a {
  js: `"hello world"`;
  ary: `@{ary}.join(', ')`;
}

/* prettier */
.a {
  js: ` "hello world" `;
  ary: ` @{ary}.join(", ") `;
}
```

A backtick JavaScript evaluation (`` `...` `` / `` ~`...` ``) is a program, not a CSS value:
`oxc-css-parser` keeps it as one `LessJavaScriptSnippet` and ours prints it verbatim.
Prettier's value printer sees the backticks as plain words and the content as CSS tokens:
it pads the backticks, re-quotes JavaScript strings and reflows a multi-line function onto one line.

## less-verbatim-glued-numbers

- Why: semantics
- Pin: `tests/fixtures/format/less/mixin-verbatim-numbers.less`

```less
/* input */
.m({ a: url(x.1.2.3.png) b.1.2.3; });
.m(1.2.3);
.h(#1e0; #0E00) when (@c = #1e0) {}
.n(@a: .50 /* 0.50 */) when (@a > .50) {}

/* ours */
.m({ a: url(x.1.2.3.png) b.1.2.3; });
.m(1.2.3);
.h(#1e0; #0E00) when (@c = #1e0) {
}
.n(@a: 0.5 /* 0.50 */) when (@a > 0.5) {
}

/* prettier */
.m({ a: url(x.10.20.3.png) b.10.20.3; });
.m(1.20.3);
.h(#1; #0) when (@c = #1) {
}
.n(@a: .50 /* 0.50 */) when (@a > .50) {
}
```

Less mixin calls, definitions and guards print as a verbatim slice with standalone numbers normalized (`adjust_numbers_and_strings`).
Prettier's word scan keeps only the FIRST number glued to a word and re-reads the rest as standalone:
`b.1.2.3` becomes `b.1` + `0.2` + `0.3` (and `1.2.3` becomes `1.2` + `0.3`), changing the value (inside a `url()` path, the file) and growing every pass (`a.10.2` -> `a.100.2` -> `a.1000.2`).
The same scan reads a hex color's digits as a number with an exponent: `#1e0` becomes `#1`, a different color.
Ours keeps a whole glued number run raw (the `is_word_glued_number` rule), reads a hash (`#1e0`) as a word,
and never adjusts comments or an unquoted `url()`.
Prettier also skips normalization when the slice holds a comment (a `/\/[/*]/` match bails to `selector-unknown`, printed verbatim);
ours normalizes it like the comment-free sibling (comment presence never changes the output), as mixin definitions and guards already do.

## less-escaped-string-gap

- Why: invariant
- Pin: `tests/fixtures/format/less/escaped-string-gap.less`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/variables/variables.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/strings/strings.less`

```less
/* input */
@quotes: "~" "~";

/* ours */
@quotes: "~" "~";

/* prettier */
@quotes: "~""~";
```

Two space-separated values keep their gap (the lossless contract): `"~" "~"` is a two-item list.
Prettier's `~` handling for escaped strings (`~"..."`) fires on a string whose content is `~` and glues the next value onto it.

## less-lookup-glue

- Why: uniform-rule (same construct, same output: `#namespace[$@prop-name]`)
- Pin: `tests/fixtures/format/less/lookup-glue.less`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-config/namespacing/namespacing-functions.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-config/namespacing/namespacing-media.less`

```less
/* input */
.a {
  width: .add(10px, 10px)[];
}
@media #ns.breakpoint(.valToGet[])[@max] {}

/* ours */
.a {
  width: .add(10px, 10px)[];
}
@media #ns.breakpoint(.valToGet[])[@max] {
}

/* prettier */
.a {
  width: .add(10px, 10px) [];
}
@media #ns.breakpoint(.valToGet[]) [@max] {
}
```

A lookup prints glued to what it reads from, as written, like a selector's attribute bracket.
Prettier glues one after a name (`#namespace[$@prop-name]`, since prettier/prettier#19782 and prettier/prettier#20279 also after `$@`)
and spaces one after a call's `)`, where its value printer treats `[` as a word boundary.
less.js (4.9.1) evaluates both forms the same.

## less-guard-list-inline

- Why: uniform-rule (same construct, same output: guard alternatives are a list, not selectors; the DIVERGENCES.md#less-extend-statement-break leak again)
- Pin: `tests/fixtures/format/less/mixin-selector-list.less`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/mixins-guards/mixins-guards.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/mixins-guards-default-func/mixins-guards-default-func.less`
  - `externals/ng-zorro-antd/components/style/mixins/customize.less`

```less
/* input */
.m(@x) when (default()), not(default()) {
}

/* ours */
.m(@x) when (default()), not(default()) {
}

/* prettier */
.m(@x) when (default()),
not(default()) {
}
```

The commas after `when` separate guard alternatives (`or`), so they print inline like any other list;
a comma before `when` is a real selector list and breaks the line.
Over the width the prelude breaks like a selector, one indent in: before `when` and after each `,`, never inside a `when <cond>` (`...)\n  when (@theme = dark) {`).
Prettier reads the whole prelude with postcss-selector-parser, where every top-level comma is a selector boundary, except that it throws on a `(`-led piece (`when (a), (b)` prints verbatim), so the same guard list breaks or not depending on whether the alternative starts with `not`;
over the width every word gap is a descendant combinator, so `when`, `and` and each condition land on their own line, while a `,`-joined guard never breaks at all.

## line-comment-continuation-indent

- Why: uniform-rule (same construct, same output: the width-wrapped continuation of the same operation)
- Pin: `tests/fixtures/format/less/line-comment-continuation-indent.less`, `tests/fixtures/format/scss/line-comment-continuation-indent.scss`
- Conformance:
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/comments/comments2.less`

```less
/* input */
@a: (@column-width * // c
  @columns) + 1;

/* ours */
@a: (
    @column-width * // c
      @columns
  ) +
  1;

/* prettier */
@a: (
    @column-width * // c
    @columns
  ) +
  1;
```

An operation's continuation line is indented one level under the operation, whatever broke the line:
the width (`@column-width *\n      @a-very-long-name`, identical in Prettier) or a `//` comment.
Prettier prints the comment-forced break inside a paren group as `dedent(hardline)` (prettier/prettier#7844, aimed at `//` lines in maps), so the same operation continues at the paren's indent after a comment but one level deeper after a width wrap.
Outside parens the two agree (the root indent is the operation's indent there).

## prelude-keyword-case

- Why: uniform-rule (same construct, same output: a case-insensitive keyword prints lowercase wherever it stands)
- Pin: `tests/fixtures/format/css/prelude-keyword-case.css`

```css
/* input */
@media ONLY SCREEN AND (MAX-WIDTH: 1PX), PRINT {}
@import url("a.css") SCREEN AND (MAX-WIDTH: 1px);
@supports (DISPLAY: FLEX) AND (NOT (DISPLAY: GRID)) {}

/* ours */
@media only screen and (max-width: 1px), print {}
@import url("a.css") screen and (max-width: 1px);
@supports (display: FLEX) and (not (display: GRID)) {}

/* prettier */
@media ONLY SCREEN AND (max-width: 1px), PRINT {}
@import url("a.css") SCREEN AND (MAX-WIDTH: 1px);
@supports (DISPLAY: FLEX) AND (NOT (DISPLAY: GRID)) {}
```

At-rule names, property names, media feature names and the prelude keywords next to them (media types, `only` / `and` / `not` / `or`, a `@supports` declaration's property) are all ASCII case-insensitive, so they all print lowercase, the way at-rule and property names already do.
Values keep their case (`LANDSCAPE`, `FLEX`), as in any declaration: a value may be a case-sensitive custom ident (`animation-name`, `grid-area`).
So do case-sensitive names (`layer(FOO)`) and any identifier carrying a variable or interpolation marker (`@media @PHONE`, `#{$Q}`).
Prettier lowercases only what its `maybeToLowerCase` reaches (at-rule names, `media-feature`, declaration props) and prints the neighbouring keywords as its media-query parser or value parser hands them over: verbatim.

## url-string-comment

- Why: uniform-rule (same construct, same output: a comment after a function argument, `calc(1px /* c */)`)
- Pin: `tests/fixtures/format/css/function-argument-comments.css`

```css
/* input */
a {
  background: url("x" /* c */);
  width: calc(1px /* c */);
}

/* ours */
a {
  background: url("x" /* c */);
  width: calc(1px /* c */);
}

/* prettier */
a {
  background: url("x"/* c */);
  width: calc(1px /* c */);
}
```

A comment after a quoted `url()` string is spaced from it, as one after any other function argument;
Prettier glues it to the string there only.

## import-media-feature-flat

- Why: uniform-rule (same construct, same output: the media feature in `@media`)
- Pin: `tests/fixtures/format/css/import-prelude-comments.css`
- Conformance:
  - `externals/prettier/css/comments/at-rules.css`

```css
/* input */
@import url("a.css") screen and (orientation-something-very-long-name-xxxxxxxxx: landscape-value-quite-long-yyyyyyyy);
@media screen and (orientation-something-very-long-name-xxxxxxxxx: landscape-value-quite-long-yyyyyyyy) {}

/* ours */
@import url("a.css") screen and
  (orientation-something-very-long-name-xxxxxxxxx: landscape-value-quite-long-yyyyyyyy);
@media screen and (orientation-something-very-long-name-xxxxxxxxx: landscape-value-quite-long-yyyyyyyy) {
}

/* prettier */
@import url("a.css") screen and
  (
    orientation-something-very-long-name-xxxxxxxxx:
      landscape-value-quite-long-yyyyyyyy
  );
@media screen and (orientation-something-very-long-name-xxxxxxxxx: landscape-value-quite-long-yyyyyyyy) {
}
```

An `@import` media feature never breaks inside its parens, however long, as in `@media`:
both preludes end in the same `<media-query-list>`, which parses to the same nodes and prints through the same printer.
Prettier value-parses an `@import` prelude, so an over-wide feature there opens its parens and breaks after the `:`.

## media-feature-comment-case

- Why: invariant
- Pin: `tests/fixtures/format/css/media-commented-feature.css`

```css
/* input */
@media (/* A */ COLOR) {}

/* ours */
@media (/* A */ color) {
}

/* prettier */
@media (/* a */ color) {
}
```

A comment in a media feature keeps its text, only the code around it lowercases;
Prettier lowercases the whole feature, so the comment's text is no longer the user's.

## missed-semicolon-accepted

- Why: uniform-rule (acceptance: the grammar owner decides)
- Pin: `tests/fixtures/format/css/missed-semicolon-accepted.css`

```css
/* input */
a {
  left: 0
  top: 0;
}

/* ours */
a {
  left: 0 top: 0;
}

/* prettier: CssSyntaxError: Missed semicolon, the input is left as-is */
```

A `;`-less declaration runs to the next `;`: `oxc-css-parser` reads one declaration, a value being any component-value run (css-syntax-3; oxc-css-parser AGENTS.md "Acceptance");
postcss rejects a value with a `word:` after its first word.
Same tokens, so same meaning: a browser drops the declaration,
or keeps it the same way when the value holds an arbitrary substitution function (`var()`, `env()`).
SCSS / Less keep rejecting it (dart-sass / lessc), so those stay as-is.
lessc's `anonymousValue()` shortcut stores some `;`-terminated runs (no `(`, `.`, `-`, ...) as text without parsing them;
that is not its value grammar, so it is not followed.

## combinator-spacing

- Why: uniform-rule (same construct, same output: the `>` / `+` / `~` combinators)
- Pin: `tests/fixtures/format/css/combinator-spacing.css`

```css
/* input */
col.selected||td {}
col.selected || td {}
.a^b {}
.a^^b {}

/* ours */
col.selected || td {
}
col.selected || td {
}
.a ^ b {
}
.a ^^ b {
}

/* prettier */
col.selected||td {
}
col.selected||td {
}
.a^b {
}
.a^^b {
}
```

Every combinator other than the descendant one prints with a space on each side (`selector.rs` `write_combinator`);
Prettier spaces only `>` / `+` / `~` / `>>>`, prints the others as written, and glues a spaced `||` too.

## document-url-brace

- Why: uniform-rule (same construct, same output: `@document url("x") {`)
- Pin: `tests/fixtures/format/css/document-url-brace.css`

```css
/* input */
@document url("https://www.example.com/") {}

/* ours */
@document url("https://www.example.com/") {
}

/* prettier */
@document url("https://www.example.com/")
{
}
```

The typed `@document` / `@-moz-document` prelude keeps the `{` on its line;
Prettier takes the `//` of `https://` for a line comment and moves the `{` to its own line.
A raw prelude (unknown at-rule params) still takes the same `//` test as Prettier (`comments.rs` `last_line_has_inline_comment`), so there the two agree.

## value-semicolon-glue

- Why: uniform-rule (same construct, same output: the SCSS `if()` branch separator; prettier/prettier#19384)
- Pin: `tests/fixtures/format/css/value-semicolon-glue.css`
- Conformance:
  - `externals/mdn-css-guides/custom_functions_and_mixins/using_custom_functions/index.md`

```css
/* input */
a {
  b: if(media(width < 700px): 1 ; else: 2);
  c: foo(x ; y);
}

/* ours */
a {
  b: if(media(width < 700px): 1; else: 2);
  c: foo(x; y);
}

/* prettier */
a {
  b: if(media(width < 700px): 1 ; else: 2);
  c: foo(x ; y);
}
```

A `;` in a value glues to what precedes it, in every dialect (`value.rs` `base_separator`);
Prettier glues it only between SCSS `if()` branches and keeps the source space elsewhere.

## supports-declaration-as-in-rule

- Why: uniform-rule (same construct, same output: the declaration in a rule)
- Pin: `tests/fixtures/format/css/supports-declaration-long-value.css`, `tests/fixtures/format/css/supports-function-args-indent.css`, `tests/fixtures/format/css/supports-commented-terms.css`
- Conformance:
  - `externals/prettier/css/atrule/supports.css`
  - `externals/prettier/css/comments/at-rules.css`

```css
/* input */
@supports (text-decoration-skip-ink: an-experimental-keyword-value-that-is-quite-long-and-longer) {}
@supports not (clip-path: shape(from center left, curve by 200px 0 with 50% -50% from start / 50% 0 from origin, close)) {}
@supports (/* a comment long enough to break this term */ grid-template-columns: repeat(auto-fill, minmax(100px, 1fr))) {}
a {
  text-decoration-skip-ink: an-experimental-keyword-value-that-is-quite-long-long-long;
}

/* ours */
@supports (
  text-decoration-skip-ink: an-experimental-keyword-value-that-is-quite-long-and-longer
) {
}
@supports not (
  clip-path: shape(
    from center left,
    curve by 200px 0 with 50% -50% from start / 50% 0 from origin,
    close
  )
) {
}
@supports (
  /* a comment long enough to break this term */
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr))
) {
}
a {
  text-decoration-skip-ink: an-experimental-keyword-value-that-is-quite-long-long-long;
}

/* prettier */
@supports (
  text-decoration-skip-ink:
    an-experimental-keyword-value-that-is-quite-long-and-longer
) {
}
@supports not (
  clip-path: shape(
      from center left,
      curve by 200px 0 with 50% -50% from start / 50% 0 from origin,
      close
    )
) {
}
@supports (
  /* a comment long enough to break this term */
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr))
) {
}
a {
  text-decoration-skip-ink: an-experimental-keyword-value-that-is-quite-long-long-long;
}
```

A `@supports` declaration prints as the same declaration does in a rule;
Prettier prints it through its prelude value parser instead, as a fill of words in a value group adding its own indent:

- An over-wide one opens its parens and stays on one line; Prettier also breaks after its `:`
- A broken function call indents its arguments one level; Prettier indents them and the `)` one level further
- A comment leading it ends its line once the parens open, the declaration at the comment's indent; Prettier puts the declaration one level in

## declaration-colon-comments

- Why: uniform-rule (same construct, same output: a `@supports` declaration's and a media feature's `:`)
- Pin: `tests/fixtures/format/css/declaration-colon-comments.css`
- Conformance:
  - `externals/prettier/css/comments/declaration.css`
  - `externals/prettier/scss/comments/between-decl.scss`
  - `externals/prettier/scss/comments/custom-properties.scss`
  - `externals/prettier/less/comments/between-decl.less`
  - `externals/prettier/less/comments/selectors.less`
  - `externals/prettier/less/less-test-suite/snippet: tests-unit/comments/comments.less`

```css
/* input */
a {
  display /* c */ : flex;
  display :/* c */ flex;
}
@supports (display /* c */ : flex) {}

/* ours */
a {
  display /* c */: flex;
  display: /* c */ flex;
}
@supports (display /* c */: flex) {
}

/* prettier */
a {
  display/* c */ : flex;
  display:/* c */ flex;
}
@supports (display /* c */: flex) {
}
```

Block comments around a declaration's `:` are spaced as in a `@supports` declaration or a media feature, where Prettier prints them that way:
the `:` glued to what precedes it, one space around each comment.
Prettier prints a rule's `raws.between` as written but for its ends, so the space before the first comment goes, the rest stays, and a line break in it lands at column 0.
A line break inside the between, a `//` or a Less merge marker keeps it as written, as Prettier prints it (a comment starting a line after the `:` or alone on its line keeps it);
a line break only at its ends goes (`display\n  /* c */ : flex` -> `display /* c */: flex`).

## grid-value-broken-in-function

- Why: invariant
- Pin: `tests/fixtures/format/scss/grid-value-broken-in-function.scss`
- Conformance:
  - `externals/gitlab/stylesheets/page_bundles/work_items.scss`
  - `externals/webawesome/page/page.styles.ts`

```scss
/* input (= ours; Prettier's output for the value on one line) */
a {
  b {
    grid-template-columns: minmax(
        0,
        1fr
      ) $work-item-overview-right-sidebar-width-x;
  }
}

/* prettier */
a {
  b {
    grid-template-columns:
      minmax(0, 1fr)
      $work-item-overview-right-sidebar-width-x;
  }
}
```

Prettier's output is not a fixpoint here.
A grid value too long for one line breaks inside its function, as Prettier prints it from one line;
Prettier then reads the line breaks it printed inside the function as the grid's line structure,
so a second pass moves each top-level value to its own line.

## value-multiline-comment-expands

- Why: uniform-rule (same construct, same output: a multi-line block comment in JS)
- Pin: `tests/fixtures/format/css/value-multiline-comment.css`

```css
/* input */
a {
  b: f(to top, x 1px /* c,
    d */);
}

/* ours */
a {
  b: f(
    to top,
    x 1px
      /* c,
    d */
  );
}

/* prettier */
a {
  b: f(to top, x 1px /* c,
    d */);
}
```

A block comment spanning lines inside a value expands its enclosing groups,
like Prettier's JS, JSONC, HTML and Markdown comments (`literalline` / `hardline` carry a break-parent).
Prettier's CSS prints the `value-comment` as a raw string with `\n`, so only its width decides:
the group stays flat when the whole comment, measured as one line, fits.
