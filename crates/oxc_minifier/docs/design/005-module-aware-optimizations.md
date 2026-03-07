# Module-Aware Optimizations

- **Status:** Not Implemented
- **Difficulty:** Trivial

## What

Optimizations enabled when input is known to be an ES module. Module semantics provide guarantees that unlock additional transformations not available in script mode.

## Why

ES modules enforce strict mode, have lexical top-level scope, and define `this` as `undefined` at the top level. These guarantees eliminate categories of runtime checks and enable more aggressive optimizations. When the minifier knows the input is a module, it can safely apply these without risk.

## Transformations

### Assume strict mode

Skip `"use strict"` directives and enable strict-mode-only optimizations throughout the module.

```js
// Before
"use strict";
function f() { "use strict"; return this; }

// After
function f() { return this; }
```

### Top-level `this` is `undefined`

In modules, top-level `this` is always `undefined`. Fold comparisons and eliminate dead branches.

```js
// Before
if (this !== undefined) { setup(this); }

// After
// (removed — condition is always false)
```

```js
// Before
var self = this || {};

// After
var self = {};
```

### Merge adjacent exports

Combine consecutive export declarations into a single statement.

```js
// Before
export { a };
export { b };
export { c };

// After
export { a, b, c };
```

### Top-level declarations are block-scoped

All top-level declarations in modules are lexically scoped — no accidental globals. This enables more aggressive variable inlining and dead code elimination at the top level, equivalent to setting `toplevel: true`.

## References

- Terser: `module: true` sets `directives["use strict"] = true` and `toplevel = true`
- SWC: `module: true` sets `in_strict: true` for all expression contexts
- esbuild: Module mode enables top-level const inlining and tree shaking
- Closure: No direct equivalent; assumes strict via `--language_out`
