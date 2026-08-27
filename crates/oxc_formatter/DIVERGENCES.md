# Known divergences

Admission reasons and rules: see `crates/oxc_formatter_core/FORMATTER_POLICY.md` "Known divergences".
The entries documented so far are not yet an exhaustive audit against the conformance snapshots.

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

## paren-comment-fixpoint

- Why: prettier-bug (fixed upstream in prettier#19893 / prettier#19894)
- Pin: `tests/fixtures/ts/semicolons/trailing-comments.ts`, `tests/fixtures/js/sequence-expression/leading-comment-in-first-element-parens.js`
- Drop when: the pin catches up (converge on Prettier's now-fixed output)

Two comment placements print Prettier's second-pass fixpoint directly, where the pinned Prettier is not idempotent:

```js
// input
assigned = (a = c /* c1 */);
((/* c2 */ a), b);

// ours
assigned = a = c; /* c1 */
/* c2 */ (a, b);

// prettier (first pass; its second pass produces ours)
assigned = a = c /* c1 */;
(/* c2 */ a, b);
```

- A trailing comment inside an expression statement's dropped parentheses moves behind the `;`, including the chain-leaf shapes prettier#19893 left out
- A comment inside a sequence's parenthesized first element leads the sequence, outside the formatter-added parens

## head-body-comment-relocation

- Why: prettier-bug (attachment artifacts of the class Prettier is fixing elsewhere: the prettier#19894 family, open prettier#12880 / prettier#7745 / prettier#5900)
- Pin: see the per-shape list below

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
