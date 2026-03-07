# Peephole Replace Known Methods

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

This pass evaluates calls to well-known built-in methods at compile time when all arguments are constants. The method call is replaced with its computed result as a literal value.

## Why

Code frequently calls standard library methods with constant arguments — string manipulation, math calculations, number parsing. When both the receiver and arguments are known at compile time, the result is deterministic and can be inlined as a literal, eliminating the function call overhead and reducing code size.

## Transformations

### String methods

Evaluate `String.prototype` methods on constant strings.

```js
// Before
"hello".charAt(1)
"hello world".indexOf("world")
"hello".slice(1, 3)
"abc".toUpperCase()
"Hello World".toLowerCase()
"  hello  ".trim()
"abc".repeat(3)
"hello".substring(1, 4)
"a-b-c".split("-")

// After
"e"
6
"el"
"ABC"
"hello world"
"hello"
"abcabcabc"
"ell"
["a","b","c"]
```

### `Array.prototype.join`

Evaluate `.join()` on constant arrays.

```js
// Before
["a", "b", "c"].join("-")
[1, 2, 3].join("")

// After
"a-b-c"
"123"
```

### `Math` methods

Evaluate `Math` static methods with constant arguments.

```js
// Before
Math.max(1, 5, 3)
Math.min(2, 8)
Math.floor(4.7)
Math.ceil(4.1)
Math.round(4.5)
Math.abs(-3)
Math.sqrt(9)
Math.pow(2, 10)

// After
5
2
4
5
5
3
3
1024
```

### `Number` methods

Evaluate `Number` conversions and checks.

```js
// Before
Number.isFinite(42)
Number.isNaN(NaN)
Number.parseInt("15", 16)

// After
true
true
21
```

### `parseInt` / `parseFloat`

Evaluate global parsing functions on constant strings.

```js
// Before
parseInt("42")
parseInt("0xFF", 16)
parseFloat("3.14")

// After
42
255
3.14
```

### `String.fromCharCode`

Evaluate character code conversion.

```js
// Before
String.fromCharCode(72, 101, 108)

// After
"Hel"
```

## References

- `PeepholeReplaceKnownMethods.java`
- `js_parser.go:15385-15610`
- `compress/evaluate.js:467`, `compress/native-objects.js`
