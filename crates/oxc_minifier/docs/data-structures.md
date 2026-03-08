# Data Structures

These are the shared infrastructure data structures consumed by the minifier. Most are built by `oxc_semantic` and `oxc_cfg` during a single AST traversal, then queried (read-only or with targeted mutations) throughout the optimization pipeline.

## Symbol Table

### What

A struct-of-arrays table that maps each `SymbolId` to its metadata: source span, declaration flags, owning scope, and declaration AST node. Defined via the `multi_index_vec!` macro in `oxc_semantic::scoping`:

```rust
struct SymbolTable<SymbolId> {
    symbol_spans: Span,
    symbol_flags: SymbolFlags,
    symbol_scope_ids: ScopeId,
    symbol_declarations: NodeId,
}
```

Symbol names are arena-allocated strings stored separately in `ScopingInner::symbol_names`. Resolved references are stored as `ArenaVec<ArenaVec<ReferenceId>>` per symbol, with each `Reference` carrying read/write flags and a `SymbolId` (if resolved).

### Why

Nearly every minification pass queries the symbol table:

- **Inlining** (015) checks whether a variable is assigned once and read once (single-use) or assigned a constant value
- **Dead code** (013) checks whether a symbol has zero references
- **Mangling** ranks symbols by reference frequency to assign the shortest names to the most-used identifiers
- **Tree shaking** checks `@__NO_SIDE_EFFECTS__` and `@__PURE__` annotations stored in `Scoping::no_side_effects`

### How It Works

`SemanticBuilder` populates the table in a single AST traversal. Each `let`/`const`/`var`/function/class/parameter declaration pushes a row into `SymbolTable` and records the name in the arena. When a reference is encountered, it resolves to a `SymbolId` by walking up the scope chain and is appended to that symbol's resolved reference list.

### References

- Closure Compiler: `SymbolTable.java`, `Var.java`
- esbuild: `js_parser.go` — `Symbol` struct and `SymbolKind`
- Terser: `lib/scope.js` — `SymbolDef` class

## Scope Tree

### What

A tree of lexical scopes stored as another struct-of-arrays table:

```rust
struct ScopeTable<ScopeId> {
    parent_ids: Option<ScopeId>,
    node_ids: NodeId,
    flags: ScopeFlags,
}
```

Bindings (name → `SymbolId` mappings) are stored per scope in `ScopingInner::bindings` as `IndexVec<ScopeId, Bindings>`. The tree uses parent pointers only — there are no child lists.

### Why

- **Name resolution** walks up the parent chain to find the declaring scope of an identifier
- **Mangling** exploits the tree structure to reuse short names via shadowing — two variables in non-overlapping scopes can share the same mangled name
- **Scope flags** (`ScopeFlags`) mark strict mode, function boundaries, arrow functions, and catch clauses — these affect which optimizations are safe
- **`eval`/`with` detection** via scope flags disables name mangling and certain inlining in affected scopes

### How It Works

`SemanticBuilder` opens a new scope (pushes a row into `ScopeTable`) at each scope-creating AST node (function, block, catch, etc.) and closes it on exit. Bindings are inserted as declarations are encountered. The parent-pointer structure is compact and sufficient because minification passes only need upward traversal (child iteration is not required).

### References

- Closure Compiler: `Scope.java`, `SyntacticScopeCreator.java`
- esbuild: two-pass scope system in `js_parser.go` — first pass discovers declarations, second pass resolves references
- Terser: `AST_Scope` and `figure_out_scope()` in `lib/scope.js`

## Control Flow Graph

### What

A directed graph of basic blocks connected by typed edges. Defined in `oxc_cfg`:

- `ControlFlowGraph` — top-level struct holding `petgraph::DiGraph<BasicBlockId, EdgeType>` plus `IndexVec<BasicBlockId, BasicBlock>`
- `BasicBlock` — a sequence of `Instruction` values (statement-level granularity)
- `InstructionKind` — `Statement`, `Return`, `Break`, `Continue`, `Throw`, `Condition`, `Iteration`
- `EdgeType` — `Jump`, `Normal`, `Backedge`, `NewFunction`, `Finalize`, `Error`, `Unreachable`, `Join`

### Why

The CFG is required for any analysis that reasons about execution order beyond syntactic nesting:

