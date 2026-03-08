# Architecture

## Overview

The oxc_minifier is a JavaScript/TypeScript minifier that achieves maximum compression
through fixed-point iteration of peephole optimizations.

## Source Layout

```
src/
├── lib.rs                  # Public API entry point
├── compressor.rs           # Main compression driver with fixed-point loop
├── options.rs              # Minifier configuration options
├── state.rs                # Shared mutable state across passes
├── keep_var.rs             # Variable declaration preservation
├── symbol_value.rs         # Constant value tracking for symbols
├── minifier_traverse.rs    # Top-level AST traversal dispatch
│
├── peephole/               # Peephole optimization passes
│   ├── mod.rs
│   ├── normalize.rs
│   ├── remove_dead_code.rs
│   ├── minimize_conditions.rs
│   ├── minimize_conditional_expression.rs
│   ├── minimize_if_statement.rs
│   ├── minimize_for_statement.rs
│   ├── minimize_logical_expression.rs
│   ├── minimize_not_expression.rs
│   ├── minimize_expression_in_boolean_context.rs
│   ├── minimize_statements.rs
│   ├── substitute_alternate_syntax.rs
│   ├── replace_known_methods.rs
│   ├── fold_constants.rs
│   ├── convert_to_dotted_properties.rs
│   ├── inline.rs
│   ├── remove_unused_declaration.rs
│   ├── remove_unused_expression.rs
│   └── remove_unused_private_members.rs
│
├── traverse_context/       # Traversal infrastructure
│   ├── mod.rs
│   ├── ancestry.rs         # Parent node tracking
│   ├── scoping.rs          # Scope and symbol management
│   ├── scopes_collector.rs # Scope collection during traversal
│   ├── ecma_context.rs     # ECMAScript context flags
│   ├── bound_identifier.rs
│   ├── maybe_bound_identifier.rs
│   ├── uid.rs              # Unique identifier generation
│   └── reusable.rs         # Reusable allocations
│
└── generated/              # Auto-generated (do not edit)
    ├── mod.rs
    ├── ancestor.rs
    ├── traverse.rs
    └── walk.rs
```

## Pipeline

1. **Parse** — AST is produced by `oxc_parser`
2. **Compress** — `Compressor` runs peephole passes in a fixed-point loop until no further changes occur
3. **Mangle** — Variable names are shortened (handled by `oxc_mangler`)
4. **Codegen** — Minified output is emitted by `oxc_codegen`

## Pass Ordering and Phase Design

The 4-step pipeline above is a simplification. Internally, compression has distinct phases
with specific ordering requirements. Understanding this structure is essential for knowing
where each optimization pass belongs and why.

This document describes the intended optimization architecture, not just the currently
implemented passes. A large part of the remaining work is not adding more local rewrites,
but defining the middle-end concepts that let advanced optimizations be both correct and
fast on real-world code.

### Phase Pipeline

```
  ┌─────────┐   ┌──────────┐   ┌──────────────────────┐   ┌────────────────────┐   ┌──────────┐   ┌─────────┐
  │  Parse  │──▶│ Semantic  │──▶│  Normalize + Analyze │──▶│  Optimization Loop │──▶│  Mangle  │──▶│ Codegen │
  │         │   │          │   │  (once)              │   │  (fixed-point)     │   │          │   │         │
  └─────────┘   └──────────┘   └──────────────────────┘   └────────────────────┘   └──────────┘   └─────────┘
                     │                    │
                     ▼                    ▼
              symbols, scopes,    call graph, escape analysis,
              CFG                 pure annotations, ref→block map
```

### Phase Descriptions

**Phase 1: Parse** — Produce the AST from source text.

**Phase 2: Semantic** — Build symbols, scopes, and CFG from the AST. This data is consumed
by all subsequent phases.

**Phase 3: Normalize + Analyze** — Rewrite AST into canonical forms and establish the
compiler reasoning that later optimizations depend on. Runs once before the loop.
Conceptually, this phase should provide:

- **Normalized AST** — canonical forms (split declarations, while→for, const→let, etc.)
  for simpler pattern matching
- **Define plugin replacements** — compile-time constants (`process.env.NODE_ENV`, `__DEV__`)
  replaced with literal values
- **Dataflow-oriented control-flow facts** — the ability to reason across basic blocks
  instead of only within local AST neighborhoods
