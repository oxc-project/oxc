# Peephole Fold Constants

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

This pass evaluates constant expressions at compile time, replacing them with their computed results. It handles arithmetic, comparisons, logical operators, bitwise operations, string concatenation, `typeof`, and property access on literals.

## Why

Constant expressions appear naturally in source code (e.g., feature flags, computed indices, concatenated strings) and are also created as intermediate results by other optimization passes. Folding them at compile time reduces code size and may enable further dead-code elimination when folded values are used in conditions.

## Transformations

### Arithmetic operations

Fold binary arithmetic on numeric literals.

```js
// Before
var x = 2 + 3;
var y = 10 * 4 - 5;
var z = 100 / 4;
var w = 17 % 5;

// After
var x = 5;
var y = 35;
var z = 25;
var w = 2;
```

### Comparison operations

Fold comparisons between constants.

```js
// Before
1 === 1
"a" < "b"
null == undefined
0 === null

// After
true
true
true
false
```

### `typeof` on literals

Evaluate `typeof` when the operand is a literal.

```js
// Before
typeof 42
typeof "hello"
typeof true
typeof undefined
typeof null

// After
"number"
"string"
"boolean"
"undefined"
"object"
```

### Logical short-circuit

Fold `&&` and `||` when the left operand determines the result.

```js
// Before
true && x
false && x
true || x
false || x

// After
x
false
true
x
```

### Nullish coalescing

Fold `??` when the left operand is known to be null/undefined or not.

```js
// Before
null ?? x
undefined ?? x
42 ?? x

// After
x
x
42
```

Side effects in the left operand must be preserved:

```js
// Before
(a(), null) ?? 1

// After
(a(), null, 1)
```

### Optional chain folding

When the base of an optional chain is known null/undefined, the entire chain evaluates to `undefined`.

```js
// Before
null?.foo       →  void 0
undefined?.foo  →  void 0
```

### Bitwise operations

Fold bitwise operations on integer constants.

```js
// Before
5 | 3
5 & 3
5 ^ 3
~0
1 << 4
-1 >>> 0

// After
7
1
6
-1
16
4294967295
```

### String concatenation

Fold string `+` operations.

```js
// Before
"hello" + " " + "world"
"value: " + 42

// After
"hello world"
"value: 42"
```

### Unary operations

Fold unary `+`, `-`, `!`, `~` on constants.

```js
// Before
-(-5)
+true
!false
!!0

// After
5
1
true
false
```

### Property access on literals

Fold known property access on literal values.

```js
// Before
"hello".length
[1, 2, 3].length

// After
5
3
```

### Spread flattening

Merge adjacent spread and non-spread elements when the spread target is a known array.

```js
// Before
[1, ...[2, 3], 4]

// After
[1, 2, 3, 4]
```

## References

- `PeepholeFoldConstants.java`
- `js_ast_helpers.go:1235`, `js_printer.go:1765`
- `compress/evaluate.js:224,273`, `compress/index.js:2236`