- **Liveness analysis** for dead assignment elimination (012)
- **Reaching definitions** for flow-sensitive constant propagation and inlining (015)
- **Reachability** — detecting unreachable code after `return`/`throw`/`break`
- **Infinite loop detection** — loops with no exit edges

### How It Works

`ControlFlowGraphBuilder` constructs the CFG during `SemanticBuilder`'s AST traversal. It maintains a context stack for nested break/continue targets and an error harness stack for try/catch/finally. Each statement becomes an instruction in the current basic block. Branching statements (if/switch/loops) terminate the current block and create edges to successor blocks. Exception-capable statements get `Error` edges to the enclosing catch handler.

### References

- Closure Compiler: `ControlFlowGraph.java`, `ControlFlowAnalysis.java`
- esbuild: no formal CFG — uses syntactic analysis only
- Terser: no formal CFG — uses syntactic analysis with `reduce_vars` for limited flow sensitivity

## Call Graph

### What

A bidirectional map of function ↔ call-site relationships. Each edge connects a call expression to the function it invokes. Dynamic dispatch (computed property calls, `eval`) is marked as having unknown targets.

### Why

- **Function inlining** (015) — determines whether a function is called exactly once (safe to inline without code size increase) or is small enough to inline at multiple sites
- **Dead function elimination** (013) — functions with zero incoming call edges and no escaping references can be removed
- **Cross-function constant propagation** — if all call sites pass the same constant for a parameter, that parameter can be replaced with the constant inside the function body

### How It Works

An AST walk registers every function declaration/expression and every call expression. Call targets are resolved via the symbol table: if the callee is an identifier that resolves to a `SymbolId` pointing to a function declaration, a direct edge is created. IIFEs (`(function(){...})()`) and immediately-invoked arrow functions are recognized as special cases with a known single call site. Unresolvable callees (member expressions, computed calls) produce edges to an "unknown" sentinel.

Per function, the call graph tracks:

- **Call count** — how many call sites invoke this function (single-call functions are safe to inline without size increase)
- **Address taken** — whether the function has any non-call reference (assigned to a variable, passed as an argument, stored in a property). If the address is taken, the function may be called from unknown sites, disabling optimizations that assume a known set of callers
- **IIFE status** — whether the function is immediately invoked at its declaration site

### References

- Closure Compiler: `CallGraph.java`, `DefinitionUseSiteFinder.java`
- esbuild: `js_parser.go` — tracks function call counts for inlining decisions
- Terser: `compress/reduce-vars.js` — tracks function call counts for inlining decisions

## Data Flow Analysis

### What

A generic framework for computing properties at each program point by propagating information along CFG edges. Instances include liveness analysis, reaching definitions, and constant propagation.

### Why

- **Dead assignment elimination** (012) — liveness analysis identifies assignments to variables that are never subsequently read
- **Flow-sensitive inlining** (015) — reaching definitions determine which value a variable holds at a given use site
- **Loop-invariant code motion** — constant propagation identifies expressions whose value does not change across loop iterations

### How It Works

The framework uses a worklist algorithm over CFG basic blocks:

1. Initialize IN/OUT sets for each block
2. Add all blocks to the worklist
3. While the worklist is non-empty, pop a block, apply the transfer function, and if the output changed, add successors (or predecessors for backward analyses) to the worklist
4. Repeat until fixed point

The framework is parameterized by:

- **Direction** — forward (entry → exit) or backward (exit → entry)
- **Lattice type** — the domain of values at each program point (bit sets, constant lattice, etc.)
- **Transfer function** — gen/kill sets that define how each block transforms the data flow state
- **Meet operator** — union for may-analyses, intersection for must-analyses

Key instances:

- **Liveness** (backward, union): `IN[b] = USE[b] ∪ (OUT[b] - DEF[b])`
- **Reaching definitions** (forward, union): `OUT[b] = GEN[b] ∪ (IN[b] - KILL[b])`
- **Constant propagation** (forward, lattice join): values flow through a lattice of ⊤ → constant → ⊥

### References

- Closure Compiler: `DataFlowAnalysis.java`, `LiveVariablesAnalysis.java`, `MustBeReachingVariableDef.java`
- esbuild: no formal data flow framework — uses syntactic heuristics
- Terser: `compress/reduce-vars.js` (single-pass variable tracking), `compress/evaluate.js` (constant evaluation)

## Interference Graph & Graph Coloring

### What