- **Effect and alias reasoning** — the ability to tell when an expression can be removed,
  duplicated, reordered, or inlined without changing semantics
- **Pure function annotations** — `@__PURE__` / `@__NO_SIDE_EFFECTS__` annotations recognized
  and marked
- **Auto-detected pure functions** — functions analyzed for side effects and annotated
- **Light interprocedural analysis** — function ↔ call-site relationships, call counts,
  address-taken flags, and simple caller/callee summaries within a single file
- **Escape analysis** — per-variable tracking of whether values escape their declaring
  function
- **Reference-to-BasicBlock associations** — each variable reference tagged with its CFG
  basic block for dominance queries
- **Profitability signals** — enough size-oriented reasoning to distinguish transforms
  that are merely legal from transforms that are worth applying

**Phase 4: Optimization Loop** — Peephole passes run in a single traversal, repeated until
no changes. This is the core of the minifier.

**Phase 5: Mangle** — Variable and property renaming. Separate from compression. Rebuilds
semantic data.

**Phase 6: Codegen** — Emit minified output with codegen-level optimizations.

### Missing Middle-End Concepts

Many of the planned passes depend on four shared compiler concepts that are still only
partially expressed in the current docs.

**Dataflow framework**

This is the ability to reason through control flow instead of only through syntax. In a
minifier, the important questions are usually: where can a value reach, where is a symbol
live, and does a definition dominate a use. Without that, optimizations such as dead
assignment elimination and flow-sensitive inlining remain heuristic.

**Effect and alias model**

Minification is constrained more by semantic safety than by pattern matching. The optimizer
needs a way to reason about whether an expression may throw, read mutable state, write
mutable state, or observe aliasing through objects, closures, getters, setters, or unknown
calls. This is what separates "can rewrite" from "must preserve as written."

**Light interprocedural analysis**

Even in single-file mode, many important opportunities span function boundaries. The
minifier needs lightweight reasoning about direct callees, argument usage, return usage,
escape of function values, and small purity summaries. This is not whole-program analysis;
it is a constrained, file-local form of function reasoning.

**Profitability model**

Not every legal rewrite helps compression, and some rewrites help size but hurt compile
time too much. The optimizer therefore needs a concept of profitability: byte-oriented
reasoning about whether a transform is worthwhile, when fixed-point iteration should stop,
and which more expensive analyses should even run.

### Pass Classification

Each of the 39 design documents maps to exactly one phase. The `#` column refers to the
design document number in [progress.md](progress.md).

**Phase 3 — Normalize + Analyze (pre-loop, run once)**

| #   | Pass                              | Rationale                                                         |
| --- | --------------------------------- | ----------------------------------------------------------------- |
| 002 | Normalize                         | Canonical AST forms for downstream pattern matching               |
| 005 | Module-Aware Optimizations        | Apply ES module semantics (strict mode, this=undefined)           |
| 023 | Drop Statements                   | Remove debugger/console before optimization begins                |
| 024 | Define Plugin                     | Replace compile-time constants before fold constants can use them |
| 025 | Pure Annotations and Side Effects | Annotate pure calls for DCE to consume                            |
| 038 | Mark Pure Functions               | Auto-detect pure functions by analyzing bodies                    |

**Phase 4 — Optimization Loop (peephole, iterated to fixed point)**

