# Variable Mangling

- **Status:** Implemented
- **Difficulty:** Medium

## What

Rename local variables (and optionally global/exported variables) to the shortest possible names. This is typically the single largest size reduction in minification.

## Why

JavaScript identifiers are often long and descriptive (`getUserProfile`, `handleSubmit`, `isAuthenticated`). Since local variable names have no observable effect on program behavior, they can be replaced with minimal names (`a`, `b`, `c`, ...) to dramatically reduce output size.

Frequency-based assignment — giving the shortest names to the most-referenced variables — maximizes the savings. Gzip-aware scope reuse further improves compressed size by creating repetitive byte patterns.

## Transformations

### Basic variable renaming

```js
// Before
function calculateTotal(itemPrice, taxRate, discount) {
  const subtotal = itemPrice * (1 - discount);
  const tax = subtotal * taxRate;
  return subtotal + tax;
}

// After
function calculateTotal(a, b, c) {
  const d = a * (1 - c);
  const e = d * b;
  return d + e;
}
```

### Frequency-based assignment

Sort symbols by reference count descending. Assign names from a Base54 alphabet (valid JS identifier start characters) so the most-used variables get single-character names.

```js
// "data" referenced 50 times → "a"
// "config" referenced 30 times → "b"
// "temp" referenced 2 times → "aA" (if single chars exhausted)
```

### Frequency-sorted Base54 alphabet

Scan the source for character frequency. Sort the 54 identifier-start characters and 64 identifier-continuation characters so the most common characters in the source get the earliest positions. This improves gzip compression because the mangled names share characters with surrounding code.

Typical frequency order: `etnriaoscludfpmhg_vybxSCwTEDOkAjMNPFILRzBVHUWGKqJYXZQ$1024368579`

First character uses base-54 (valid identifier starts), subsequent characters use base-64 (includes digits). Maximum name length: 6 characters for u32 range.

### Slot-based renaming algorithm

1. Compute "liveness" for each symbol — the set of scopes where the symbol is referenced
2. Graph coloring: assign symbols to numbered slots, reusing a slot when its liveness doesn't intersect with the symbol's (greedy first-fit)
3. Sibling scopes naturally share slots since their liveness sets don't overlap
4. Precompute ancestor BitSets for O(1) scope membership testing during liveness intersection checks

### Gzip-aware sibling scope reuse

Variables in non-overlapping scopes share the same slot index, so they receive the same mangled name. This creates repeated character sequences that compress well under gzip/brotli.

```js
// Before
function foo() {
  let longName1 = 1;
}
function bar() {
  let longName2 = 2;
}

// After — both get "a" since scopes don't overlap
function foo() {
  let a = 1;
}
function bar() {
  let a = 2;
}
```

### Scope nesting and shadowing

Inner scopes can shadow outer variable names. The mangler exploits this to reuse short names at every scope level.

```js
// Before
function outer() {
  const outerVar = 1;
  function inner() {
    const innerVar = 2;
    return innerVar;
  }
  return outerVar + inner();
}

// After
function outer() {
  const a = 1;
  function inner() {
    const a = 2; // shadows outer "a" — safe because inner doesn't reference it
    return a;
  }
  return a + inner();
}
```

### Label mangling

Labels have their own namespace and can be renamed independently using the same Base54 alphabet.

```js
// Before
outer: for (...) { inner: for (...) { continue outer; } }

// After
a: for (...) { b: for (...) { continue a; } }
```

Refs: esbuild label symbols; Terser `mangle.label`.

### Eval safety

When `eval()` is present in a scope, all variables in the containing scope chain must retain their original names. This disables mangling for affected scopes because `eval` can reference any variable by name at runtime.

Refs: esbuild direct eval deoptimization; Terser `eval` option.

### `keep_names` handling

`keep_names` is primarily a renaming constraint, not a wrapper rewrite. The mangler reserves symbols whose bindings determine the `.name` of functions/classes, and compressor passes must avoid removing function/class names when the corresponding `keep_names` option is enabled.

This matters both for declarations and for anonymous expressions whose `.name` comes from the binding:

```js
// Before
var longName = function () {};

// With keep_names enabled
var longName = function () {};
// Not:
// var a = function() {};
```

## References

- Closure Compiler: `RenameVars.java`
- esbuild: `renamer.go` — slot-based renaming with sibling scope reuse
- Terser: `lib/scope.js` — `mangle()`, `base54`
- SWC: `mangle_names/` + `base54.rs`
