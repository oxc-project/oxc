# Codegen Optimizations

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Size-reducing techniques applied at code generation time (owned by `oxc_codegen`). These do not transform the AST — they choose the shortest textual representation when printing.

## Why

The same AST node can be printed in multiple equivalent ways. Picking the shortest representation for numbers, strings, and punctuation can save significant bytes without any semantic analysis.

## Transformations

### Shortest number representation

Try integer, decimal, exponential, and hex representations; emit the shortest.

```js
// Before → After
1000      → 1e3
0.5       → .5
0.0001    → 1e-4
1000000   → 1e6
0xFFFF    → 65535  // or keep hex if shorter
-0        → -0     // must preserve
```

### Best quote selection

Count escape costs for single quotes, double quotes, and backticks. Pick the quote style requiring the fewest escape characters.

```js
// If string contains more double quotes:
"it's a \"test\"" → 'it\'s a "test"'

// If string contains more single quotes:
'it\'s a "test"' → "it's a \"test\""
```

Algorithm: analyze string content and count escape costs — each quote character that would need escaping adds +1 cost for that quote style. Newlines reduce backtick cost (template literals don't need `\n` escapes). `${` sequences increase backtick cost. Pick the quote with lowest total cost. Preference when tied: double quote (most common convention).

### NaN and Infinity representation

Replace global constants with shorter arithmetic equivalents.

```js
NaN      → 0/0
Infinity → 1/0
undefined → void 0
```

### Semicolon omission

Use ASI (Automatic Semicolon Insertion) rules to omit semicolons where safe — typically before `}`, at end of file, or before line breaks in certain contexts.

```js
// Before
function f() {
  return 1;
}

// After
function f() {
  return 1;
}
```

### UTF-8 charset mode

When the output charset is known to be UTF-8, emit non-ASCII identifiers and strings directly instead of using `\uXXXX` escape sequences.

```js
// ASCII-safe mode
var é = "\u00E9";

// UTF-8 mode
var é = "é";
```

### Remove `new` parentheses

When a constructor call has no arguments, the parentheses are optional per spec and can be omitted.

```js
// Before
new Foo();

// After
new Foo();
```

Caveat: `new Foo` without parens cannot appear in a postfix-precedence context:

```js
// This is ambiguous:
new Foo.bar(); // means new (Foo.bar), not (new Foo).bar
// So parentheses must be kept when followed by member access:
new Foo().bar; // cannot simplify to new Foo.bar
```

### ASCII-only output mode

Force `\uXXXX` escapes for all non-ASCII characters. This is the inverse of UTF-8 charset mode and ensures output is safe for any encoding context.

```js
// UTF-8 mode
var é = "café";

// ASCII-only mode
var é = "caf\u00E9";
```

Default in esbuild (`--charset=ascii`), Terser (`ascii_only: true`).

### Whitespace edge cases

Smart spacing is needed to prevent invalid output:

- `return true` must not become `returntrue`
- `typeof x` must not become `typeofx`
- `/regex/` after `/` would form a comment `//`

## References

- esbuild: `js_printer.go` — number printing, quote selection
- Terser: `output.js` — `make_num()`, quote optimization
