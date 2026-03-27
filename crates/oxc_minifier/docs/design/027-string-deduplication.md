# String Deduplication

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Extract repeated string literals that appear N or more times into a shared variable declaration. The variable name (a single character after mangling) replaces each occurrence, saving bytes when the total savings exceed the cost of the variable declaration.

## Why

Codebases often repeat the same string literals — error messages, event names, property keys, CSS class names. Each repetition costs the full string length. Extracting to a variable costs `var a="str";` once, then `a` at each use site. For a string of length L appearing N times, this saves approximately `(N-1) * L - (4 + L + 2)` bytes (accounting for `var a="str";` overhead). The pass is profitable when strings are long enough and repeated often enough.

## Transformations

### Basic string extraction

```js
// Before
f("error: invalid input");
g("error: invalid input");
h("error: invalid input");

// After
var a = "error: invalid input";
f(a);
g(a);
h(a);
```

### Profitability check

Short or infrequent strings are not extracted:

```js
// Before — "ok" appears 3 times but is too short
f("ok");
g("ok");
h("ok");

// After (unchanged — extraction would increase size)
f("ok");
g("ok");
h("ok");
```

### Placement

The variable declaration is placed at the highest common scope of all occurrences, as early as possible:

```js
// Before
function a() {
  log("event_name");
}
function b() {
  log("event_name");
}

// After
var c = "event_name";
function a() {
  log(c);
}
function b() {
  log(c);
}
```

### Safety constraints

- Only extract string literals (not template literals or computed strings)
- The profitability threshold depends on the string length and occurrence count
- Must respect scope boundaries — the shared variable must be accessible at all use sites
- String extraction changes reference identity (`===` still works since strings are value-compared)
- Should run late in the pipeline (after other passes have finalized string usage)

## References

- Closure: `AliasStrings.java` — aliases repeated strings to module-level variables
- Terser: Does not implement string deduplication as a built-in pass
- esbuild: Does not implement string deduplication
- SWC: Does not implement string deduplication
