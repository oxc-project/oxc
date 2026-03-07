# Drop Statements

- **Status:** Not Implemented
- **Difficulty:** Trivial

## What

Configurable removal of `console.*` calls, `debugger` statements, and labeled blocks. These are development-only constructs that should not appear in production builds.

## Why

`console.log` calls and `debugger` statements are used during development but waste bytes in production. Removing them is a straightforward win with no semantic risk when configured by the user. Labeled block dropping enables custom compile-time removal of tagged code sections.

## Transformations

### Drop console calls

When `drop_console: true` is configured:

```js
// Before
console.log("debug info");
console.warn("warning");
console.error("error");

// After
// (all removed)
```

### Drop debugger statements

When `drop_debugger: true` is configured:

```js
// Before
function f() {
  debugger;
  return 1;
}

// After
function f() {
  return 1;
}
```

### Drop labeled blocks

When `drop_labels: ["DEV"]` is configured:

```js
// Before
DEV: {
  console.log("development only");
  validate(input);
}

// After
// (removed entirely)
```

### Console calls in expressions

When a console call is used as an expression (not a statement), replace with `void 0` or remove the branch:

```js
// Before
var x = console.log("test") || fallback;

// After
var x = void 0 || fallback;
```

### Safety constraints

- All three options are off by default — user must opt in
- `drop_console` removes all `console.*` member calls
- `drop_debugger` is commonly enabled by default in production builds
- `drop_labels` only removes blocks with explicitly listed label names

## References

- esbuild: `--drop:console`, `--drop:debugger`, `--drop-labels:NAME`
- Terser: `drop_console: true`, `drop_debugger: true` in compress options
- SWC: `drop_console`, `drop_debugger` in compress options (mirrors Terser)
