# Function to Arrow

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Convert function expressions to arrow functions when safe. This saves the `function` keyword and enables concise body syntax for single-expression returns.

## Why

Arrow functions are shorter than function expressions. `function(x) { return x; }` (27 chars) becomes `x=>x` (4 chars). Even with block bodies, removing the `function` keyword saves 8 bytes per conversion. Arrow body simplification (`(x) => { return expr; }` → `(x) => expr`) saves additional bytes.

## Transformations

### Function expression to arrow (concise body)

When a function expression has a single `return` statement, convert to arrow with concise body.

```js
// Before
var f = function(x) { return x * 2; };

// After
var f = (x) => x * 2;
```

### Function expression to arrow (block body)

When a function expression has a multi-statement body, convert to arrow with block body.

```js
// Before
var f = function(x) { console.log(x); return x; };

// After
var f = (x) => { console.log(x); return x; };
```

### Single parameter parentheses removal

When an arrow has a single simple parameter, omit parentheses (codegen concern).

```js
// Before
(x) => x * 2

// After
x => x * 2
```

### Safety constraints

Only convert when the function does NOT:
- Use `this` (arrows inherit `this` from enclosing scope)
- Use `arguments` (arrows have no own `arguments`)
- Use `new.target` or `super`
- Is a generator (`function*`)
- Is used as a constructor (`new fn()`) — arrows cannot be constructed
- Has `.prototype` access checked — arrows have no `.prototype` property

```js
// NOT safe — uses `this`
var obj = { f: function() { return this.x; } };

// NOT safe — used as constructor
var F = function(x) { this.x = x; };
new F(1);
```

## References

- Terser: `arrows: true` (body simplification only); `unsafe_arrows` (opt-in function→arrow)
- SWC: `unsafe_optimize_fn_as_arrow` checks `this`/`arguments`/generator; `optimize_arrow_body` simplifies block→expression; `optimize_arrow_method_prop` converts methods↔arrows
- esbuild: Does NOT convert function→arrow (changes constructability). Only does arrow block→expression body
- Closure: No function→arrow pass (only arrow→function for ES5 transpilation)
