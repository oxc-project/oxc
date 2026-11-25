# Architecture

## Design Philosophy

Achieve maximum compression through comprehensive optimizations while maintaining correctness.

### Size Optimization Strategy

- Fixed-point iteration until no more changes or the iteration limit is reached
- Combine multiple optimization techniques
- Learn from all major minifiers
- Apply optimizations exhaustively

### Learning from Industry Leaders

#### From Closure Compiler (Size Focus)

- Advanced dead code elimination
- Aggressive constant folding
- Cross-statement optimizations
- Property flattening techniques
- Whole program analysis

#### From Terser/UglifyJS (Comprehensive)

- All peephole optimizations
- Battle-tested patterns
- Edge case handling
- Extensive transformation catalog

#### From esbuild (Performance)

- Minimal AST passes where possible
- Arena allocation strategy
- Efficient algorithms
- Smart traversal patterns

#### From SWC (Modern)

- Rust safety and performance
- Clean visitor pattern
- Parallel processing potential
- Modern architecture

## Core Components

### Compressor (`compressor.rs`)

Orchestrates the optimization pipeline with fixed-point iteration.

```rust
pub struct Compressor<'a> {
    allocator: &'a Allocator,
}
```

Key responsibilities:

- Builds semantic model
- Applies normalization
- Runs peephole optimizations to fixed-point
- Manages optimization state

### Peephole Optimizations (`peephole/`)

Transformation modules dispatched together during a peephole traversal, including:

- Constant folding
- Dead code elimination
- Control flow optimization
- Expression simplification
- Syntax substitution

Each optimization implements transformations in the AST traversal visitor pattern.

### Context (`traverse_context/`)

The crate's `TraverseCtx<'a>` alias refers to `MinifierTraverseCtx<'a>`.
It provides the AST builder, ancestry, scoping, and `MinifierState`.
ECMAScript evaluation and mutation helpers are in `traverse_context/ecma_context.rs`.

Use typed `replace_*` helpers for replacements and `drop_*` helpers for discarded
subtrees. They record removed references and dropped direct `eval` calls as well
as requesting another pass. Use `replace_*_with` when moving an old node into its
replacement; the closure must report discarded subtrees. `notice_change()` is
for in-place edits that retain their subtrees.

### State Management (`state.rs`, `symbol_state.rs`)

`MinifierState` stores configuration, per-body state, private-member usage, and
per-pass changes. `SymbolState` groups symbol values, metadata, and liveness.

### Pass Completion (`compression_pass.rs`)

After normalization and each peephole traversal, pass completion prunes removed
semantic references, refreshes direct-eval scope flags when needed, and updates
function reachability. Its outcome drives `Compressor::run_in_loop`; debug
guards check for references that were incorrectly removed or left behind.

## Optimization Pipeline

```text
1. Parse JavaScript (caller)
2. Optional property-name mangling
   └─> Collect, assign, and rewrite before compression
3. Compression (if enabled)
   ├─> Build semantic scoping
   ├─> Normalize and complete the normalization pass
   └─> Repeat peephole traversal and pass completion
        until no revisit is needed or the iteration limit is reached
4. Optional identifier/private-member mangling
   └─> Rebuild semantics for the compressed AST
5. Code generation (caller)
   └─> Pass scoping and class_private_mappings from MinifierReturn to Codegen
```

`Minifier::minify` selects full compression. `Minifier::dce` selects tree-shaking
compression, which keeps folds needed to expose dead branches but skips transforms
whose only purpose is shorter syntax. Mangling remains controlled by the supplied
`MinifierOptions`.

Callers of `Compressor::build_with_scoping` or
`dead_code_elimination_with_scoping` must supply scoping whose resolved-reference
lists match the current AST. Added references missing from those lists can cause
incorrect output; stale extra references can prevent optimizations.

## Memory Management

- Arena allocation via `oxc_allocator`
- Minimal cloning through `TakeIn` trait
- Efficient AST manipulation with arena pointers

## Traversal Strategy

Using the crate's generated traversal through `minifier_traverse.rs`:

- Enter/exit hooks for each node type
- Bottom-up transformations (exit handlers)
- State tracking through context

## Integration Points

- `oxc_ast`: AST node definitions
- `oxc_semantic`: Scope and symbol analysis
- `oxc_ecmascript`: ECMAScript operations
- `oxc_mangler`: Variable renaming
- `oxc_codegen`: Output generation
