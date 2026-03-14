# Flow-Sensitive Variable Inlining

- **Status:** Not Implemented
- **Difficulty:** Complex

## What

Use CFG reaching-definitions analysis to inline variables across control flow branches, beyond the simple sequential inlining in design 019. Standard inlining only replaces a variable when the definition and use are in the same straight-line code sequence. Flow-sensitive inlining handles cases where the definition is in one branch and the use is in another, as long as the data flow analysis proves the definition uniquely reaches the use.

## Why

Sequential inlining misses opportunities that arise from control flow:

```js
// Before — sequential inlining cannot handle this
var x;
if (condition) {
  x = expensive();
} else {
  x = cheap();
}
// At this point, x has a unique reaching definition on each path,
// but sequential analysis sees two assignments and gives up
```

Flow-sensitive analysis tracks definitions through the CFG and determines, at each use site, which definitions can reach it. When exactly one definition reaches a use, the variable can be inlined even though the definition and use are in different basic blocks.

## How It Works

1. Build the CFG for the function
2. Run **MustBeReachingVariableDef** analysis — a forward data flow analysis that computes, for each program point, the set of variable definitions that _must_ reach it (intersection at join points)
3. For each variable use, check whether exactly one definition reaches it
4. If the definition is a simple expression (no side effects, or evaluated the same number of times), replace the use with the definition's value
5. If all uses of the variable are inlined, remove the declaration

### Safety constraints

- The reaching definition must be unique (must-reach, not may-reach) — at join points, if different branches define different values, neither can be inlined
- Side-effect ordering must be preserved — the inlined expression must not move past other side-effectful operations
- The inlined value's dependencies must not be reassigned between definition and use
- Variables captured by closures require extra care — the closure may be invoked after a reassignment

## References

- Closure Compiler: `FlowSensitiveInlineVariables.java` — uses `MustBeReachingVariableDef` data flow analysis
- Design 019 (Inline) — covers sequential variable/function inlining
- Data flow analysis framework — described in `data-structures.md`
