# Design

Why `oxc-codegen` is shaped the way it is.

It is a port of Oxc's Rust [`oxc_codegen`] crate, and its output has to be byte-identical to it.
But it is not a transliteration. A straight port would not take into account differences between Rust and JS,
and would likely be several times slower.

This document is for someone about to change the printer, wondering which of its oddities are load bearing.
Everything here is also documented at the point it applies. This is an overview.

## Contents

- [Why not native?](#why-not-native)
- [What this is](#what-this-is)
- [What it is not at present](#what-it-is-not-at-present)
- [`state.last`](#statelast)
- [Four builds from one source tree](#four-builds-from-one-source-tree)
- [Other things which make it fast](#other-things-which-make-it-fast)
- [Where the ESTree AST differs](#where-the-estree-ast-differs)
- [How it is tested](#how-it-is-tested)
- [Rules to keep](#rules-to-keep)

## Why not native?

Oxc's other JS packages are bindings around a native library. This one deliberately is not.

The starting point was a hunch:

> **When the AST is already in JS, no native printer can beat a JS one.**

### The argument

- **Printing is cheap.** It is a linear walk of the AST, emitting strings. There is no parsing, no analysis,
  no search. It doesn't play to the strengths of a faster language the way parsing or minifying does.
- **Crossing into native is not cheap.** A native printer cannot see a JS object graph.
  The whole AST has to be serialized across the boundary before it can print a single character.
- **The boundary dominates.** The serialization is proportional to the size of the AST, just as the printing is -
  but it is work a JS printer can skip entirely. It is added to a total that was small to begin with.

The conclusion: A native printer loses the game before it's even started - still shovelling data over to native side,
while the JS printer already finished work and has gone off to the pub.

### Where this applies

Only when the AST is **already** in JS - it came from `oxc-parser`, or a plugin built it, or something
transformed it on JS side. That is the common use case this package is for.

It does **not** apply when you are printing source text you also parsed natively. Then stay on native side -
the AST never has to leave native memory, and Rust `oxc_codegen` is the right tool. The two are complementary.

The benchmarks in the [README](./README.md) measure `oxc-codegen` over representative JS-held ASTs.

## What this is

A printer from an ESTree AST to source text, written in TypeScript.

### The output contract

Byte-identical to `oxc_codegen` with default options - tab indentation, double quotes, comments off.

The conformance suites check this on every fixture. It is what makes the two printers substitutable for one another.

## What it is not at present

`oxc_codegen` (Rust printer) is more capable than this one.

- No minify mode.
- No support for comments.
- No symbol mangling.

Many of these features can be added in future, and we should be able to do so without _much_ impact on performance.

## `state.last`

This is the largest divergence from Rust, and most of the others follow from it.

### The problem

Two adjacent tokens can merge into a different token when written straight up against each other.
`in` after an identifier becomes part of that identifier. `+` after `+` becomes `++`.

Deciding whether to insert a space means knowing what was written last.

### How Rust answers it

Rust just looks. Its output buffer is a `Vec<u8>`, so `code.last_byte()` is a pointer dereference - very cheap.

### Why this doesn't work in JS

**In JS, reading the last character is catastrophic for perf.** `state.output` is built by repeated `+=`,
which V8 represents as a **cons string** - a tree of the pieces, never actually concatenated. Reading any
character of it forces V8 to flatten that tree.

Do that after every append and the printer spends its time flattening ropes it is about to extend again.
Result is a **4x slow-down**.

So `oxc-codegen` has to find a different solution. Every time it pushes a string to output, it also records
in `state.last` what the output now ends with. `state.output` never needs to be read from.

### The answer: A category, not a character

It turns out the functions which decide whether spaces need to be inserted between keywords / identifiers
don't actually need to know what the last character was, they only need to know what _category_ of character
it was. i.e. It doesn't matter if the last character was `"a"` or `"Z"` or `"$"` - they're all equivalent.

Almost everywhere in the printer, it knows statically what _category_ it's writing, even if it doesn't know
the exact bytes until runtime. e.g. When writing an identifier, it's the `IDENT` category.

```ts
write(state, node.name, CAT_IDENT);
```

This has 3 advantages over storing the last character:

1. Categories are just small integers, the cheapest data type there is.
2. The categories are statically defined at call sites. No `node.name.at(-1)`.
3. Checking the category of last write is cheap - `last === CAT_IDENT` not `isIdentifierPart(lastChar)`.

That one `last` field replaces all of this:

| Rust                                  | Question it answers                 | JS                       |
| :------------------------------------ | :---------------------------------- | :----------------------- |
| `last_byte()` + `is_identifier_part`  | Would a following identifier merge? | `CAT_IDENT`              |
| `last_byte() == Some(b'?')`           | Would a following `?` make `??`?    | `CAT_QUESTION`           |
| `peek_nth_byte_back(1) == Some(b'<')` | Is this `!` the `!` of a `<!`?      | `CAT_OP_UN_NOT_AFTER_LT` |

### Extending this scheme to operators

Rust `oxc_codegen` also holds state about what operator was last written. It stores this info as a pair:

- `prev_op` - Last operator that was written
- `prev_op_end` - Position of the end of that operator

Each time an operator is written, it sets these 2 fields. When other code wants to know "am I writing this straight
after an operator, and which operator?" it:

1. Checks if `code.len() == prev_op_end`. If true, then yes, it's right after an operator.
2. `prev_op` then says which operator.

In Rust printer, this makes sense. Checking these 2 fields, and comparing to `code.len()` costs 2 or 3 memory accesses,
and 1 or 2 comparisons. But that's fine! It only reads or writes these 2 fields infrequently - only when an operator
is being written. And operators are only a small fraction of code.

In JS codegen, we're in a different situation. We've been forced to update `state.last` after every write,
to avoid the huge perf hit of reading the last character from `output`.

So we might as well use `last` to record operators too, and also some similar character classes:

| Rust                                    | Question it answers                                     | JS                |
| :-------------------------------------- | :------------------------------------------------------ | :---------------- |
| `code.len() == prev_op_end` + `prev_op` | Which operator came last, and was it right before this? | `CAT_OP_*`        |
| `code.len() == need_space_before_dot`   | Is space needed after digit? e.g. `0 .toExponential()`  | `CAT_INT_DIGIT`   |
| `code.len() == prev_reg_exp_end`        | Did a flagless regex just close?                        | `CAT_REGEX_SLASH` |

All these Rust state fields, and every read of the output buffer, collapse into one field in the JS printer.

It costs no write barrier to store (because categories are represented by "SMI" small integers), and one compare
to test. Being touched by every single write, it is the hottest field in `State`, reliably in L1 cache,
and probably often also benefits from fast store-to-load forwarding.

#### The `<!--` case, as an example

Rust decides whether a `!` is the `!` of a `<!--` hazard by peeking at the _second_-to-last byte,
at the moment a `--` is about to be written.

Here the question is answered when the `!` is written, where the preceding character is already known,
and the answer is baked into which category gets stored. The reader has nothing left to look up.

### Position marks

Similar to operators, Rust printer also maintains state about the _context_. For example:

```js
// Function declaration
export default function foo () {}
// Function expression
export default (function bar () {});
```

These 2 are semantically different - `foo` is available in outer scope, `bar` is not.

When printing a `FunctionExpression`, print needs to know "am I the value of an `export default`?"
to know whether it needs to wrap the function in parentheses.

Similar to operators, Rust printer stores this info in field `start_of_default_export`.
When `code.len() == start_of_default_export` then, yes, what's currently being printed is the value of
an `export default` statement.

Again, JS codegen uses `last` to store this info. Except here `last` doesn't describe what _was_ written,
it says what the context is.

Note that the node which reads the mark need not be the direct child of the `export default`.
In `export default (function bar() {})();` the function is the callee of a call expression, and the parens
are still needed. It gets them because `printCallExpression` wrote nothing before descending into the callee,
so the mark was still there when `printFunction` looked. Whatever is printed leftmost sees the mark,
however deeply nested it is.

| Rust                                    | Question it answers                   | JS                            |
| :-------------------------------------- | :------------------------------------ | :---------------------------- |
| `code.len() == start_of_stmt`           | At the start of a statement?          | `CAT_START_OF_STMT`           |
| `code.len() == start_of_arrow_expr`     | At the start of a concise arrow body? | `CAT_START_OF_ARROW_EXPR`     |
| `code.len() == start_of_default_export` | At the start of an `export default`?  | `CAT_START_OF_DEFAULT_EXPORT` |

The reason this works is that in every case the last character printed in these positions was a space, a line break,
or the start of the file, which all fall into the `CAT_OTHER` category. So these values of `last` aren't ambiguous
to any other code which judges what to do next based on what came last. They just treat these values as
"last character written was something I don't need to do anything special about".

#### Why the substitution is safe

Two facts, both checked across every printer file:

1. **Nothing writes output on the way down from a mark to its reader.** Every path from a mark to a
   reader either writes nothing, or writes a real `(` - never a `writeNoLast`, which would leave the
   mark visible past output which should have killed it.
2. **It can only ever add parentheses, never remove them.** A real `write` always grows the output
   (empty writes are illegal), so the set of things which kill a mark under categories is a strict
   subset of what killed the offsets.

So every possible failure is a spurious pair of parentheses - slightly longer output, but not _invalid_ output.
The byte-for-byte conformance tests would catch this, but even if there are gaps in the conformance suite
for crazy edge cases and a bug sneaks through, it can't produce invalid JavaScript.

### Why category numbering is load bearing

The full table is at the top of [`print/write.ts`] and is the authority.

Three properties are relied on. Adding a code without preserving them will silently space output wrongly.

1. **Identifier hazards are the lowest codes.** So `printSpaceBeforeIdentifier` is `last <= CAT_REGEX_SLASH` -
   one compare, no table, no branch tree. The operators `printSpaceBeforeOperator` must distinguish are the highest,
   for the same reason (`last >= CAT_OP_UN_NOT_AFTER_LT`).
2. **The `CAT_START_OF_*` codes sit between those two ranges**, which is what makes both range checks
   treat them as "nothing to separate".
3. **`CAT_START_OF_STMT` is odd, with the other two marks either side.** The five reader sites each ask
   about a _pair_ of marks, never all three, so both pairs fall out as one `|`-and-compare:

   ```
   (last | 1) === CAT_START_OF_STMT          // Statement or `export default`
   ((last - 1) | 1) === CAT_START_OF_STMT    // Statement or concise arrow body
   ```

Each property has an `if (DEBUG)` block which iterates every category and asserts the compare selects
exactly the intended set and nothing else - in [`print/write.ts`] for the `|` identities, and in [`print/space.ts`]
for the two range checks.

Renumber freely. The assertions will tell you what you broke, and cost release builds nothing.

**One deliberate exception**: `CAT_OP_UN_NOT` is an operator code but sits _below_ the operator range.
No operator merges with a plain `!`, so storing it must not cost `printSpaceBeforeOperator` a call into its slow path.

### `writeNoLast`

Not every write needs to update `last`.

When one token is written in pieces - a string's opening quote, its contents, its closing quote -
only the last piece's category can possibly be read. Hence four write primitives in [`print/write.ts`]:

| Function             | Updates `last` | Records a source mapping |
| :------------------- | :------------- | :----------------------- |
| `write`              | Yes            | No                       |
| `writeWithMap`       | Yes            | Yes                      |
| `writeNoLast`        | No             | No                       |
| `writeWithMapNoLast` | No             | Yes                      |

- The `*NoLast` pair is used at 89 sites, against 596 for the other two.
- The rule is exact: **only sound where the value of `last` is provably dead**.
  Another real write must follow before anything reads it.
- JSX carries the longest runs. Printing `<a.b x="1">hi</a.b>` updates `last` exactly once, on the final `>`.
- `writeNoLast` accepts an empty string, `write` does not.
  A `write` of `""` would leave `last` claiming a category for a character that was never written.

### How the invariants are enforced

Every rule above fails quietly - a missing space in one construct out of thousands. So three debug-only
mechanisms guard them, all removed from release builds.

1. **`debugAssertCategoryMatches`** runs on every `write`, checking the category truthfully describes
   the final character written. It cannot be a simple lookup - a category encodes the merge hazard of
   the trailing _token_, not the trailing character - a digit legitimately ends both a `CAT_IDENT` write
   and a `CAT_INT_DIGIT` one - so each final character has a permitted set.
2. **`lastIsStale` / `debugAssertLastFresh`** track the `writeNoLast` rule. `writeNoLast` sets the flag,
   the next real write clears it, and every reader of `last` calls `debugAssertLastFresh` first,
   to make sure they're not reading a stale value.
   An author who was wrong about `last` being dead gets a clear failure, not a wrongly spaced construct.
3. **The numbering assertions** described above.

This is why the debug build is the one to run the conformance suites against - `pnpm run build-test`.

## Four builds from one source tree

`dist` holds the printer compiled four times, plus an entry point which picks between them.

| Build              | `TS`  | `SOURCEMAPS` |  Size |
| :----------------- | :---- | :----------- | ----: |
| `print_js.js`      | false | false        | 25 KB |
| `print_js_maps.js` | false | true         | 26 KB |
| `print_ts.js`      | true  | false        | 39 KB |
| `print_ts_maps.js` | true  | true         | 40 KB |

`index.ts` picks one from the caller's `ts` and `sourcemap` options and `require`s it on first use -
`require` rather than `import()`, because `printSync` is synchronous.

A caller printing only JavaScript never loads, parses or compiles the TypeScript printers at all.

### Why separate TS and JS builds

**The obvious reason is code size and complexity.** JS-only builds have all TS-related code removed,
as it's all gated behind compile-time `TS` constant. Every `if (TS && node.declare) { ... }` block is removed
by minifier, and every `case "TSEnumDeclaration":` arm by the `strip_ts` TSDown plugin. With the arms gone,
the TS-only printer functions lose their last references and are tree-shaken away. That is a third of the code.
However, it has a surprisingly small impact on perf.

**The bigger reason is monomorphism.** `oxc-parser` produces differently-_shaped_ node objects for a
JS-shaped AST and a TS-shaped one - the TS shape carries `optional`, `declare`, `accessibility`,
`override` and the rest.

If both shapes flow through one `printFunction`, V8 sees two hidden classes at every property access,
the inline caches go polymorphic, and the whole printer slows down.

Two builds means each one only ever sees one shape. JS printer sees only JS AST-shaped objects,
TS printer sees only TS AST-shaped objects. This gives a perf boost of over 10%.

Additionally, the split helps branch prediction. If feeding a mix of JS and TS ASTs through a single printer,
for a JS AST branches for "does this node have a type annotation?" always find the answer is "no".
Branch predictor believes there's a pattern and when that code runs again, it expects no type annotation again.
But this time, the AST is a TS AST, and the answer is "yes". Mispredict! Go back 100 instructions, go directly back,
do not pass Go, do not collect $200!

With 2 different builds for JS and TS, this doesn't arise. In the JS build the branch isn't there at all -
it was compiled away. The TS build only ever sees TS ASTs, so what the branch predictor learns stays true.

### Why `State` lives outside `print` directory

The same argument, one level up.

`State` is defined in `src-js/state.ts`, which is **not** part of any printer build. The printers import
it as a type only; the entry point constructs it. So all four builds share one class, and therefore one
hidden class, for the object they thread through every single function.

This is why **a build-time flag must never add or remove a field on `State`**.

- The source-map fields are present in every build, and set to `null` when unused.
- The debug fields are present in every build, and only _initialised_ under `if (DEBUG)`.

Making either conditional would give each build a different object shape, undoing the whole arrangement.

### The build plugins

There are 4 TSDown plugins in `tsdown_plugins`. Order matters - `strip_ts` is a text transform,
so it must run before the plugins which parse.

#### `strip_ts`

Deletes regions fenced with `/* IF TS */` ... `/* END_IF */` from JS-only builds.

They exist because a minifier can fold `if (TS)` away, but cannot remove a `switch` `case` arm on the grounds
that its node type never occurs.

It fails the build if the fences are unbalanced - and also if it finds _no_ fences at all.
A plugin silently doing nothing is a failure mode worth guarding against.

#### `unmap_writes`

Rewrites `writeWithMap(state, code, cat, node)` into `write(state, code, cat)` in non-sourcemap builds,
drops the `node` argument, and rewrites the import to match.

Without it, the node argument would still be evaluated and held live across a call which ignores it.

It re-parses its own output and fails the build if anything still reaches a mapped write, or calls a name
it did not bring into scope.

#### `const_functions`

Rewrites top-level `function foo(...) { ... }` into `const foo = (...) => { ... }`.

A function declaration creates a binding which could be assigned later, so every call has to read
the current value out of module scope. A `const` cannot be, so V8 calls it directly.

- **Measured at ~3%** across all benchmark fixtures.
- It really is the `const`. The same rewrite with `let` measures ~4% _slower_ than this, and slower than
  leaving the declarations alone.
- The source keeps its declarations, because they hoist and read better. Which is why this is a build step
  rather than a house style.

#### `remove_asserts`

Removes `debugAssert` and `typeAssertIs` calls _and the expressions inside them_ from release builds.

The minifier can drop the calls on its own, but cannot prove `node.value` in `debugAssert(node.value !== null)`
has no getter, so it would leave `node.value !== null` behind as a bare expression statement.

This is what gives us Rust `debug_assert!`-style checks in tests and conformance, with zero cost in release build.

## Other things which make it fast

### Popularity-ordered dispatch

`printExpression` (34 arms) and `printStatement` (33 arms) are megamorphic string switches.

V8 lowers a switch on strings to what amounts to an `if / else if` chain, so arm order is execution order.
Both are ordered roughly by how often each node type occurs in real code - common variants like `Identifier`,
`MemberExpression`, `CallExpression` first.

This sped up the printer by ~30%.

It also means **adding an arm near the top of either switch has a cost**, however rare the node type is.

Even after ordering `case` arms, these "dispatch" functions remain a large part of runtime (~25%).
Much sweat and tokens were spilled trying to reduce it further:

- Perfect hash table to convert `node.type` to an integer ID.
- `switch`-ing on an ID to reduce the switch to a jump table.
- Branching on first character to narrow down the range - `switch (node.type.charCodeAt(0)) { case 69: ... }`.
- Many others...

All to no effect. Arm ordering worked, nothing else did.

If anyone else can crack this, it's probably the single largest unrealized perf gain on the table.

### Monomorphism

As much as possible, call paths are structured so that functions receive the same object shape consistently.
Once it has gathered type feedback from the first runs, V8 tiers up hot functions to TurboFan, which generates
code along these lines (pseudo-code):

```ts
function printThing(state, node): void {
  if (SHAPE_OF(node) !== K_EXPECTED_SHAPE) {
    DE_OPTIMIZE(printThing);
    return printThing(state, node);
  }

  // Now it's known exactly what shape `node` is, it becomes static-typed like Rust.
  // Working with it is very cheap.

  // Original code: `const { left } = node;`
  const left: Expression = GET_UNCHECKED(node, K_LEFT_OFFSET);
  // ... fast code from here on ...
}
```

In particular, it is **essential** that `State` remains the same shape _everywhere_. Every function touches it,
so any inconsistency in shape would turn every access polymorphic and completely tank performance.

The obvious exception is the dispatch functions discussed above (`printExpression` etc), which are
unavoidably megamorphic.

We have not done a full audit of all functions to check if any are receiving polymorphic/megamorphic inputs
when that could be avoided. Such an audit might throw up potential improvements.

#### Moral of the story

Consistent object shapes are _really_ important for good perf in JS engines.

We should provide users with utility functions to construct AST nodes which produce the exact same object shape
as what `oxc-parser` produces. This would speed up not just `oxc-codegen`, but also their own code that
operates on ASTs.

```ts
// Good
const binary = oxc.binaryExpression(left, operator, right);
// Bad - different field order, different object shape
const binary = { left, right, operator };
```

### Intended parser pairing

The printer is meant to be used with `oxc-parser` with `experimentalRawTransfer` enabled.

Raw transfer builds the AST with a JS deserializer rather than via JSON. Node `type` strings then come out
as source literals, so those big `switch`es compare interned strings by pointer, rather than character by character.
This gives a 10% - 30% perf boost versus a JSON AST.

### Fast paths

- Pretty mode always puts a single space either side of a binary or assignment operator, so the whole
  token is one constant string - `" + "`, `" ??= "` - and one `write`, instead of space-operator-space.
  It also makes the token-glue checks unobservable around those operators, so they are not made at all.
  See `PADDED_BIN_OPERATORS` in [`print/operators.ts`].
- `printNonNegativeFloat` prints integers below 1000 straight from `String()`. They are the overwhelming
  majority, and three digits are too few for any shortening form to win.
- `printString` tests the whole string against one regex of characters needing escapes, and appends it
  whole when there are none. The escaping loop is a separate function because it almost never runs.
- `printSpaceBeforeOperator` is one compare, with its three-clause slow path split out so the common path
  stays small enough for V8 to inline.
- `printLiteral` switches on `typeof value` first, identifying strings and numbers without touching the
  `regex` or `bigint` properties.
- `printNumericLiteral` tests `value > 0 && value < Infinity` first, reaching the same branch the full
  chain would without the two builtin calls.

Each of these produced a small perf bump individually, ~10% gain in aggregate.

### Indent cache

`printIndent` reads a pre-built string for the current level out of an array.

- The cache is created once per _process_, alongside the module-level state in `state.ts`.
  Every `State` carries a reference to it. So indent strings one printer run has grown are there for the next.
  This cache is shared between JS and TS, sourcemap and no-sourcemap builds.
- `growIndents` deliberately forces each new indent string flat (a throwaway `charCodeAt(0)`) before caching it,
  paying the flatten cost once rather than on every append.

### Source maps computed at the end

During printing, a mapped write records only an output offset and the node's original UTF-16 offset
from `start` / `end`. Nodes without offsets are not mapped, and `sourceText` is required to convert
source offsets to line/column positions.

`generateSourceMap` then walks the output once at the end, counting ECMAScript line terminators,
builds the equivalent line table for `sourceText`, turns both sets of offsets into line/column,
and encodes the mappings as base64 VLQ. Tracking lines and columns throughout would cost every write
in every build.

#### Potential future improvement

Producing source maps is exactly the kind of raw number-crunching that native code excels at.
And, unlike the AST, it's just small integers - the kind of data that transfers across the JS-native
boundary cheaply.

It might be worthwhile assembling all the offsets on JS side in an `Int32Array`, instead of a plain `Array`,
and making a single JS-Rust-JS round-trip, with the sourcemap generation (VLQ encoding etc) happening on Rust side.

WASM might also be a good option for this.

### No separate stack in the binary walk

`printBinaryish` walks the left spine of a binary/logical chain iteratively. `a + b + c + d` nests as deep
as it is long, so recursion would put the stack at the mercy of the input.

Rust does the same, but with an explicit `Stack` on `Codegen`. Here each level has a small visitor
object, and pending outer levels are threaded through its `parent` field rather than a separate stack array.

### Lookup tables are `__proto__: null`

So a property miss cannot walk up to `Object.prototype`.

## Where the ESTree AST differs

The two printers walk different ASTs, and some of the differences reach the output.

### Node types

- **One `Literal` node type** covers strings, numbers, booleans, `null`, regexes and bigints, where Rust
  has six separate types. `printLiteral` tells them apart from `typeof value` and the presence of `regex` / `bigint`.
- **`#x in obj` arrives as a `BinaryExpression`** with a `PrivateIdentifier` on the left, not as its own node type.
  Hence the extra test in `printExpression`'s `BinaryExpression` arm.

### Parenthesized function expressions

Oxc's Rust AST records that a function expression was parenthesized in a `pife` flag, and prints the parentheses
back from it.

ESTree has no such flag. The information is only there when the AST was parsed with `preserveParens`,
as a `ParenthesizedExpression` wrapper. `printExpression`'s arm for that node re-emits the parentheses
around a function or arrow, and is transparent around anything else.

This is why `printBinaryish` prints its operands **unstripped**. `withoutParens` is used for
the structural type tests, but the node handed to `printExpression` keeps its wrapper,
so that arm can run.

### Things ESTree cannot express

- `import "m";` versus `import {} from "m";`
- An absent `with` clause versus an empty `with {}`
- `assert {...}` versus `with {...}`
- The span of a `with {...}` clause (only its individual attributes have ESTree locations)

The conformance harness normalizes the Rust AST down to what ESTree can express before printing,
rather than expecting the JS side to reproduce information it was never given.
See `Normalize` in `tasks/codegen_conformance/src/lib.rs`.

### Positions, and other producers

- **Oxc offsets only.** `start` / `end` offsets are converted to UTF-16 line/column once at the end.
  Source maps require `sourceText`; `loc` and `range` are not supported inputs.

## How it is tested

`oxc-codegen` is tested against all Test262, Acorn-JSX, and TypeScript test cases - about 62,000 fixtures.

Every fixture is printed three times:

1. In Rust, with a source map, via the `oxc-codegen-conformance` NAPI addon (`tasks/codegen_conformance`).
2. In JS, through the no-maps build.
3. In JS, through the separately compiled maps build.

All three generated outputs must agree byte for byte. Every decoded generated/original line, column,
and optional original name from the maps build must also agree with Rust, including ordering and
duplicate suppression.

Both sides parse the same source text with the same `SourceType`, derived by the same function.
Fixtures which do not parse cleanly are skipped, rather than passing quietly.

Every fixture is checked in **both `preserveParens` modes**. They are different paths through the printer -
with the wrappers present it must decide what to re-emit, without them it must re-derive every parenthesis
from precedence.

## Rules to keep

A checklist, all of it argued for above.

1. **Never read a character of `state.output`.**
2. **A `write` must declare a category which truthfully describes its final character**, and must never be
   given an empty string.
3. **Only use `writeNoLast` where `last` is provably dead.** A real write must follow before any reader runs.
4. **Preserve the three numbering properties** when adding or renumbering a `CAT_*` code, and keep the
   `if (DEBUG)` assertions which check them.
5. **Never let a build-time flag add or remove a field on `State`.**
6. **Do not add an arm near the top of `printExpression`, `printStatement` or other dispatch functions**
   unless the node type really is that common.
7. **Do not relax `const_functions` to `let`.**
8. **Run the conformance suites against a debug build** (`pnpm run build-test`), so the assertions are live.
9. **Benchmark against a release build** (`pnpm run bench` rebuilds one first). A debug build keeps
   `debugAssert` calls and does not represent shipped performance.

[`oxc_codegen`]: https://github.com/oxc-project/oxc/tree/main/crates/oxc_codegen
[`print/write.ts`]: https://github.com/oxc-project/oxc/blob/main/packages/codegen/src-js/print/write.ts
[`print/space.ts`]: https://github.com/oxc-project/oxc/blob/main/packages/codegen/src-js/print/space.ts
[`print/operators.ts`]: https://github.com/oxc-project/oxc/blob/main/packages/codegen/src-js/print/operators.ts
[`print/types.ts`]: https://github.com/oxc-project/oxc/blob/main/packages/codegen/src-js/print/types.ts
