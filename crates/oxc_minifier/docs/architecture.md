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

## Design Plans

See [progress.md](progress.md) for a full list of design documents.
