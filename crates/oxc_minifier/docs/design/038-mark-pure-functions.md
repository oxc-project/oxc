# Auto-Detect Pure Functions

- **Status:** Not Implemented
- **Difficulty:** Medium

## What

Analyze function bodies to automatically determine purity (no observable side effects) without requiring `@__PURE__` or `@__NO_SIDE_EFFECTS__` annotations. A function is pure if it does not modify external state: no writes to outer-scope variables, no property mutations on non-local objects, no I/O, no thrown exceptions (or only thrown on invalid input that dead code elimination already handles).

## Why

Design 025 (Pure Annotations and Side Effects) handles user-provided purity markers. But most pure functions in real code are not annotated. Auto-detection bridges this gap, enabling dead code elimination to remove calls to pure functions whose return values are unused:

```js
function square(x) {
  return x * x;
}
function createPoint(x, y) {
  return { x, y };
}

square(5); // unused result — can be removed if square is known pure
createPoint(1, 2); // unused result — can be removed if createPoint is known pure
```

Without auto-detection, these calls are retained because the minifier conservatively assumes they might have side effects.

## Conceptual Dependencies

This design depends on two shared concepts:

- **Effect reasoning** — purity is a specialized form of effect classification, not a
  standalone syntax check
- **Light interprocedural analysis** — once a function calls another function, purity becomes
  a dependency problem across local callees instead of a property of one body in isolation

## How It Works

1. For each function, analyze its body for side effects:
   - **No writes to outer-scope variables** — only writes to local variables and parameters
   - **No property mutations on non-local objects** — `this.x = 1` is a side effect unless `this` is a locally-constructed object
   - **No calls to impure functions** — calls to other functions are impure unless those functions are themselves proven pure (requires fixed-point analysis or bottom-up call graph traversal)
   - **No I/O** — no `console.*`, DOM access, network calls, etc.
   - **No thrown exceptions** — or only deterministic throws on inputs that callers control

2. Build a dependency graph: function A calls function B, so A's purity depends on B's purity. Process in reverse topological order (leaves first).

3. Mark proven-pure functions in a side-effect-free set, which dead code elimination and tree shaking can then query.

### Limitations

- **Megamorphic call sites** — if a function is called through a variable that could point to multiple functions, purity is unknown
- **Getters/setters** — property access can trigger arbitrary code; conservative analysis must assume property access is impure unless the object's shape is known
- **Recursion** — mutually recursive functions require fixed-point iteration over the purity lattice
- **`eval` and `with`** — disable analysis for affected scopes

## References

- Closure Compiler: `PureFunctionIdentifier.java` — global purity analysis that marks functions as side-effect-free
- Design 025 (Pure Annotations and Side Effects) — complementary pass that handles user-provided annotations
- esbuild: uses `@__PURE__` annotations only, no auto-detection
- Terser: uses `pure_funcs` option and `@__PURE__` annotations, no auto-detection
