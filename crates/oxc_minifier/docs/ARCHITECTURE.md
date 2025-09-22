# Architecture

## Design Philosophy

Achieve maximum compression through comprehensive optimizations while maintaining correctness.

### Size Optimization Strategy

- Fixed-point iteration until no more improvements
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

17+ transformation passes including:

- Constant folding
- Dead code elimination
- Control flow optimization
- Expression simplification
- Syntax substitution

Each optimization implements transformations in the AST traversal visitor pattern.

### Context (`ctx.rs`)

Shared utilities for optimizations:

- AST manipulation helpers
- Constant evaluation
- Side effect analysis
- Semantic utilities

```rust
pub struct Ctx<'a, 'b> {
    // Provides access to:
    // - AST builder
    // - Scoping information
    // - Symbol values
    // - Optimization options
}
```

### State Management (`state.rs`)

Tracks optimization state:

- Symbol values
- Changed flags
- Pure functions
- Source type

## Optimization Pipeline

```
1. Parse and Build Semantic Model
   └─> SemanticBuilder creates scoping and symbols

2. Normalization Pass
   └─> Convert to canonical form for optimizations

3. Peephole Optimization Loop
   └─> Apply all optimizations
   └─> Check if any changes made
   └─> Repeat until fixed-point (no changes)

4. Optional Mangling
   └─> Rename variables for size

5. Code Generation
   └─> Output optimized JavaScript
```

## Memory Management

- Arena allocation via `oxc_allocator`
- Minimal cloning through `TakeIn` trait
- Efficient AST manipulation with arena pointers

## Traversal Strategy

Using `oxc_traverse` for AST walking:

- Enter/exit hooks for each node type
- Bottom-up transformations (exit handlers)
- State tracking through context

## Integration Points

- `oxc_ast`: AST node definitions
- `oxc_semantic`: Scope and symbol analysis
- `oxc_ecmascript`: ECMAScript operations
- `oxc_mangler`: Variable renaming
- `oxc_codegen`: Output generation
