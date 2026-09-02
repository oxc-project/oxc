# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".

## array-hole-trailing-comment

- Why: prettier-bug (attachment artifact)
- Pin: `tests/fixtures/js/comments/array-holes.js`

```js
// input
const a = [1, , /* c */];

// ours
const a = [1, ,/* c */];

// prettier
const a = [1 /* c */, ,];
```

Prettier's attachment relocates the comment backward across the commas to the last real element; we keep it in place.

## type-alias-trailing-comment-move

- Why: uniform-rule
- Pin: `tests/fixtures/ts/semicolons/trailing-comments.ts`

```ts
// input
export type T = string /* c */;
type U = string /* c */;

// ours
export type T = string; /* c */
type U = string; /* c */

// prettier
export type T = string; /* c */
type U = string /* c */;
```

Prettier's attachment moves the comment behind the `;` only in the exported form; one uniform move-behind-the-terminator rule instead of emulating the asymmetry.

## paren-trailing-comment-operand-chain

- Why: uniform-rule
- Pin: `tests/fixtures/js/comments/unary-argument-parens.js`

```js
// input
!(
  a &&
  b // B
);

// ours
!(
  a && b // B
);

// prettier
!(
  a &&
  b // B
);
```

A trailing comment before a closing paren never breaks the operand chain, as both formatters already do in every other paren-surviving position (return/throw argument, call argument, assignment, arrow body).
Prettier preserves the source break only in the unary position and only when the last operand was alone on its source line (attachment binds the comment to that operand): internal inconsistency plus source-layout sensitivity, overridden by the uniform rule.
Conditions are a separate shared rule (logical operands always break).

## operator-position-comment-double-space

- Why: prettier-bug (separator-space artifact)
- Pin: `tests/fixtures/js/operator-position/comments.js`

`experimentalOperatorPosition: "start"`, binary-like chains: a single space before the previous operand's flushed trailing line comment.

```js
// input
y = aaaaaaaaaaaaaaaaaaaaaa && bbbbbbbbbbbbbbbbbbbbbbbb && // same line
// own line comment
cccccccccccccccccccccccccc;

// ours
y =
  aaaaaaaaaaaaaaaaaaaaaa
  && bbbbbbbbbbbbbbbbbbbbbbbb // same line
  // own line comment
  && cccccccccccccccccccccccccc;

// prettier ("bbbb  //": two spaces)
y =
  aaaaaaaaaaaaaaaaaaaaaa
  && bbbbbbbbbbbbbbbbbbbbbbbb  // same line
  // own line comment
  && cccccccccccccccccccccccccc;
```

An artifact of Prettier's comment-extraction doc surgery: an unconditional separator space that its end-of-line trimming can only remove when no line-suffix comment flushes behind it.

## operator-position-intersection-own-line-comment

- Why: uniform-rule
- Pin: `tests/fixtures/ts/operator-position/intersection.ts`

`experimentalOperatorPosition: "start"`, intersection types: a leading own-line comment stays own-line, above the leading `&`.

```ts
// input
type WithComment = SerializedProps &
  // own line comment
  { cause: unknown };

// ours
type WithComment = SerializedProps
  // own line comment
  & { cause: unknown };

// prettier
type WithComment = SerializedProps
  & // own line comment
  { cause: unknown };
```

Prettier prints it behind `& `, losing its own-line-ness and idempotency (the second pass inlines the type with the comment behind `;`).
Binary-like chains hoist the comment in both formatters; one uniform rule (and the own-line invariant) over Prettier's internal inconsistency.

## union-added-paren-comment-side

- Why: uniform-rule
- Pin: `tests/fixtures/ts/union/paren-comments.ts`

```ts
// input
type KO = keyof /* c */ (A | B);

// ours
type KO = keyof /* c */ (A | B);

// prettier
type KO = keyof (/* c */ A | B);
```

