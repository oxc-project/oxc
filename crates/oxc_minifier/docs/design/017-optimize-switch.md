# Optimize Switch

- **Status:** Not Implemented
- **Difficulty:** Medium

## What

Optimize switch statement structure and cases. This includes folding constant discriminants, removing unreachable cases, converting trivial switches to `if` statements, and deduplicating identical case bodies.

## Why

Switch statements often contain redundancy: constant discriminants make most cases dead, single-case switches are better as `if`, and adjacent cases with identical bodies can be merged. These optimizations reduce code size and enable further dead code elimination.

## Transformations

### Fold constant discriminant

When the switch discriminant is a known constant, extract the matching case and remove the rest.

```js
// Before
switch (1) {
  case 0:
    a();
    break;
  case 1:
    b();
    break;
  case 2:
    c();
    break;
}

// After
b();
```

### Remove unreachable cases

Cases after a `return`/`throw`/`break` with no fallthrough are dead.

```js
// Before
switch (x) {
  case 1:
    return a();
  case 1:
    return b(); // duplicate case, unreachable
}

// After
switch (x) {
  case 1:
    return a();
}
```

### Convert single-case switch to `if`

```js
// Before
switch (x) {
  case 1:
    f();
    break;
}

// After
if (x === 1) f();
```

### Remove empty default case

```js
// Before
switch (x) {
  case 1:
    f();
    break;
  default:
}

// After
switch (x) {
  case 1:
    f();
    break;
}
```

### Deduplicate identical case bodies

Merge adjacent cases with the same body.

```js
// Before
switch (x) {
  case 1:
    f();
    break;
  case 2:
    f();
    break;
}

// After
switch (x) {
  case 1:
  case 2:
    f();
    break;
}
```

### Safety constraints

- `break` inside nested structures (if/loops/inner switches) must not be confused with switch-level break. All transforms require a `containsNestedBreak` check.
- Variable declarations in removed cases must be hoisted if they use `var`.
- Fallthrough semantics must be preserved — only remove cases that are truly unreachable.
- Case tag expressions may have side effects — only remove when the tag is side-effect-free or the case is provably unreachable.

```js
// NOT safe to remove — var declaration must be hoisted
switch (1) {
  case 0:
    var x = 1;
    break;
  case 1:
    use(x);
    break; // x is undefined but declared
}
```

## References

- Terser: `switches` option. Constant discriminant evaluation, `branches_equivalent()` for body grouping, dead case pruning, switch→if conversion
- SWC: `remove_empty_switch`, `optimize_const_switches`, `optimize_small_switch` (1-2 case → if/if-else), `optimize_switch_with_default_on_last`. Uses `BreakFinder`/`contains_nested_break()`
- esbuild: `analyzeSwitchCasesForLiveness` for dead case elimination; switch unwrapping for static discriminant (v0.25.0+); switch→ternary reduction (v0.27.2+)
- Closure: `tryOptimizeSwitch` in `PeepholeRemoveDeadCode.java`. Two-pass: (1) remove useless cases, (2) fold literal discriminant via `evaluateComparison(SHEQ)`. `tryRemoveSwitchWithSingleCase` for single-case. `areAllCaseTagsLiterals` guard
