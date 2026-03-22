# Modern Syntax Optimizations

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Exploit ES2020+ syntax features to produce shorter output. These optimizations are only applied when the target environment supports the required syntax.

## Why

Modern JavaScript syntax provides more concise ways to express common patterns. When the output target supports these features, using them can save bytes without changing semantics. These optimizations are gated by the configured target environment.

## Transformations

### Negate IIFE

Wrap IIFEs with `!` instead of parentheses to save one byte.

```js
// Before (2 bytes for parens)
(function () {
  /* ... */
})();

// After (1 byte for !)
!(function () {
  /* ... */
})();
```

### Optional chaining insertion

Replace null/undefined guard patterns with optional chaining (`?.`).

```js
// Before
a != null && a.b;
a && a.b && a.b.c;

// After
a?.b;
a?.b?.c;
```

### Nullish coalescing optimization

Replace explicit null/undefined checks with `??`.

```js
// Before
a !== null && a !== undefined ? a : b;

// After
a ?? b;
```

### Spread evaluation

Evaluate spread operations on known literals at compile time.

```js
// Before
({...{a: 1, b: 2}})
foo(...[1, 2, 3])
[...[1, 2], 3]

// After
({a: 1, b: 2})
foo(1, 2, 3)
[1, 2, 3]
```

### Built-in object shortening

Replace constructor calls with shorter literal equivalents.

```js
// Before
new Array()
new Object()
new RegExp("pattern", "gi")
new Array(1, 2, 3)

// After
[]
({})
/pattern/gi
[1, 2, 3]
```

Note: `new Array(n)` with a single numeric argument cannot be replaced with `[n]` as it creates a sparse array of length `n`.

### Logical assignment operators

Replace self-assignment patterns with logical assignment operators (ES2021).

```js
// Before
x = x ?? y    →  x ??= y
x = x && y    →  x &&= y
x = x || y    →  x ||= y
```

Gated by target environment supporting ES2021+. Refs: SWC logical assignment optimization; Terser `logical` option.

## References

- esbuild: `TryToInsertOptionalChain` in `js_parser.go`
- Terser: `negate_iife` option, optional chaining in `compress`
- SWC: `misc.rs` — spread folding, IIFE negation
