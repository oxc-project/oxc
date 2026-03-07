# Peephole Remove Dead Code

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

This pass removes code that is provably unreachable or has no observable effect. It operates locally on small AST neighborhoods — "peephole" style — without requiring whole-program analysis.

## Why

Dead code inflates bundle size and obscures other optimization opportunities. Removing it early lets subsequent passes work on a smaller, cleaner AST and may expose further simplifications (e.g., an `if` whose body was the only reference to a variable).

## Transformations

### Fold constant conditionals

When the condition of an `if`, ternary, or logical expression is a known constant, replace the entire node with the taken branch (preserving side effects of the discarded branch if any).

```js
// Before
if (true) { a(); } else { b(); }

// After
a();
```

```js
// Before
if (false) { a(); }

// After
// (removed entirely)
```

```js
// Before
var x = true ? a : b;

// After
var x = a;
```

### Remove side-effect-free expression statements

An expression statement whose value is never used and that produces no side effects can be removed.

```js
// Before
"use strict";
42;
x;  // just a read, no side effect in sloppy mode
a();

// After
"use strict";
a();
```

### Remove code after unconditional exits

Statements after `return`, `throw`, `break`, or `continue` in the same block are unreachable.

```js
// Before
function f() {
  return 1;
  console.log("unreachable");
  var x = 2;
}

// After
function f() {
  return 1;
}
```

### Fold `do-while(false)`

A `do { ... } while (false)` executes its body exactly once. Replace it with the body statements directly (adjusting for `break`/`continue` labels).

```js
// Before
do { a(); b(); } while (false);

// After
a();
b();
```

### Simplify `try`/`catch`/`finally`

When a `try` block cannot throw (contains only simple statements), the `catch` block is dead. When a `finally` block is empty, it can be removed.

```js
// Before
try { x = 1; } catch (e) { handle(e); }

// After
x = 1;
```

### Remove empty statements and blocks

Empty statements (`;`), empty blocks (`{}`), and control structures with empty bodies are removed.

```js
// Before
if (x) {}
;
for (;;) break;

// After
// (all removed, assuming no side effects in conditions)
```

### `var` hoisting in dead code

`var` declarations must be preserved even in dead code because they hoist. The initializer is removed but the declaration is kept.

```js
// Before
if (false) { var x = 1; }

// After
var x;
```

### Function and class declarations in dead code

Function and class declarations need different handling.

- Function declarations may need to be preserved when hoisting or Annex B block-function semantics make them visible outside the dead branch.
- Class declarations are block-scoped and do not hoist like `var`; they can only be removed when dropping the dead block preserves block-scope semantics.

## References

- `PeepholeRemoveDeadCode.java`
- `js_parser.go:8747,9077,10184`
- `compress/tighten-body.js:1209`, `compress/index.js:1088`
