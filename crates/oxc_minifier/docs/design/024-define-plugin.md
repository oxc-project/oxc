# Define Plugin

- **Status:** Implemented
- **Difficulty:** Simple

## What

Replace compile-time constants (`process.env.NODE_ENV`, `__DEV__`, user-defined globals) with literal values at build time. The resulting dead branches are then removed by DCE passes.

## Why

Production builds universally gate debug code behind environment checks. Replacing these checks with constants at compile time enables entire code paths to be statically eliminated, often removing significant portions of development-only code (logging, warnings, invariant checks, devtools hooks).

This is one of the most impactful optimizations in practice because virtually every framework and library uses `process.env.NODE_ENV` or similar guards.

## Transformations

### Replace `process.env.NODE_ENV`

```js
// Before
if (process.env.NODE_ENV !== "production") {
  console.warn("Debug mode");
}

// After (with NODE_ENV="production")
if ("production" !== "production") {
  console.warn("Debug mode");
}

// After constant folding + DCE
// (entire block removed)
```

### Replace user-defined globals

```js
// Config: { __DEV__: false, __VERSION__: '"1.0.0"' }

// Before
if (__DEV__) {
  enableDevTools();
}
console.log(__VERSION__);

// After
if (false) {
  enableDevTools();
}
console.log("1.0.0");
```

### Replace member expressions

```js
// Config: { "process.env.API_URL": '"https://api.prod.com"' }

// Before
fetch(process.env.API_URL + "/users");

// After
fetch("https://api.prod.com" + "/users");
```

## References

- Closure Compiler: `ProcessDefines.java`
- esbuild: `--define:` flag, `define` in build options
- Terser: `global_defs` compress option
- SWC: `jsc.transform.optimizer.globals`
