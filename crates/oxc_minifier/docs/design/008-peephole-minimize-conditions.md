# Peephole Minimize Conditions

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

This pass rewrites conditional expressions and control flow into shorter equivalent forms. It targets `if`/`else` statements, ternary expressions, logical operators, and comparison operators to find more compact representations.

## Why

Conditional logic is one of the most common patterns in JavaScript. Small savings on each conditional compound across an entire program. Many conditionals also have structural redundancies — nested `if`s, negated tests, or branches that could be expressed as expressions instead of statements.

## Transformations

### Convert `if` to expression with `&&` / `||`

When an `if` body is a single expression statement with no `else`, rewrite as a logical-AND expression.

```js
// Before
if (x) { a(); }

// After
x && a();
```

When the body is an assignment or other expression, the `&&` form is shorter because it eliminates braces, `if`, and parentheses overhead.

### Convert `if`/`else` to ternary

When both branches of an `if`/`else` are single expression statements, rewrite as a conditional expression.

```js
// Before
if (c) { a(); } else { b(); }

// After
c ? a() : b();
```

```js
// Before
if (c) { x = 1; } else { x = 2; }

// After
x = c ? 1 : 2;
```

### Apply De Morgan's law

Distribute negation across logical operators to eliminate a `!` when the result is shorter.

```js
// Before
if (!a && !b) { x(); }

// After
if (!(a || b)) { x(); }
```

```js
// Before
!(!a || !b)

// After
a && b
```

### Merge nested `if` statements

Consecutive or nested `if`s with no `else` can combine their conditions with `&&`.

```js
// Before
if (a) { if (b) { x(); } }

// After
if (a && b) { x(); }
```

### Flip negated comparisons

Remove leading `!` by using the negated comparison operator.

```js
// Before
!(a === b)

// After
a !== b
```

```js
// Before
!(x > 0)

// After
x <= 0
```

### Hoist `return` from `if`/`else`

When both branches of an `if`/`else` end with `return`, hoist the `return` and use a ternary for the value.

```js
// Before
if (c) { return a; } else { return b; }

// After
return c ? a : b;
```

### Invert `if` when `else` falls through

When an `if` branch returns/breaks and the `else` continues, invert the condition to reduce nesting.

```js
// Before
if (a) {
  return x;
} else {
  b();
  c();
}

// After
if (!a) {
  b();
  c();
  return;
}
return x;
```

### Collapse null/undefined checks to loose equality

Strict equality checks against both `null` and `undefined` can be collapsed to a single loose equality check.

```js
// Before
x === null || x === undefined
x !== null && x !== undefined

// After
x == null
x != null
```

Only safe when `x` is side-effect-free (no getter). Refs: esbuild `MangleEquals`; Terser `comparisons`.

### Simplify ternary to logical

Ternary expressions with boolean literal branches can be simplified to logical expressions.

```js
// Before → After
a ? b : false    →  a && b
a ? true : b     →  a || b
a ? false : b    →  !a && b
a ? b : true     →  !a || b
```

## References

- `PeepholeMinimizeConditions.java`
- `js_parser.go:10184`, `js_ast_helpers.go:2788,2096`
- `compress/index.js:1088,3208`, `compress/tighten-body.js:991`
