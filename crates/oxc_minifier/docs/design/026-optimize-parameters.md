# Optimize Parameters

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Remove trailing unused function parameters and unused default parameter values. After mangling, parameter names are single characters, so removing unused trailing parameters saves at least 2 bytes each (`,a`). Removing unused defaults saves the default expression bytes.

## Why

Functions often declare parameters they don't use — callback signatures with unused later arguments, or functions that evolved over time. Trailing unused parameters are dead weight. This pass is especially effective after inlining and DCE, when call sites have been simplified but parameter lists haven't been updated.

## Transformations

### Remove trailing unused parameters

```js
// Before
function f(a, b, unused1, unused2) {
  return a + b;
}

// After
function f(a, b) {
  return a + b;
}
```

### Preserve non-trailing unused parameters

Parameters that precede used ones cannot be removed (positional calling convention):

```js
// Before
function f(unused, b) {
  return b;
}

// After (unchanged — cannot remove `unused` without breaking call sites)
function f(unused, b) {
  return b;
}
```

### Remove unused default values

```js
// Before
function f(a, b = expensive()) {
  return a;
}

// After
function f(a) {
  return a;
}
```

### Safety constraints

- Only remove parameters from the end of the parameter list
- All call sites must be visible — if a function escapes (exported, passed as callback, stored in object), parameters cannot be removed
- Must not remove parameters from functions that access `arguments` (would change `arguments.length`)
- Must not remove parameters from functions with rest parameters that depend on position
- Default value removal requires verifying the default expression's side effects are not needed

## References

- Closure: `OptimizeParameters.java` — removes unused params; `OptimizeCalls.java` — removes unused arguments from call sites
- Terser: `keep_fargs: false` (default: `true`) — drops unused trailing parameters
- esbuild: Does not implement parameter removal
- SWC: `keep_fargs: false` mirrors Terser behavior