| #   | Pass                         | Rationale                                                                     |
| --- | ---------------------------- | ----------------------------------------------------------------------------- |
| 003 | Substitute Alternate Syntax  | Local rewrites: shorter syntax forms                                          |
| 004 | Convert to Dotted Properties | `a["b"]` → `a.b`                                                              |
| 006 | Fold Constants               | Evaluate constant expressions                                                 |
| 007 | Replace Known Methods        | Evaluate known built-in methods                                               |
| 008 | Minimize Conditions          | Simplify conditional expressions                                              |
| 009 | Remove Dead Code             | Eliminate unreachable branches                                                |
| 010 | Collect Property Assignments | Merge property assignments into initializers                                  |
| 011 | Statement Fusion             | Fuse consecutive expression statements                                        |
| 012 | Minimize Exit Points         | Remove redundant return/break/continue                                        |
| 013 | Exploit Assigns              | Combine assignments into expressions                                          |
| 014 | Function to Arrow            | Convert eligible functions to arrow syntax                                    |
| 015 | Replace Arguments Access     | Replace `arguments[i]` with named parameters                                  |
| 016 | Optimize Loops               | Loop-specific simplifications                                                 |
| 017 | Optimize Switch              | Switch statement optimizations                                                |
| 018 | Remove Unused Code           | Mark-and-sweep unused declaration removal                                     |
| 019 | Inline                       | Variable, function, and property inlining                                     |
| 026 | Optimize Parameters          | Remove unused trailing parameters                                             |
| 027 | String Deduplication         | Extract repeated string literals into shared variables                        |
| 029 | Collapse Declarations        | Join consecutive var/let/const; collapse function expressions to declarations |
| 030 | Modern Syntax Optimizations  | Use modern JS features for shorter output                                     |
| 031 | TypeScript Optimizations     | TS-specific size reductions                                                   |
| 033 | Optimize Calls               | Call-site and return-value optimizations                                      |
| 036 | Inline Simple Methods        | Inline trivial method bodies at call sites                                    |
| 037 | Extract Prototype Members    | Merge prototype property assignments into compound form                       |

**Phase 4b — Optimization Loop, advanced passes (need CFG/dataflow)**

These run inside the loop but require infrastructure not yet built:

| #   | Pass                         | Dependency                                                |
| --- | ---------------------------- | --------------------------------------------------------- |
| 021 | Dead Assignments Elimination | CFG + liveness dataflow + effect reasoning                |
| 022 | Collapse Variables           | CFG + reaching definitions + alias/effect + profitability |
| 032 | Hoist Properties             | Escape analysis + alias reasoning                         |
| 035 | Flow-Sensitive Inline        | CFG + reaching definitions + effect reasoning             |

**Phase 5 — Mangle (separate from compression)**

| #   | Pass                 | Rationale                            |
| --- | -------------------- | ------------------------------------ |
| 020 | Mangle Properties    | Property renaming                    |
| 034 | Variable Mangling    | Variable renaming via scope analysis |
| 039 | Ambiguate Properties | Cross-type property name reuse       |

**Phase 6 — Codegen**

| #   | Pass                  | Rationale                                          |
| --- | --------------------- | -------------------------------------------------- |
| 028 | Codegen Optimizations | Emit-time decisions (number formats, quote styles) |

### Visitor Ordering Within a Peephole Traversal

All Phase 4 passes execute within a single AST traversal. Their ordering matters because
one pass's output feeds into another.

**Enter visitors (top-down):**

- Collect symbol metadata (pure function annotations, symbol values)
- Push class scopes for private member tracking

**Exit visitors (bottom-up):**
Most optimization fires on exit, because children are optimized before parents.

**Ordering constraints (exit phase):**

1. **Fold constants** before **remove dead code** — folded constants reveal dead branches
2. **Define plugin** before **fold constants** — replaced constants enable folding
3. **Pure annotations** before **remove dead code** — marked pure calls can be removed
4. **Minimize conditions** before **fold if** — simplified tests enable branch folding
5. **All expression optimizations** before **remove unused assignments** — don't remove
   what might get folded
6. **Inline** after **fold constants** — inlined values should already be folded

### Closure Compiler Correspondence

Our phase structure parallels Closure Compiler's pipeline:

- **Phase 3** ≈ Closure's `normalize` + early optimization passes (`ReplaceIdGenerators`,
  `MarkNoSideEffectCalls`)
- **Phase 4** ≈ Closure's `PeepholeOptimizationsPass` within `optimizeLoops()`
- **Phase 4b** ≈ Closure's `FlowSensitiveInlineVariables`, `DeadAssignmentsElimination`
- **Phase 5** ≈ Closure's `RenameVars`, `RenameProperties`, `AmbiguateProperties`
- **Phase 6** ≈ Closure's `CodePrinter` with its built-in optimizations

## Fixed-Point Loop

Optimization passes interact — one pass's output often enables another pass to fire. Constant folding
may produce a dead branch that dead code elimination can remove, which may leave a variable unused
for unused code removal to clean up. A single traversal cannot catch all these cascading opportunities.

