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

When the surrounding context only observes truthiness, or when the test is already known to be boolean, ternary expressions with boolean literal branches can be simplified to logical expressions.

```js
// In boolean context:
if (a ? b : false)  →  if (a && b)
if (a ? true : b)   →  if (a || b)

// If `a` is already known boolean:
a ? b : false       →  a && b
a ? true : b        →  a || b
```

These rewrites are not valid for arbitrary `a`: for example, `0 ? b : false` evaluates to `false`, while `0 && b` evaluates to `0`.

### Type-aware equality relaxation

```js
// typeof always returns a string, so === is unnecessary:
typeof foo === 'number'  →  typeof foo == 'number'
// instanceof always returns boolean:
a instanceof b === true  →  a instanceof b
```

### Negation distribution into sequences

```js
!(a, b)  →  a, !b
```

### Conditional expression sequence hoisting

```js
(a, b) ? c : d  →  a, b ? c : d
```

### Assignment pattern in null checks

```js
// Before
(a = foo.bar) === null || a === undefined

// After
(a = foo.bar) == null
```

### Boolean context minimization

```js
// In boolean context (if test, logical operand):
!!a       →  a
(a|b)===0 →  !(a|b)
(a|b)!==0 →  !!(a|b)   // or just (a|b) in boolean context
```

### Remove side-effect-free values from logical chains

```js
// In boolean context, truthy constants in && and falsy constants in || can be removed:
if (a && true)   →  if (a)
if (a || false)  →  if (a)
```

## References

- `PeepholeMinimizeConditions.java`
- `js_parser.go:10184`, `js_ast_helpers.go:2788,2096`
- `compress/index.js:1088,3208`, `compress/tighten-body.js:991`
