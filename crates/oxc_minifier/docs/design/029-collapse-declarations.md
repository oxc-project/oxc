# Collapse Declarations

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Three related cleanup passes that reduce declaration verbosity and merge statements:

1. **Join variable declarations** — combine consecutive `var`/`let`/`const` statements
2. **Collapse anonymous functions** — convert `var f = function(){}` to `function f(){}`
3. **Denormalize** — merge statements into for-loop init positions and collapse expanded assignments

These are the inverse of normalization (002) and run after optimization passes complete.

## Why

Normalization splits declarations and expands assignments to simplify analysis. After all optimization passes finish, recombining them produces shorter output. Joining `var` declarations saves one `var` keyword plus semicolon per merge. Collapsing anonymous functions to declarations saves the assignment operator and `var` keyword. Denormalization folds preceding statements into for-loop headers.

## Transformations

### Join variable declarations

```js
// Before
var a = 1;
var b = 2;
var c = 3;

// After
var a = 1, b = 2, c = 3;
```

Only joins consecutive declarations of the same kind (`var`/`let`/`const`).

### Collapse anonymous functions

```js
// Before
var f = function() { return 1; };

// After
function f() { return 1; }
```

Only safe when `f` is not reassigned and the declaration is in statement position (not inside an expression).

### Denormalize: fold into for-loop init

```js
// Before
a = 0;
for (; a < 10; a++) { body(); }

// After
for (a = 0; a < 10; a++) { body(); }
```

### Denormalize: collapse compound assignments

```js
// Before (expanded by normalization)
x = x + 1;
x = x * 2;

// After
x += 1;
x *= 2;
```

## References

- Closure Compiler: `CollapseVariableDeclarations.java`, `CollapseAnonymousFunctions.java`, `Denormalize.java`
- Terser: `join_vars` option in `compress`