The compressor therefore runs all optimization passes inside a **fixed-point loop**: traverse the AST,
apply all passes, and repeat until an iteration produces no changes (convergence). Closure Compiler
uses the same approach, with a convergence heuristic that stops when consecutive iterations yield
less than 0.05% size reduction, plus a safety cap of 100 maximum iterations to guarantee termination
even if the convergence threshold is never met.

Conceptually, this loop also depends on a profitability model. "Changed the AST" is not the
same thing as "improved the output." Some transforms expose future wins, but others merely
shuffle syntax or even increase emitted size. The mature version of this loop therefore needs
to distinguish legality from profitability and use size-oriented heuristics to decide when
another iteration is justified.

### Selective subtree traversal

The natural unit for revisiting is the **function**. Functions are self-contained scopes — most
optimizations operate within a single function body, and the results of optimizing one function
rarely affect the internals of another. After the first iteration, only functions whose bodies
were modified (or whose external context changed, e.g. a call-site argument was simplified)
need to be re-visited.

The fixed-point loop should therefore track which functions changed and only re-traverse those
on subsequent iterations. This applies to all optimization passes (not just peepholes) — any pass
that modifies the AST within a function marks that function for re-visitation.

Crucially, optimizing one function can dirty _another_. Removing a call to `f()` while
processing its caller leaves `f` with a potentially-unused parameter; inlining a variable
from an outer scope changes the reference counts visible to that scope's function. The dirty
marking mechanism must therefore allow a pass to mark any function — not just the one currently
being traversed — for re-visitation on the next iteration.

The traversal needs a mechanism to **skip functions that haven't changed** since the last
iteration. Closure Compiler provides this through `shouldTraverse(node)` — a predicate called
before descending into a node's children. This separates two concerns:

1. **The mechanism** — how the traversal skips (a predicate checked before descent)
2. **The policy** — what decides which functions to skip (change tracking)

Closure Compiler implements this with a **monotonic change counter** and **per-function-scope
timestamps**. Each pass increments a global counter when it modifies the AST. Each function scope
records the counter value at which it was last optimized. On the next iteration, a function is
skipped if its timestamp equals the current counter (no changes since last visit). The
`shouldTraverse(node)` predicate checks this timestamp before descending into a function body.

Both concerns are unsolved for the new minifier. The generated walk functions currently always
recurse into all children unconditionally, and there is no change tracking beyond a single
global boolean.

## Symbol and Scope Synchronization

Optimization passes mutate the AST: they remove declarations, inline variables, eliminate dead
branches, and unwrap block scopes. Each mutation can make the symbol table and scope tree stale,
because those structures are built once upfront and not automatically kept in sync with AST
changes. Passes depend on this data for correctness — `symbol_is_unused()` gates whether a
declaration can be removed, reference counts determine inlining eligibility, and scope parent
pointers are used for name resolution.

The single-pass model (all peephole passes in one traversal) makes this harder: there is no
opportunity to rebuild scope/symbol data between individual passes the way Terser does with
`figure_out_scope()`. Updates must happen in-traversal — each pass must incrementally fix up
reference counts, scope bindings, and symbol flags as it mutates the AST.

### How other minifiers handle this

**Closure Compiler** builds scopes lazily. `SyntacticScopeCreator` constructs scope objects on
demand during each `NodeTraversal`, so every traversal sees fresh data derived from the current
AST state. `MemoizedScopeCreator` caches the results, but passes explicitly invalidate caches
by calling `reportChangeToEnclosingScope()` when they modify declarations. Scope caches are
also cleared at phase boundaries (type checking → optimization → codegen).

**Terser** rebuilds from scratch. `figure_out_scope()` walks the entire AST and reconstructs
all scope and variable data — parent pointers, enclosed variable sets, symbol definitions. This
is called between compression passes to ensure each pass starts with accurate data.

**esbuild** sidesteps the problem. Identifiers are referenced by 64-bit symbol IDs rather than
names, so renaming cannot cause resolution errors. Parsing, scope construction, and optimization
are merged into 2–3 tightly coupled passes rather than iterated independently, which minimizes
the window where data can go stale.

**SWC** uses strict pass ordering. A `resolver` pass runs first to assign `SyntaxContext` marks
that encode scope chains directly on identifier nodes. Transforms run next without needing to
query or update a separate scope tree. A `hygiene` pass runs last to rename identifiers whose
marks conflict, restoring valid JavaScript output.

