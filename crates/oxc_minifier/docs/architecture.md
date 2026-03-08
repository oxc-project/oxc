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

## Fixed-Point Loop

Optimization passes interact — one pass's output often enables another pass to fire. Constant folding
may produce a dead branch that dead code elimination can remove, which may leave a variable unused
for unused code removal to clean up. A single traversal cannot catch all these cascading opportunities.

The compressor therefore runs all optimization passes inside a **fixed-point loop**: traverse the AST,
apply all passes, and repeat until an iteration produces no changes (convergence). Closure Compiler
uses the same approach, with a convergence heuristic that stops when consecutive iterations yield
less than 0.05% size reduction.

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

## Design Plans

See [progress.md](progress.md) for a full list of design documents.
