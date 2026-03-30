# Inline Simple Methods

- **Status:** Not Implemented
- **Difficulty:** Simple-Medium

## What

Inline methods whose bodies are trivial — they return a property, return a parameter, or return a constant. These accessor-like methods are common in class-based and object-oriented code, and their call overhead can be eliminated entirely.

## Why

Class-based JavaScript frequently wraps property access in getter methods:

```js
class Point {
  getX() {
    return this.x;
  }
  getY() {
    return this.y;
  }
}
```

Each call to `p.getX()` introduces a function call for what is semantically just `p.x`. Inlining these trivial methods reduces code size and eliminates call overhead.

## Transformations

### Return a property of `this`

```js
// Before
Foo.prototype.getBar = function () {
  return this.bar;
};
x = obj.getBar();

// After
x = obj.bar;
```

### Return a parameter

```js
// Before
Foo.prototype.identity = function (x) {
  return x;
};
y = obj.identity(42);

// After
y = 42;
```

### Return a constant

```js
// Before
Foo.prototype.isEnabled = function() { return true; };
if (obj.isEnabled()) { ... }

// After
if (true) { ... }
```

### Safety constraints

- The method body must be a single `return` statement
- The returned expression must be side-effect-free
- For `this.prop` returns, the receiver at the call site must be the same object (no aliasing through `call`/`apply`/`bind`)
- The method must not be overridden in subclasses (requires whole-program analysis or conservative assumptions)
- The method must not use `arguments`, `super`, or `new.target`

## References

- Closure Compiler: `InlineSimpleMethods.java`
- Design 019 (Inline) — covers general function inlining; this pass targets a specific profitable subset
