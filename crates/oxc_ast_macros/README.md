# Oxc AST Macros

Procedural macros for generating AST-related code and ensuring memory layout consistency.

## Overview

This crate marks AST types and enforces the memory layouts recorded by `oxc_ast_tools`. Trait implementations are generated ahead of time by `just ast`, rather than by the attribute macro.

## Key Features

- **`#[ast]` attribute**: Marks types for AST tooling, enables helper attributes, and checks requested generated traits
- **Memory layout control**: Adds `#[repr(C)]` or `#[repr(transparent)]` for structs, and `#[repr(u8)]` or `#[repr(C, u8)]` for enums
- **Trait generation metadata**: Supports attributes consumed by `oxc_ast_tools`, such as `#[generate_derive(CloneIn)]`
- **Code generation**: Reduces boilerplate and ensures consistency across AST types

## What the `#[ast]` Macro Does

1. **Adds a representation attribute**: Uses the generated layout metadata to select `#[repr(C)]` or `#[repr(transparent)]` for structs, `#[repr(u8)]` for fieldless enums, and `#[repr(C, u8)]` for fieldful enums
2. **Marker for tooling**: Identifies types as AST nodes for code generation tools
3. **Trait metadata**: Enables helper attributes through the no-op `Ast` derive and checks that requested generated traits are in scope; `oxc_ast_tools` generates their implementations
4. **Consistency**: Ensures all AST nodes follow the same patterns

Foreign metadata types marked with `#[ast(foreign = ...)]` bypass representation insertion.
After changing AST types or `#[generate_derive(...)]` metadata, run `just ast` to regenerate the
implementations and layout metadata. Do not edit files under `generated/` directly.

## Architecture

This macro system enables:

- **Maintainable AST**: Reduces boilerplate across hundreds of AST types
- **Consistent layout**: Critical for performance and correctness
- **Tool integration**: Allows `oxc_ast_tools` to generate visitor code
- **Type safety**: Ensures all AST nodes have required traits

The macros are designed to be transparent and generate minimal, efficient code.
