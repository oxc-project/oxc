# Oxc AST Macros

Procedural macros for generating AST-related code and ensuring memory layout consistency.

## Overview

This crate provides procedural macros that generate boilerplate code for AST nodes, ensuring consistent memory layout and providing derived traits automatically.

## Key Features

- **`#[ast]` attribute**: Marks types as AST nodes and generates required traits
- **Memory layout control**: Ensures `#[repr(C)]` for predictable memory layout
- **Trait derivation**: Automatically derives common traits like `Debug`, `Clone`, etc.
- **Code generation**: Reduces boilerplate and ensures consistency across AST types

## What the `#[ast]` Macro Does

1. **Adds `#[repr(C)]`**: Ensures predictable memory layout across platforms
2. **Marker for tooling**: Identifies types as AST nodes for code generation tools
3. **Trait derivation**: Automatically implements common traits
4. **Consistency**: Ensures all AST nodes follow the same patterns

## Architecture

This macro system enables:

- **Maintainable AST**: Reduces boilerplate across hundreds of AST types
- **Consistent layout**: Critical for performance and correctness
- **Tool integration**: Allows `oxc_ast_tools` to generate visitor code
- **Type safety**: Ensures all AST nodes have required traits

The macros are designed to be transparent and generate minimal, efficient code.
