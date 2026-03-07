# Statement Fusion

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

This pass fuses consecutive expression statements into a single statement using the comma operator. It also folds expressions into `return`, `throw`, and loop constructs to reduce the total number of statements.

## Why

Each statement in a block requires a semicolon and prevents the block from being reduced to a single expression. By fusing N expression statements into one comma expression, the block may shrink enough to eliminate braces entirely (e.g., in `if`/`else` bodies, arrow functions). The comma operator provides the same execution order and the overall value is that of the last expression.

## Transformations

### Fuse consecutive expression statements

Combine adjacent expression statements with the comma operator.

```js
// Before
a();
b();
c();

// After
a(), b(), c();
```

### Fuse into `return`

Merge preceding expression statements into a `return` statement.

```js
// Before
a();
b();
return c();

// After
return a(), b(), c();
```

### Fuse into `throw`

Merge preceding expression statements into a `throw` statement.

```js
// Before
cleanup();
throw new Error("fail");

// After
throw cleanup(), new Error("fail");
```

### Fuse into `if` condition

Merge preceding expression statements into the `if` test.

```js
// Before
a();
b();
if (c) { d(); }

// After
a(), b();
if (c) { d(); }
```

Note: only some statements before the `if` are fused — the last expression statement before the `if` is fused into the condition only if it produces a boolean-compatible value or if the fused form is shorter.

### Fuse into `for` loop initializer

Move expression statements into an empty `for` initializer.

```js
// Before
x = 0;
y = 1;
for (; i < n; i++) { body(); }

// After
for (x = 0, y = 1; i < n; i++) { body(); }
```

### Enable brace elimination

After fusion, single-statement blocks can drop their braces.

```js
// Before (after minimize-conditions)
if (c) { a(); b(); }

// After fusion
if (c) a(), b();
```

## References

- `StatementFusion.java`
- `js_parser.go:9233-9428`
- `compress/tighten-body.js:1247,1299`
