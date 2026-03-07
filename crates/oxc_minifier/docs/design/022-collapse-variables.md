# Collapse Variables

- **Status:** Not Implemented
- **Difficulty:** Very Complex

## What

Flow-sensitive variable collapsing — inline non-constant single-use variables by tracking assignments through control flow. This extends beyond 015-inline.md which handles only constants, aliases, and simple single-use cases without flow sensitivity.

## Why

Many variables exist only to name an intermediate result that is used once. Collapsing these eliminates `var` declarations, reduces scope pressure, and shortens code. This is among the most impactful optimization passes — Terser's `collapse_vars` and Closure's `FlowSensitiveInlineVariables` together account for significant size reductions on real-world code.

## Transformations

### Collapse single-use non-constant variables

Move the assigned value to the use site when no intervening side effects modify referenced state.

```js
// Before
var x = a + b;
f(x);

// After
f(a + b);
```

### Collapse across expressions

When an assignment's value can be safely moved past intervening expressions.

```js
// Before
var x = obj.method();
var y = pure();
use(x, y);

// After
var y = pure();
use(obj.method(), y);
```

Note: This requires verifying that `pure()` does not modify state that `obj.method()` depends on.

### Reduce variables (annotate effective constness)

Track all assignments to a variable. When a variable has a single assignment that dominates all reads and is never reassigned, annotate it as effectively constant (`fixed`). This enables multi-use inlining for non-literal values.

```js
// Before
var x = expensive();  // assigned once, read twice, never reassigned
use(x);
use(x);

// After (if `expensive()` is pure)
use(expensive());
use(expensive());
```

Note: Only profitable when the expression is pure and shorter than the variable name (post-mangling).

### Collapse into return

```js
// Before
var x = expr;
return x;

// After
return expr;
```

### Collapse into assignment

```js
// Before
var x = expr;
y = x;

// After
y = expr;
```

## How it works

Three approaches exist across minifiers:

### 1. CFG + dataflow (Closure)

Forward analysis (`MustBeReachingVariableDef`) determines the single reaching definition. Backward analysis (`MaybeReachingVariableUse`) determines the single reaching use. Inline only when: single def, single use, no side effects on any path between def and use, not in a loop, RHS has no order-dependent side effects.

### 2. Scan-and-fold (Terser)

`extract_candidates()` scans right-to-left for assignment candidates. Safety checked via `get_lvalues`, `shadows`, `side_effects_external`. A separate `reduce_vars` pre-pass annotates `def.fixed` for each variable (tracks whether a variable has a single, fixed value). Aborts at: await, yield, try, with, class, iteration statements, loop control.

### 3. Conservative sequential (esbuild)

Single-pass, ordering-safe substitution. `UseCountEstimate == 1` triggers inlining. Side-effect ordering verified by checking if primitives can skip past side-effecting expressions. No flow sensitivity, no fixed-point iteration.

## Safety constraints

These constraints are the union of all minifiers' requirements:

- **No side effects between def and use** that could modify referenced variables
- **Evaluation order preserved** — surrounding expression evaluation order must not change
- **RHS validity at new location** — the right-hand side must not reference variables with different values at the substitution site
- **Loop boundaries** — not inside a loop (Closure) or same loop iteration only (Terser)
- **`arguments` usage** blocks parameter inlining (mutating params affects `arguments` in sloppy mode)
- **`eval` presence** blocks all inlining — `eval` can access any variable by name
- **Cannot reorder past**: `await`, `yield`, `try`/`catch`, `with`, class expressions
- **Getters/setters** — property access may have side effects, blocking reordering

```js
// NOT safe — side effect between def and use
var x = obj.a;
obj.a = 42;  // modifies what x read
f(x);        // x must remain 'obj.a' before mutation

// NOT safe — inside loop
for (var i = 0; i < n; i++) {
  var x = arr[i];  // different value each iteration
  use(x);          // cannot collapse across iterations
}

// NOT safe — eval
var x = 1;
eval("x = 2");
use(x);  // x may have been modified by eval
```

## References

- Terser: `collapse_vars` in `tighten-body.js`, `reduce_vars` in `reduce-vars.js`. Tracks `fixed`, `escaped`, `single_use`, `direct_access` per definition
- SWC: `compress/optimize/inline.rs`. Uses `VarUsageInfo` with `REASSIGNED`, `ref_count`, `assign_count` flags. Multi-use inlining only for "simple enough" expressions
- esbuild: Conservative single-pass in `js_parser.go`. Only within function scope. Fixed in v0.15.12 for read-modify-write operator ordering
- Closure: `FlowSensitiveInlineVariables.java`. Requires exactly one reaching def AND one reaching use. `CheckPathsBetweenNodes` verifies no side effects on any path between def and use in different CFG nodes
