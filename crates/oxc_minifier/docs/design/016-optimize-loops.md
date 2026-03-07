# Optimize Loops

## What

Optimize loop constructs beyond what normalization covers. This includes converting infinite loop patterns to `for(;;)`, eliminating dead loops, and merging break conditions into loop tests.

Note: 002-normalize already covers `while` → `for` conversion and for-loop initializer extraction; 003-peephole-remove-dead-code covers `do-while(false)` → block. This doc covers the remaining loop optimizations.

## Why

Loop constructs often contain redundancy. `while(true)` and `for(;true;)` waste bytes on a condition that is always true. Loops with constant-false conditions are dead code. Break-as-first-statement patterns can be folded into the loop condition for shorter output.

## Transformations

### Convert infinite loops to `for(;;)`

```js
// Before
while (true) { body; }
while (1) { body; }
for (; true; ) { body; }

// After
for (;;) { body; }
```

### Remove redundant loop condition

```js
// Before
for (; true; update) { body; }

// After
for (;; update) { body; }
```

### Dead loop elimination

When the condition is statically false, extract the initializer and hoist `var` declarations from the body.

```js
// Before
for (init(); false; update()) { var x = 1; f(); }

// After
init();
var x;
```

### Merge break condition into loop test

When a loop body starts with `if (cond) break;`, merge the condition into the loop test.

```js
// Before
for (;;) { if (x > 10) break; body; }

// After
for (; x <= 10;) { body; }
```

```js
// Before
while (a) { if (b) break; body; }

// After
for (; a && !b;) { body; }
```

### Eliminate instant-break loops

When a loop body is just `break`, the loop is equivalent to evaluating the initializer and condition.

```js
// Before
for (a; b; c) break;

// After
a; b;
```

### Remove empty loops

When a loop body is empty and the condition has no side effects, remove entirely.

```js
// Before
for (;;) {}  // infinite loop — cannot remove (intentional hang)
for (init(); false;) {}  // dead — remove

// After
init();
```

Note: Empty infinite loops cannot be removed as they represent intentional blocking behavior.

### Safety constraints

- `var` declarations inside removed loop bodies must be hoisted (they are visible outside the loop due to hoisting).
- `break`/`continue` inside nested structures must not be confused with the loop-level break.
- Loop condition merging must preserve short-circuit evaluation order.
- Labels on the loop must be considered — labeled breaks from inner structures may target the outer loop.

## References

- Terser: `loops` option. `AST_While` → `AST_For`. `do-while(true)` → `for(;;)`. `if_break_in_loop()` merges break into condition
- SWC: Instant break elimination `for(a;b;c) break` → `a;b;`. Constant condition simplification. `UnreachableHandler::preserve_vars()` for var hoisting
- esbuild: Does NOT do `while(true)` → `for(;;)`. Dead code elimination inside loop bodies via constant folding
- Closure: `tryFoldFor` removes dead for-loops, hoists vars via `redeclareVarsInsideBranch`. `tryFoldDoAway` removes `do-while(false)`. `tryFoldEmptyDo` converts empty do→for. `hasUnnamedBreakOrContinue(block)` safety check
