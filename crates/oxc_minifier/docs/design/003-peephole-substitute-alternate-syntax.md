# Peephole Substitute Alternate Syntax

## What

This pass replaces common JavaScript constructs with shorter equivalent syntax. Each substitution preserves semantics while reducing code size by exploiting language quirks and shorter idioms.

## Why

JavaScript has verbose standard forms for many operations — `true`, `undefined`, `new Array()`, `String(x)`. Shorter alternatives exist that produce identical results. Individually each saves a few bytes, but they appear frequently and the savings compound across a full program.

## Transformations

### Boolean literals

Replace `true` with `!0` and `false` with `!1`.

```js
// Before
var a = true;
var b = false;

// After
var a = !0;
var b = !1;
```

### `undefined` references

Replace `undefined` with `void 0` (shorter and cannot be shadowed in older environments).

```js
// Before
return undefined;

// After
return void 0;
```

### Remove `return undefined`

A `return undefined` or `return void 0` at the end of a function is equivalent to just `return` or omitting the statement entirely.

```js
// Before
function f() { doSomething(); return undefined; }

// After
function f() { doSomething(); }
```

### Constructor shortcuts

Replace `new Object()` with `{}`, `new Array()` with `[]`.

```js
// Before
var o = new Object();
var a = new Array();

// After
var o = {};
var a = [];
```

### Type coercion shortcuts

Replace wrapper function calls with shorter operator equivalents.

```js
// Before
Boolean(x)
String(x)
Number(x)

// After
!!x
x + ""   // or "" + x
+x
```

### `typeof` comparisons

Rewrite `typeof x === "undefined"` to shorter form when in a boolean context.

```js
// Before
typeof x === "undefined"

// After
typeof x > "u"
```

### Template literal to string

When a template literal has no expressions, replace with a regular string.

```js
// Before
var s = `hello world`;

// After
var s = "hello world";
```

### String `.split()` for arrays

Replace array literals of single-character strings with a `.split()` call.

```js
// Before
var a = ["a", "b", "c", "d", "e"];

// After
var a = "abcde".split("");
```

### `Infinity` shorthand

Replace `Infinity` with `1/0`.

```js
// Before
var x = Infinity;

// After
var x = 1 / 0;
```

### Exponentiation

Replace `Math.pow(a, b)` with `a ** b` when targeting ES2016+.

```js
// Before
Math.pow(x, 3)

// After
x ** 3
```

## References

- `PeepholeSubstituteAlternateSyntax.java`
- `js_printer.go:2909`, `js_parser.go:14674`
- `compress/index.js:3469,1677,2952`
