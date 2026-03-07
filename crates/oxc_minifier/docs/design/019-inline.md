# Inline

- **Status:** Not Implemented
- **Difficulty:** Medium

## What

This pass inlines variables, functions, and properties — replacing references with their values and call sites with function bodies. It covers three related transformations:
- **Variable inlining**: replace a variable reference with its assigned value
- **Function inlining**: replace a function call with the function body
- **Property inlining**: replace a property access with the property's constant value

## Why

Inlining eliminates indirection. Each inlined variable removes a `var` declaration and a name reference. Each inlined function call removes a function definition and call overhead. This often enables further optimizations: constant folding on inlined literal values, dead code elimination of the now-unreferenced declaration, and scope reduction.

## Transformations

### Inline single-use variables

When a variable is assigned once and read once, replace the read with the value and remove the declaration.

```js
// Before
var x = a + b;
console.log(x);

// After
console.log(a + b);
```

### Inline constant variables

When a variable is assigned a constant value (literal, or pure expression of other constants) and read multiple times, replace each read with the value.

```js
// Before
var PI = 3.14159;
var circumference = 2 * PI * r;
var area = PI * r * r;

// After
var circumference = 2 * 3.14159 * r;
var area = 3.14159 * r * r;
```

### Inline aliases

When a variable is assigned another variable name and never reassigned, replace references with the original name.

```js
// Before
var x = longVariableName;
use(x);
use(x);

// After
use(longVariableName);
use(longVariableName);
```

Note: after mangling, `longVariableName` becomes a short name, making this profitable.

### Inline simple functions (direct return)

When a function body is a single `return` expression, replace call sites with the expression (substituting arguments for parameters).

```js
// Before
function square(x) { return x * x; }
var result = square(5);

// After
var result = 5 * 5;
```

### Inline function body (block inlining)

For single-use functions with a multi-statement body, replace the call with the body statements (adjusting variable names to avoid conflicts).

```js
// Before
function init() {
  setupA();
  setupB();
  configure();
}
init();

// After
setupA();
setupB();
configure();
```

### Inline IIFEs

Immediately-invoked function expressions with no arguments and no `this`/`arguments` usage can be replaced with their body.

```js
// Before
(function() {
  setup();
  configure();
})();

// After
setup();
configure();
```

### Inline property values

When an object property is assigned a constant and only read (never written after initialization), inline the value at read sites.

```js
// Before
var config = {};
config.DEBUG = false;

if (config.DEBUG) {
  console.log("debug mode");
}

// After
if (false) {
  console.log("debug mode");
}
// (then dead code elimination removes the if block)
```

### Inlining cost thresholds

Values cheap enough to inline even with multiple references:
- **Numbers**: integers from -99 to 999 (no fractional part) — at most 3 characters
- **Strings**: ≤3 characters (including quotes, still small)
- **Booleans**: `!0` and `!1` are 2 bytes each
- **null, undefined** (`void 0`): always small
- **Single reference**: always inline regardless of value size

### Safety constraints

Inlining is only safe when:
- The inlined expression has no side effects, or is evaluated exactly as many times as before
- The evaluation order of surrounding expressions is preserved
- The inlined expression does not reference variables that would have different values at the new location
- For functions: `this`, `arguments`, `new.target`, and `super` references prevent inlining
- `var` vs `let`/`const`: `var` declarations have different TDZ semantics, requiring extra care
- Must detect write references to avoid inlining values that are later mutated

```js
// NOT safe to inline — side effect would execute twice
var x = sideEffect();
use(x, x);  // cannot replace both x with sideEffect()
```

## References

- `InlineVariables.java`, `InlineFunctions.java`, `InlineProperties.java`
- `js_parser.go:9623,11863`, `js_printer.go:1673`
- `compress/inline.js:169,~250`