Inline comments around a union's formatter-added `(` keep their source side.
Prettier moves the comment inside for `keyof`/type-operator operands while keeping it outside in array/indexed-access positions.

## eol-comment-after-assign-colon

- Why: uniform-rule (prettier#14617-family attachment artifact)
- Pin: `tests/fixtures/js/comments/assignment-eol-line-comment.js`, `tests/fixtures/ts/comments/operator-eol-line-comment.ts`

An end-of-line line comment right after `=`/`:` keeps its position (`= // c` + mandatory break).

```ts
// input
const v1 = // c
  1;
type Alias = // c
  "VALUE";

// ours
const v1 = // c
  1;
type Alias = // c
  "VALUE";

// prettier
const v1 = 1; // c
type Alias =
  // c
  "VALUE";
```

Prettier treats the same shape three ways:

- JS keeps it only when the right-hand side breaks and flushes it past a fitting one (the prettier#14617-family attachment artifact)
- TS type aliases and union-valued property signatures get it own-lined (the 3.9 union rewrite)
- simple-typed property signatures get it flushed past the member and its `;` separator

Not yet covered: default parameters, destructuring defaults, enum members (different formatting paths still flush, Prettier-compatible).

## union-leading-pipe-comment-normalization

- Why: uniform-rule (Prettier's exceptions are not idempotent)
- Pin: `tests/fixtures/ts/union/leading-pipe-comments.ts`

A union's leading comments normalize to behind the leading `|` (`| /* c */ A`) whenever no comment ends its source line, regardless of the source shape.

```ts
// input
type NestedParens = | (
  /* c */ | (
    | A
    // force break
    | AmemberLongEnoughToMakeTheUnionTypeBreakIntoMultipleLines
  )
);

// ours
type NestedParens =
  | /* c */ A
  // force break
  | AmemberLongEnoughToMakeTheUnionTypeBreakIntoMultipleLines;

// prettier (comment kept before the `|`)
type NestedParens =
  /* c */ | A
  // force break
  | AmemberLongEnoughToMakeTheUnionTypeBreakIntoMultipleLines;
```

Prettier does the same normalization except for nested single-member paren sources and multiline block comments starting their line, where it keeps `/* c */ | A` — an output it then reformats into `| /* c */ A` itself for the first shape (not idempotent); we normalize directly.

## deferred-own-line-comment-stays-own-line

- Why: uniform-rule (the own-line invariant; Prettier's glue is an attachment artifact)
- Pin: `tests/fixtures/ts/semicolons/trailing-comments.ts`

An own-line comment deferred across a statement terminator (printed behind the previous node) stays own-line, with its blank lines preserved; the already-printed `;` is transparent to the break measurement (`lines_after_skipping_terminators`):

```ts
// input
bar = 2
/* own line */;

quux();

// ours
bar = 2;
/* own line */

quux();

// prettier (comment glued onto the next statement, the blank hoisted above it)
bar = 2;

/* own line */ quux();
```

Prettier attaches the comment as the next statement's leading and measures its break from the source `;`, gluing them; ours keeps the comment's own-line-ness and its distance to both neighbors as written.

## paren-comment-fixpoint

- Why: uniform-rule (upstream converging piecewise: prettier#19893 / prettier#19894 / prettier#19930 merged, prettier#19956 open)
- Pin: `tests/fixtures/ts/semicolons/trailing-comments-parens.ts`, `tests/fixtures/ts/semicolons/trailing-comments-class-members.ts`, `tests/fixtures/js/sequence-expression/leading-comment-in-first-element-parens.js`

Comment placements printing Prettier's second-pass fixpoint directly, where the pinned Prettier is not idempotent:

```js
// input
assigned = (a = c /* c1 */);
((/* c2 */ a), b);
chained = (a) => ((b) => {
  c();
} /* c3 */)
asiLeaf = (someValue /* c4 */)

// ours
assigned = a = c; /* c1 */
/* c2 */ (a, b);
chained = (a) => (b) => {
  c();
}; /* c3 */
asiLeaf = someValue; /* c4 */

// prettier (first pass; its second pass produces ours)
assigned = a = c /* c1 */;
(/* c2 */ a, b);
chained = (a) => (b) => {
  c();
} /* c3 */;
asiLeaf = someValue /* c4 */;
```

- A trailing comment inside a statement's dropped parentheses moves behind the `;`, including the chain-leaf shapes prettier#19893 left out
- The chain passes through arrow expression bodies, and a dropped `)` counts as the terminator even without a source `;`
  - prettier#19930 covers assignment/arrow links, the open prettier#19956 the rest; ours is one uniform rule
- The walk stops where the comment stays inside re-added parens (sequence, assignment, JSX); conditional bodies move
- Applied at every semicolon-terminated site: expression statements, export defaults, variable declarations, class property values, return arguments
- A class member's same-line comment moves even when an own-line comment sits between it and the `;` (the pinned Prettier cancels the move on its first pass, then moves on its second — the `d = 4` and `dmixed` pins); the own-line one defers to the next element, like statements
- A comment inside a sequence's parenthesized first element leads the sequence, outside the formatter-added parens

Known limitation, shared with upstream: whether these parens survive is a group-fit decision, unknowable when the content end is chosen, so the comment settles only on the second pass when they end up dropped — a short JSX arrow body (`x = (a) => (<div /> /* c */)`), or a `;`-less return argument whose parenthesized binary fits flat (the `asiReturn` pin).

## binary-cast-own-line-comment

- Why: uniform-rule (upstream converging piecewise: prettier#19939 merged, prettier#19958 open)
- Pin: `tests/fixtures/ts/comments/binary-cast-own-line-comment.ts`

Comments around the operator keep their source side and their line-start side — the head-body comment policy applied to the operator gap, unifying it with `=`/`:` (`eol-comment-after-assign-colon`) and statement head-body gaps:

- before the operator: trail the expression, never cross the operator — except what the grammar-defined slot cannot hold (no line terminator may precede the operator, a multiline comment's interior counts): those normalize to the type side so the output re-parses
- glued after the operator: stay on the operator's line; a line comment forces the type onto the next line, a single-line block comment leaves the layout free to inline the type
- own-line (and a line-ending multiline block, paragraph-like): lead the type on its own line, the type breaks under the operator

```ts
// input
const t1 = {
  prop1: 1,
} satisfies
// Comment
Record<string, number>;
const eolLine = 1 as // c
Foo;
const eolBlock = {} satisfies /* c */
{};

// ours
const t1 = {
  prop1: 1,
} satisfies
  // Comment
  Record<string, number>;
const eolLine = 1 as // c
  Foo;
const eolBlock = {} satisfies /* c */ {};

// prettier (pinned): pulls the own-line comment beside the operator (three passes
// to settle), relocates the line comment across the type and the `;`,
// and the block comment backward across the operator
const t1 = {
  prop1: 1,
} satisfies // Comment
Record<string, number>;
const eolLine = 1 as Foo; // c
const eolBlock = {} /* c */ satisfies {};
```

Prettier is converging on its own piecewise fixes (own-line comments in prettier#19939, endOfLine comments reattached in the open prettier#19958, both normalizing toward own-line); we preserve the written position instead, so the entry outlives them.

`as const` follows the same policy (`const` is a type like any other; the pinned Prettier relocates its comments across `const` and the `;`).
The one exclusion, pinned in the fixture: union types defer to the union printer's own comment claiming (a same-line line comment before a union still moves behind the statement, crossing the type — an invariant violation tolerated only here) — that claiming is its own subsystem, see #union-leading-pipe-comment-normalization and #union-added-paren-comment-side.
Drop when: the union printer's claiming is bounded to its own gap; this exclusion then collapses into the general slot rule above (the `unionEol` pin flips).

## head-body-comment-relocation

- Why: prettier-bug (attachment artifacts of the class Prettier is fixing elsewhere: the prettier#19894 family, open prettier#12880 / prettier#7745 / prettier#5900)
- Pin: see the per-shape list below
- Drop when: the family fixes cover these shapes and the pin catches up

The head-body comment policy family (see AGENTS.md "Comment placement invariants"): comments between a head and its body keep their position, where Prettier's attachment relocates them.

```js
// input
function f() // c
{
  g();
}

// ours
function f() // c
{
  g();
}

// prettier
function f() {
  // c
  g();
}
```

The shapes, each pinned by its fixture:

- Line and own-line comments before a body's `{` stay outside the braces; Prettier pulls them inside (function/arrow/method/class/try/catch/finally/interface/switch clauses), past the braces entirely (enum/namespace/module), into the catch parameter's parens, or hoists a labeled statement's comment above the label
  - `tests/fixtures/js/comments/head-body-blocks.js`, `tests/fixtures/js/comments/try-catch-finally-head.js`, `tests/fixtures/ts/comments/declaration-head-body.ts`, `tests/fixtures/ts/comments/method.ts`, `tests/fixtures/js/switch/clause-block-comment.js`
- The same before an empty-statement body's `;`; Prettier pulls the comment backward inside the head's parens (`while (x /* c */);`) or hoists an own-line one onto the head line
  - `tests/fixtures/js/comments/empty-statement-head.js`
- Comments in a classic for-head keep their slot between the `;`s; Prettier moves empty-slot comments backward across the `;`s onto the init, or forward out of the parens entirely when every slot is empty (`for (/*c*/;;)` -> `for (;;) /*c*/`)
  - `tests/fixtures/js/comments/for-head-slots.js`
- Comments between a `}` and a following `else`/`catch`/`finally`/`while` keep their side of the keyword; Prettier pulls them into the next block (or past a do-while's whole `while (x);` head)
  - `tests/fixtures/js/comments/try-catch-finally-head.js`, `tests/fixtures/js/comments/do-while-head.js`
- A line comment inside a for-in/for-of head stays before the `)` (the head flushes it); Prettier moves it past the body's `{` and is not idempotent there (prettier#12880)
  - `tests/fixtures/js/comments/head-paren-line-comment.js`
- A blank line between an own-line comment and a following `else` is preserved like any other leading position; Prettier collapses it
  - `tests/fixtures/ts/comments/if.ts`
- An `if` consequent's trailing line comment rides the line; Prettier's attachment marks the consequent multiline and breaks it onto its own line — only with an `else`: every sibling shape (plain statement, no-`else` `if`, `while`/`for` bodies) stays inline in both formatters
  - `tests/fixtures/js/comments/head-body-blocks.js`
- A multiline block comment before the `{` stays inline like any same-line block comment; Prettier own-lines it for while/do/else heads only, keeping it inline everywhere else
  - `tests/fixtures/js/comments/multiline-block-head.js`

## union-annotation-flat-retry

- Why: style-hold (oxc-project/oxc#25841)
- Pin: `tests/fixtures/ts/union/annotation-flat-retry.ts` (also tracked by oxfmt's conformance suite, e.g. vue-vben-admin `api-component.vue`, webawesome `*.ts`)
- Drop when: the wait-and-see on Prettier 3.9's union style resolves (follow, or re-classify)

```ts
/* input */
autoSelect?: "first" | "last" | "one" | ((item: OptionsItem[]) => OptionsItem) | false;

/* ours */
autoSelect?:
  | "first"
  | "last"
  | "one"
  | ((item: OptionsItem[]) => OptionsItem)
  | false;

/* prettier */
autoSelect?:
  "first" | "last" | "one" | ((item: OptionsItem[]) => OptionsItem) | false;
```

A union broken out of its `:`/`as` position expands to leading-`|` members right away;
Prettier 3.9 changed this to first retry the whole union FLAT on the indented next line and only then expand.
Community pushback on the 3.9 style (users migrated to Oxfmt over it) put following on hold:
ours deliberately matches Prettier 3.8's output for this construct.

## arrow-chain-exact-fill-signature

- Why: cost
- Pin: `tests/fixtures/js/arrow-function/chain-exact-fill-signature.js`

```js
/* input (printWidth 80) */
const exactlyFillsThePrintWidthEightySignatureLinePaddedToLength = (x) => (y) => cond ? longlonglong : shortshort;

/* ours */
const exactlyFillsThePrintWidthEightySignatureLinePaddedToLength = (x) => (y) =>
  cond ? longlonglong : shortshort;

/* prettier */
const exactlyFillsThePrintWidthEightySignatureLinePaddedToLength =
  (x) => (y) => (cond ? longlonglong : shortshort);
```

An arrow chain with a parens-adding conditional body keeps a signature line that EXACTLY fills
the print width (the signature alone is measured; both idempotent). Prettier counts the hug layout's
literal trailing space, so the exact fill overflows: it breaks the assignment and retries the chain
flat on the indented next line. One char over and the outputs converge; a naive port of the
literal-space structure regresses `js/arrows/currying-4.js` (Prettier gates the hug on the chain's
expand state), so matching costs more than this exact-width edge is worth.

## suppressed-for-head-declaration

- Why: semantics (Prettier's output no longer parses)
- Pin: `tests/fixtures/js/semicolons/suppressed-for-head-declaration.js`
- Drop when: Prettier stops re-adding `;` for a suppressed `for` head declaration

```js
// input
for (/* prettier-ignore */ var i   =   1;;) [].sort();

// ours
for (/* prettier-ignore */ var i   =   1; ;) [].sort();

// prettier
for (/* prettier-ignore */ var i   =   1;; ;) [].sort();
```

Prettier's `shouldIgnoredNodePrintSemicolon` lists `VariableDeclaration` unconditionally,
so a suppressed declaration in a `for` head gets an extra `;` and the head no longer parses
(a `for (;;)` head admits exactly two semicolons).
In the head the declaration has no terminator of its own; we keep it verbatim and let the `for` statement print its separators.

## suppressed-source-paren-asi-guard

- Why: semantics (Prettier's output re-parses as a call)
- Pin: `tests/fixtures/js/semicolons/suppressed-source-paren-asi-guard.js`
- Drop when: Prettier guards a suppressed statement whose verbatim text starts with `(`

```js
// input (semi: false)
let x = 1;

// prettier-ignore
(sourceParen).sort();

// ours
let x = 1

// prettier-ignore
;(sourceParen).sort()

// prettier
let x = 1

// prettier-ignore
(sourceParen).sort()
```

A suppressed expression statement prints its source text, which keeps parens the reprint would drop, so the line starts with `(` and needs the `semi: false` ASI guard.
Prettier's `expressionNeedsAsiProtection` walks the AST's naked left side and never sees the source paren, so its output re-parses as `1(sourceParen)...`.
We check the verbatim range's first byte instead and print the guard.

## suppressed-cast-comment-asi-guard

- Why: semantics (Prettier's guard placement detaches the type cast; verified with tsc)
- Pin: `tests/fixtures/js/semicolons/suppressed-cast-comment-asi-guard.js`
- Drop when: Prettier prints the ignored-slice ASI guard before a leading type cast comment

```js
// input (semi: false)
// prettier-ignore
/** @type {string[]} */ (cast).sort();

// ours
// prettier-ignore
;/** @type {string[]} */ (cast).sort()

// prettier
// prettier-ignore
/** @type {string[]} */ ;(cast).sort()
```

A cast comment types its parenthesized expression only when directly adjacent:
with Prettier's placement tsc reports the target as its uncast type again.

Prettier's `printIgnored` prepends the guard to the ignored slice, which starts after the leading comments; we reuse the reprint path's split (`ExpressionStatement::write`), so the guard, the cast comment, and the verbatim content print in that order.
