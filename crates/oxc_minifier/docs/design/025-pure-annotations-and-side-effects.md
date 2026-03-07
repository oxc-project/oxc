# Pure Annotations and Side Effects

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Recognize `/*@__PURE__*/` and `/*#__PURE__*/` annotations on call expressions and mark them as side-effect-free. When the result of an annotated call is unused, DCE can safely remove the entire call. Also support `/*@__NO_SIDE_EFFECTS__*/` on function declarations to treat all calls to that function as pure.

Additionally, support configuration-based pure function lists (`pure_funcs`) and a `pure_getters` flag that assumes property access has no side effects.

## Why

Many libraries wrap factory functions in IIFEs or call helper functions whose results may go unused. Without annotations, the minifier must conservatively assume these calls have side effects and keep them. Pure annotations let library authors and users communicate intent, enabling significantly better dead code elimination — especially for tree-shaking unused exports.

## Transformations

### Remove unused pure-annotated call

```js
// Before
var x = /*@__PURE__*/ createComponent();

// After (x is unused)
// (removed entirely)
```

### Remove unused IIFE with pure annotation

```js
// Before
var Foo = /*@__PURE__*/ (function () {
  function Foo() {}
  Foo.prototype.method = function () {};
  return Foo;
})();

// After (Foo is unused)
// (removed entirely)
```

### `@__NO_SIDE_EFFECTS__` on function declarations

```js
// Before
/*@__NO_SIDE_EFFECTS__*/
function helper(x) {
  return x + 1;
}
var y = helper(2);

// After (y is unused)
// (removed entirely)
```

### `pure_funcs` configuration

When `pure_funcs: ["Math.floor"]` is configured:

```js
// Before
var x = Math.floor(3.14);

// After (x is unused)
// (removed entirely)
```

### `pure_getters` flag

When `pure_getters: true` is set, property access chains are assumed side-effect-free, allowing removal of unused access expressions:

```js
// Before
a.b.c;

// After (result unused, pure_getters enabled)
// (removed entirely)
```

### Auto-detected pure constructors

Some `new` expressions can be marked `@__PURE__` automatically, but only after constructor-specific checks. The callee must resolve to the standard global constructor, the arguments must be side-effect-free, and the invocation must be proven non-throwing for the given arity/value types.

Examples include selected zero/one-argument cases for `WeakMap`, `WeakSet`, `Map`, `Set`, `Date`, `ArrayBuffer`, boxed primitives, the standard `Error` constructors, `DataView`, and validated `RegExp` construction.

This is intentionally narrower than "all built-in constructors are pure": constructors such as `Promise` or arbitrary typed-array forms require additional analysis and should not be blanket-annotated.

### Safety constraints

- Only remove annotated calls when the result is provably unused
- The annotation must appear immediately before the call expression
- `pure_getters` is unsafe by default — getters can have side effects via `Object.defineProperty`
- `pure_funcs` is user-provided and trusted without verification

## References

- Closure: `PureFunctionIdentifier.java` — infers purity from `@nosideeffects` JSDoc annotations
- Terser: `pure_funcs` option (list of known-pure functions); `pure_getters` option; `/*@__PURE__*/` support
- esbuild: `--pure:name` flag; `/*@__PURE__*/` and `/*#__PURE__*/` annotations; `--drop` for side-effect-free removal
- SWC: `compress.pure_funcs`; `/*@__PURE__*/` support; `pure_getters` option
- Rollup: Originated `/*#__PURE__*/` annotation convention for tree-shaking
