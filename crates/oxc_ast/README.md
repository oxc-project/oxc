# Oxc AST

Abstract Syntax Tree definitions for JavaScript, TypeScript, and JSX.

## Overview

This crate provides comprehensive AST (Abstract Syntax Tree) node definitions that support the full spectrum of JavaScript and TypeScript syntax, including JSX. The AST closely follows ECMAScript specifications while providing ergonomic APIs for manipulation.

## Key Features

- **Complete language support**: JavaScript, TypeScript, JSX, and TSX
- **ECMAScript compliant**: AST structure follows official specifications
- **Memory efficient**: Designed to work with `oxc_allocator` for fast allocation
- **Visitor patterns**: Integrates with `oxc_ast_visit` for traversal
- **Type-safe**: Leverages Rust's type system for correctness

## AST Design Principles

The AST design differs from estree in several important ways:

- **Explicit identifiers**: Uses specific types like `BindingIdentifier`, `IdentifierReference`, and `IdentifierName` instead of generic `Identifier`
- **Precise assignment targets**: `AssignmentExpression.left` uses `AssignmentTarget` instead of generic `Pattern`
- **Literal specialization**: Replaces generic `Literal` with `BooleanLiteral`, `NumericLiteral`, `StringLiteral`, etc.
- **Evaluation order**: Field order follows ECMAScript evaluation order for consistency

## Architecture

The AST is designed for:

- **Parse**: Efficient construction during parsing
- **Transform**: Easy manipulation during transpilation
- **Visit**: Systematic traversal for analysis
- **Codegen**: Clean conversion back to source code

All AST nodes are allocated in an arena (`oxc_allocator`) for optimal performance.
