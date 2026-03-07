# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix: Minifier performance regression with many declarators (Issue #17369)

## Context

A file only ~6% larger takes 2.5x longer to minify (282ms vs 704ms). The `bad.js` file contains ~10,000 root-level `var` declarations like:
```js
var import_react$9947 = /* @__PURE__ */ __toESM(require_react());
```
This is a common pattern produced by bundlers (esbuild) when deduplicating imports across module boundaries.

### Root cause

In `substitute_single_use_symbol_in_ex...

### Prompt 2

i don't like the solution, check esbuild in /Users/boshen/github/esbuild

### Prompt 3

continue

### Prompt 4

use rust iterator?

### Prompt 5

create pr

