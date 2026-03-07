# Dead Assignments Elimination

- **Status:** Not Implemented
- **Difficulty:** Complex

## What

This pass removes assignments to variables whose values are never subsequently read. It uses dataflow analysis (liveness) on the control flow graph to determine which assignments are "dead" — overwritten before being read or never read at all.

## Why

Dead assignments waste bytes and obscure program intent. They arise naturally from code patterns (initializing a variable then overwriting it conditionally), from inlining (where the inlined code introduces redundant stores), and from other optimization passes that eliminate reads but leave assignments behind.

## How It Works

1. **Build control flow graph** — represent the function as basic blocks connected by control flow edges
2. **Compute liveness** — perform backward dataflow analysis to determine, at each program point, which variables may be read before their next write
3. **Identify dead assignments** — an assignment to variable `x` is dead if `x` is not live after the assignment
4. **Remove or simplify** — if the RHS has no side effects, remove the entire statement; if the RHS has side effects, keep the RHS as an expression statement and drop the assignment target

## Transformations

### Remove dead simple assignments

When the RHS is side-effect-free, remove the entire statement.

```js
// Before
function f(x) {
  var y = x + 1; // y is never read
  return x * 2;
}

// After
function f(x) {
  return x * 2;
}
```

### Preserve side effects

When the RHS has side effects, keep the expression but drop the assignment.

```js
// Before
function f() {
  var result = sideEffect(); // result never read
  return 0;
}

// After
function f() {
  sideEffect();
  return 0;
}
```

### Remove overwritten assignments

When a variable is assigned twice with no read between, the first assignment is dead.

```js
// Before
x = computeA(); // dead — overwritten before read
x = computeB();
use(x);

// After
computeA();
x = computeB();
use(x);
```

### Handle conditional control flow

Liveness analysis correctly handles branches — an assignment is only dead if _all_ paths from the assignment point overwrite the variable before reading it.

```js
// Before
var x = 1; // NOT dead — read on the else path
if (cond) {
  x = 2;
  use(x);
} else {
  use(x); // reads original x = 1
}
```

### Remove dead parameters

Function parameters that are never read can have their assignments conceptually eliminated (the parameter slot remains for call-site compatibility, but internal uses can be simplified).

```js
// Before
function f(a, b, unused) {
  return a + b;
}

// After (no change to signature, but internal analysis marks `unused` as dead)
function f(a, b, unused) {
  return a + b;
}
```

## References

- `DeadAssignmentsElimination.java`
- `js_parser.go:9104-9137`
- `compress/drop-unused.js:112`, `compress/reduce-vars.js:116`
