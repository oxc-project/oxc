# Optimize Calls

- **Status:** Not Implemented
- **Difficulty:** Simple-Medium

## What

Call-site and return-value optimizations that go beyond basic function inlining. These passes analyze how functions are called and how their return values are used to eliminate unnecessary code.

## Why

Functions often contain code that is unnecessary in context: return expressions whose values are never used, constructors that do nothing, simple property accessors that could be inlined, and dead property assignments. These patterns are common in class hierarchies and prototype-based code.

## Conceptual Dependencies

This design depends on **light interprocedural analysis**. Even within a single file, the
minifier still needs reasoning about which call sites target which functions, whether
return values are observed, whether parameters are used, and whether the callee escapes
through unknown references.

## Transformations

### Optimize returns

If all callers ignore a function's return value, simplify `return expr` to `expr; return`.

```js
// Before — all call sites: doWork() (return value unused)
function doWork() {
  return expensiveComputation();
}

// After
function doWork() {
  expensiveComputation();
}
```

This enables further optimization: if the expression is side-effect-free, the entire statement can be removed.

### Optimize constructors

Remove redundant ES class constructors: empty bodies or constructors that only call `super(...args)`.

```js
// Before
class Child extends Parent {
  constructor(...args) {
    super(...args);
  }
}

// After
class Child extends Parent {}
```

### Inline simple methods

Replace calls to trivial prototype methods with the method body.

```js
// Before
A.prototype.getName = function () {
  return this.name;
};
a.getName();

// After
a.name;
```

Only safe when the method is a simple property access or constant return, and the receiver type is known.

### Dead property assignments

Remove property assignments where the value is overwritten before any read.

```js
// Before
this.x = computeA();
this.x = computeB(); // overwrites without reading

// After
computeA(); // keep for side effects
this.x = computeB();
```

## References

- Closure Compiler: `OptimizeReturns.java`, `OptimizeConstructors.java`, `InlineSimpleMethods.java`, `DeadPropertyAssignmentElimination.java`
