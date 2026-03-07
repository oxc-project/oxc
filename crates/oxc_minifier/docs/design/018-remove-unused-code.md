# Remove Unused Code

- **Status:** Not Implemented
- **Difficulty:** Medium

## What

This pass performs mark-and-sweep elimination of unreferenced declarations. It identifies variables, functions, classes, and properties that are never read and removes their declarations entirely. Unlike dead-assignment elimination (which works on individual assignments via dataflow), this pass works at the declaration level across the entire scope tree.

## Why

Bundled JavaScript often includes library code, polyfills, and utility functions that are imported but never used. Removing these unreferenced declarations can yield significant size reductions — sometimes eliminating entire modules' worth of dead code.

## How It Works

1. **Mark** — walk all expressions and statements, recording every identifier that is _read_ (not just assigned). Build a set of referenced names.
2. **Sweep** — walk all declarations. If a declaration's binding is not in the referenced set, remove it.
3. **Iterate** — removing a declaration may cause other declarations to become unreferenced (if the removed code was the only reader). Repeat until no more removals are possible (fixed-point).

RHS expressions are traversed lazily: if a variable's name is unreferenced, its initializer is only checked for side effects, not fully traversed for references. This prevents a chain of unused variables from keeping each other alive.

## Transformations

### Remove unused variables

```js
// Before
var used = 1;
var unused = expensiveComputation();
console.log(used);

// After
var used = 1;
expensiveComputation(); // kept for side effects
console.log(used);
```

If the initializer is side-effect-free, the entire declaration is removed:

```js
// Before
var used = 1;
var unused = 42;
console.log(used);

// After
var used = 1;
console.log(used);
```

### Remove unused functions

```js
// Before
function usedFn() {
  return 1;
}
function unusedFn() {
  return 2;
}
console.log(usedFn());

// After
function usedFn() {
  return 1;
}
console.log(usedFn());
```

### Remove unused classes

```js
// Before
class Logger {
  log(msg) {
    console.log(msg);
  }
}
class UnusedHelper {
  help() {}
}
new Logger().log("hi");

// After
class Logger {
  log(msg) {
    console.log(msg);
  }
}
new Logger().log("hi");
```

### Cascade removal

Removing one declaration may make others unreferenced.

```js
// Before
function helper() {
  return util();
}
function util() {
  return 42;
}
// neither helper nor util is called

// After
// (both removed)
```

### Preserve side effects in initializers

When an unused declaration has a side-effectful initializer, the initializer is kept as a bare expression.

```js
// Before
var unused = registerPlugin("foo");

// After
registerPlugin("foo");
```

### Remove unused prototype assignments

Assignments to prototypes of unreferenced constructors are also removed.

```js
// Before
function Unused() {}
Unused.prototype.method = function () {};

// After
// (both removed)
```

### Destructuring edge cases

Empty destructuring patterns can still have observable effects:

```js
// Kept — triggers Symbol.iterator:
const [] = iterable;

// Can be removed — no observable effect for plain objects:
const {} = obj;
```

### Private member removal

Unused private class fields, methods, and accessors can be removed. However, removal must be skipped in scopes containing `eval()` since eval can reference private members dynamically via `#member in obj` syntax.

### Remove unused destructured bindings

When only some bindings from a destructuring pattern are used, remove the unused ones. ([esbuild #3276](https://github.com/evanw/esbuild/issues/3276), [Terser #105](https://github.com/terser/terser/issues/105))

```js
// Before
const { a, b, c } = obj;
use(a, c);

// After
const { a, c } = obj;
use(a, c);
```

### Noop function call elimination

When a function body is empty or only `return;`, calls to it can be removed if the return value is unused. ([esbuild #3676](https://github.com/evanw/esbuild/issues/3676))

```js
// Before
function noop() {}
noop();

// After
// (removed)
```

### Static property cascade removal

Extend prototype assignment cascade to static properties. When a constructor/class is unused, its static property assignments are also dead. ([Terser #776](https://github.com/terser/terser/issues/776))

```js
// Before
function Unused() {}
Unused.propTypes = {
  /* ... */
};
Unused.defaultProps = {
  /* ... */
};

// After
// (all removed)
```

### Side-effect-free constructor instances

Unused `new` expressions for classes with no side-effect constructors can be removed. Unused private methods can also be dropped. ([SWC #11320](https://github.com/swc-project/swc/issues/11320), [esbuild #771](https://github.com/evanw/esbuild/issues/771))

### Empty class constructors

Empty constructors or constructors that only call `super(...args)` can be removed. Cross-reference with 033-optimize-calls.md. ([SWC #9727](https://github.com/swc-project/swc/issues/9727))

## References

- `RemoveUnusedCode.java`
- `linker/linker.go:3141`, `js_ast_helpers.go:2432`
- `compress/drop-unused.js:112`
