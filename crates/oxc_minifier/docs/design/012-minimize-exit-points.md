# Minimize Exit Points

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

This pass reduces the number of explicit exit statements (`return`, `break`, `continue`) in functions and loops. It removes redundant trailing exits and restructures control flow to eliminate unnecessary branching.

## Why

Every `return`, `break`, or `continue` keyword costs bytes. Trailing returns at the end of a function are unnecessary — control falls off naturally. Conditional returns followed by remaining code can be restructured to eliminate either the return or the else branch. These savings also enable further optimizations like statement fusion and brace elimination.

## Transformations

### Remove trailing `return`

A `return` (with no value) at the end of a function is unnecessary.

```js
// Before
function f() {
  doSomething();
  return;
}

// After
function f() {
  doSomething();
}
```

### Remove trailing `break` in `switch`

The last `case` in a `switch` does not need a `break`.

```js
// Before
switch (x) {
  case 1:
    a();
    break;
  case 2:
    b();
    break;
}

// After
switch (x) {
  case 1:
    a();
    break;
  case 2:
    b();
}
```

### Remove trailing `continue`

A `continue` at the end of a loop body is unnecessary.

```js
// Before
for (var i = 0; i < n; i++) {
  process(i);
  continue;
}

// After
for (var i = 0; i < n; i++) {
  process(i);
}
```

### Convert `if`-return to inverted guard

When an `if` block ends with a return and subsequent code follows, invert the condition and move the remaining code inside the guard.

```js
// Before
function f() {
  if (check) {
    return value;
  }
  a();
  b();
  return other;
}

// After
function f() {
  if (!check) {
    a();
    b();
    return other;
  }
  return value;
}
```

This is beneficial when the inverted form enables further minimization (e.g., the condition pass can then fold the if/return into a ternary).

### Move siblings into `else` branch

When an `if` block always exits (via `return`/`throw`), subsequent statements at the same level are effectively the `else` branch and can be moved inside one.

```js
// Before
function f(x) {
  if (x) {
    return a();
  }
  b();
  c();
}

// After
function f(x) {
  if (x) return a();
  b();
  c();
}
```

When combined with the minimize-conditions pass, this often leads to expression form:

```js
// Final
function f(x) {
  return x ? a() : (b(), c());
}
```

## References

- `MinimizeExitPoints.java`
- `js_parser.go:9464-9617`
- `compress/tighten-body.js:991`