### Oxc's Approach: Incremental Cleanup

Oxc uses incremental cleanup per mutation (Closure Compiler style), not batch rebuild (Terser
style). Each pass that mutates the AST calls cleanup helpers immediately after the mutation.
No deferred reconciliation pass is needed — scope and symbol data stays accurate throughout
the traversal, so subsequent passes within the same iteration see correct reference counts
and scope structure.

### Mutation Categories and Required Cleanup

Five categories of AST mutations require scope/symbol bookkeeping:

**DELETE — Remove an AST subtree**

When removing dead code, unused declarations, or unreachable branches.

- Walk the removed subtree, delete every reference
- Remove bindings for any declarations
- Reparent orphaned child scopes to the removed node's parent scope

**REPLACE — Swap one expression for another**

When inlining a variable (replacing identifier with its value) or folding a constant.

- Delete references in the old expression
- The new expression (a literal, `void 0`, etc.) typically has no references
- If the replacement introduces new identifiers, create references for them

**MOVE — Relocate an AST subtree between scopes**

When inlining a function body at a call site, or hoisting properties out of an object.

- Reparent all scopes within the moved subtree to the new parent scope
- Move bindings: for each declaration in the moved subtree, transfer the binding from the old
  scope to the new scope
- References within the moved subtree may need re-resolution if they now resolve to different
  symbols (e.g., a parameter that becomes a local variable at the call site)
- References from _outside_ the moved subtree to symbols _inside_ it: these still point to the
  same `SymbolId` (stable), so no reference update is needed — only the symbol's owning scope
  changes

**CREATE — Introduce new declarations**

When string deduplication creates shared variables, or when a pass introduces temporaries.

- Generate fresh symbols and add bindings to the target scope
- Create references at each use site
- No cleanup of existing data needed

**MERGE — Collapse a scope into its parent**

When unwrapping a block scope or removing an IIFE wrapper.

- Move all bindings from inner scope to parent scope
- Reparent inner scope's child scopes to parent scope
- The inner scope becomes empty and orphaned (harmless in arena allocation)

### Cleanup Helpers

Rather than requiring every pass to manually call low-level scoping methods (error-prone, easy
to miss steps), provide mid-level helpers that bundle the required operations:

**`delete_references_in_expression(expr)`** — Walk an expression subtree and delete all
references found. Call this before removing any expression from the AST. Implemented as a small
visitor over identifier references.

**`delete_references_in_statement(stmt)`** — Same for statement subtrees.

**`remove_declaration(symbol_id)`** — Remove a symbol's binding from its scope and delete all
its resolved references. Single call replaces multi-step manual cleanup.

**`move_declaration(symbol_id, new_scope_id)`** — Move a declaration's binding from its current
scope to a new scope. Updates the symbol's owning scope and transfers the binding entry.

**`reparent_child_scopes(subtree, new_parent_scope)`** — Walk a subtree, find all immediate
child scopes, reparent them to the new parent. Used when removing a scope boundary or moving
code.

These compose naturally for common operations:

```
// Removing dead code:
delete_references_in_statement(dead_branch)

// Inlining a variable reference:
delete_reference(ident)  // the replaced reference
// then replace identifier with value in AST

// Removing an unused function:
remove_declaration(symbol_id)
delete_references_in_statement(function_body)

// Inlining a function body at a call site:
for each parameter symbol:
    move_declaration(param_symbol, caller_scope)
reparent_child_scopes(body, caller_scope)
// then splice body into caller
```

### Reference Count Queries

Optimization passes need reference counts (how many reads/writes a symbol has) for inlining
and removal decisions. Two design options:

**Cache counts at iteration start** — Compute read/write counts once when entering the
traversal. Fast lookups but counts go stale as passes delete references within the same
traversal, causing missed optimizations.

**Query counts live** — Call `get_resolved_reference_ids(symbol_id).len()` each time. Always
accurate. O(1) cost (vector length). No staleness. Enables cascade removal within a single
traversal: removing one declaration makes another symbol unreferenced, which can be removed
immediately without waiting for the next iteration.

Live queries are preferred. Cached metadata should be limited to properties that don't change
during optimization: the symbol's constant value (if any), whether it's exported, and its
owning scope.

## Design Plans

See [progress.md](progress.md) for a full list of design documents.