An undirected graph where edges connect variables that are simultaneously live at any program point. Greedy graph coloring assigns "colors" (variable slots) so no two adjacent nodes share a color.

### Why

- **Variable coalescing (022)** — determines which variables can safely share the same name/slot because their live ranges don't overlap
- **Variable mangling (034)** — enables sibling scope name reuse; variables in non-overlapping scopes get the same short name
- **Dead assignments elimination (021)** — liveness analysis (the input to interference graph construction) identifies dead stores

### How It Works

Build from liveness analysis output: for each program point, all simultaneously-live variables get pairwise edges. Then greedy graph coloring (sorted by degree or frequency) assigns minimum colors. Variables with the same color can share a name.

### References

- Closure Compiler: `GraphColoring.java`, `LiveVariablesAnalysis.java`, `CoalesceVariableNames.java`

## Type Inference (Value Type Tracking)

### What

Lightweight type predicates (`is_string()`, `is_number()`, `is_boolean()`, `is_bigint()`, `is_32_bit_integer()`) that propagate through expressions without a full type system.

### Why

- **Peephole fold constants (006)** — determines when arithmetic/string operations can be constant-folded (e.g., `"a" + "b"` requires knowing both sides are strings)
- **Peephole minimize conditions (008)** — enables `===` to `==` conversion when operand types are known to match (no coercion risk)
- **Peephole substitute alternate syntax (003)** — `typeof x === "undefined"` optimization requires knowing `x` is not a special type
- **Peephole replace known methods (007)** — method call evaluation requires knowing receiver type (e.g., `.length` on string vs array)
- **Modern syntax optimizations (030)** — context-aware numeric optimization (string-to-number in numeric context)

### How It Works

Chains through AST: sequences → type of last element; assignments → type of RHS; conditionals → type if both branches agree; binary ops → known result type (e.g., `+` with two numbers → number, `+` with a string → string). Does NOT use full type inference — just local propagation.

### References

- Terser: `inference.js`
- SWC: `compress/optimize/evaluate.rs`
- `oxc_ecmascript` crate

## Escape Analysis

### What

Tracks whether values escape their declaring function — i.e., whether a variable's value can be observed outside the function that creates it. A value escapes if it is returned, assigned to an outer-scope variable, stored in a property of an escaping object, or passed as an argument to a function that is not fully analyzable.

### Why

- **Hoist properties (032)** — an object can only be decomposed into standalone variables if it does not escape. If the object is passed to another function or returned, external code may observe the object as a whole, and property hoisting would break that contract
- **Stack allocation** — values that do not escape can be allocated on the stack or arena rather than the heap (future optimization)

### How It Works

For each variable, the analysis walks all references and checks whether any reference causes the value to leave its declaring scope. A reference escapes if it appears as: a function call argument (unless the callee is known and analyzed), a return value, an assignment to a variable in an outer scope, or a property write on an escaping object. The analysis is conservative — if any reference is ambiguous, the value is marked as escaping.

### References

- Closure Compiler: `ReferenceCollection.isEscaped()` — checks whether a collected reference escapes its declaring function
- esbuild: no formal escape analysis
- Terser: no formal escape analysis — uses heuristics in `hoist_props`

## Reference-to-BasicBlock Association

### What

Associates each variable reference with the CFG basic block in which it occurs. This is an augmentation of the existing reference data (which tracks symbol, read/write flags, and span) with control flow context.

### Why

This association enables two key queries:

- **`is_well_defined(ref)`** — the variable's assignment dominates all uses (the assignment's basic block dominates the reference's basic block in the CFG). This is required for safe inlining: a variable can only be inlined at a use site if the assigned value is guaranteed to have been computed by that point
- **`is_escaped(ref)`** — the reference is in a different hoist scope (function) than the variable's declaration. Combined with control flow, this detects cases where a closure captures a variable that may be reassigned between capture and invocation

### How It Works

During scope analysis (or as a post-pass over the reference list), each reference is tagged with the `BasicBlockId` of the block it falls within. The CFG's dominator tree then answers dominance queries: given an assignment in block A and a use in block B, the assignment reaches the use if A dominates B in the dominator tree.

### References

- Closure Compiler: `ReferenceCollector` stores `BasicBlock` context with each collected reference, used by `FlowSensitiveInlineVariables`
- esbuild: no formal reference-to-block mapping
- Terser: no formal reference-to-block mapping — uses syntactic heuristics in `reduce-vars.js`
