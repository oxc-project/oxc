# Normalize

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

AST normalization rewrites the program into a canonical form before optimization passes run. It does not change program behavior or reduce size directly — it simplifies the AST so that downstream passes can pattern-match against fewer syntactic variations.

## Why

JavaScript has many ways to express the same thing: `while` vs `for`, comma-separated `var` declarations vs individual ones, compound assignments vs expanded forms. Without normalization, every optimization pass must handle all variants independently, increasing complexity and the chance of missed optimizations.

By canonicalizing the AST up front, each subsequent pass sees a uniform structure and can focus on semantic transformations rather than syntactic variations.

## Transformations

### Split variable declarations

Separate multi-binding declarations into one declaration per variable, making each independently analyzable.

```js
// Before
var a = 1, b = 2, c = 3;

// After
var a = 1;
var b = 2;
var c = 3;
```

### Convert `while` to `for`

Rewrite `while` loops as `for` loops so that all loops use a single construct.

```js
// Before
while (condition) { body(); }

// After
for (; condition;) { body(); }
```

### Extract `for`-loop initializers

Pull complex initializers out of `for` headers into preceding statements.

```js
// Before
for (var x = expensive(); x < 10; x++) { use(x); }

// After
var x = expensive();
for (; x < 10; x++) { use(x); }
```

### Hoist function declarations

Move function declarations to the top of their scope, matching runtime hoisting semantics and simplifying scope analysis.

```js
// Before
a();
function a() { return 1; }

// After
function a() { return 1; }
a();
```

### Remove duplicate variable declarations

When a `var` is declared more than once in the same scope, keep only the first (or the one with an initializer).

```js
// Before
var x = 1;
var x;

// After
var x = 1;
```

### Expand compound assignments (for analysis)

Internally rewrite compound assignments so that optimization passes can reason about the read and write separately.

```js
// Before
x += 2;

// After (internal representation for analysis)
x = x + 2;
```

After optimization passes complete, the codegen phase may re-collapse these back into compound form if it is shorter.

### Remove `ParenthesizedExpression` nodes

Parenthesized expressions carry no semantic meaning after parsing — operator precedence is already encoded in the AST structure. Remove them so downstream passes don't need to handle wrapped variants.

### Convert `Infinity`/`NaN` to raw numeric values

Replace `Infinity`, `NaN`, and `Number.NaN` with their raw numeric representations early so that constant folding can operate on them directly.

```js
// Before
Infinity   →  1/0
NaN        →  0/0
Number.NaN →  0/0
```

### Mark known pure constructors

Annotate known side-effect-free constructors with `@__PURE__` so DCE can remove them when unused: `new WeakMap()`, `new WeakSet()`, `new Map()`, `new Set()`, `new Promise(r => r())`.

## References

- `Normalize.java`
- `js_parser.go:14391`, `js_printer.go` (`EBoolean`, `printUndefined`)
- `compress/index.js:734` (`hoist_declarations`), `compress/index.js:870` (`hoist_properties`)
