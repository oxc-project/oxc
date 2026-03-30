# Extract Prototype Member Declarations

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Merge sequences of `Foo.prototype.x = ...; Foo.prototype.y = ...;` into a compound assignment form that avoids repeating the `Foo.prototype.` prefix.

## Why

Pre-class code and transpiled class output frequently produce long sequences of prototype assignments:

```js
Foo.prototype.method1 = function() { ... };
Foo.prototype.method2 = function() { ... };
Foo.prototype.method3 = function() { ... };
```

The repeated `Foo.prototype.` prefix is redundant and compresses poorly. Extracting it into a single reference saves bytes proportional to the number of methods.

## Transformations

### Extract to temporary variable

```js
// Before
Foo.prototype.a = function () {
  return 1;
};
Foo.prototype.b = function () {
  return 2;
};
Foo.prototype.c = function () {
  return 3;
};

// After
var _p = Foo.prototype;
_p.a = function () {
  return 1;
};
_p.b = function () {
  return 2;
};
_p.c = function () {
  return 3;
};
```

### Extract to comma expression (IIFE context)

When inside an IIFE or module scope where a temporary variable is free:

```js
// Before
Foo.prototype.a = 1;
Foo.prototype.b = 2;

// After (alternative form — single expression)
var _p = Foo.prototype;
((_p.a = 1), (_p.b = 2));
```

### Safety constraints

- All assignments in the sequence must target the same prototype object
- The prototype object expression must be side-effect-free (a simple member expression chain)
- No intervening statements that could modify the prototype chain between assignments
- The sequence must have at least 3 assignments to be profitable (the temporary variable declaration costs bytes)

## References

- Closure Compiler: `ExtractPrototypeMemberDeclarations.java`
- Terser: does not implement this optimization
- esbuild: does not implement this optimization
