# Oxc Syntax

Common JavaScript and TypeScript syntax definitions and utilities.

## Overview

This crate provides shared syntax definitions, constants, and utilities used across the oxc toolchain. It contains fundamental types and enums that represent JavaScript and TypeScript language constructs.

## Key Features

- **Syntax constants**: Keywords, operators, and language tokens
- **AST utilities**: Common patterns and helper functions for AST manipulation
- **Identifier validation**: ECMAScript identifier checking and utilities
- **Operator precedence**: Precedence tables for expression parsing
- **Module records**: Import/export relationship tracking

## Key Components

## Architecture

### Design Principles

- **Shared definitions**: Avoid duplication across oxc crates
- **Type safety**: Use newtypes for different kinds of IDs
- **Performance**: Efficient representations for common operations
- **Standards compliance**: Follow ECMAScript specifications

### Core Components

#### Language Constants

- **Keywords**: All JavaScript/TypeScript reserved words
- **Operators**: Binary, unary, and assignment operators
- **Tokens**: Punctuation and special symbols

#### AST Utilities

- **Node flags**: Metadata about AST nodes (computed properties, etc.)
- **Traversal helpers**: Common patterns for walking AST trees
- **Type guards**: Runtime type checking for AST nodes

#### Semantic Types

- **IDs**: Type-safe identifiers for nodes, symbols, scopes, references
- **Flags**: Bitfield metadata for semantic entities
- **Relationships**: Parent-child and reference relationships

#### Module System

- **Import/Export tracking**: Comprehensive module dependency analysis
- **Resolution**: Module specifier resolution utilities
- **Metadata**: Module type and format information

### Integration

This crate serves as the foundation for:

- **Parser**: Uses syntax definitions during tokenization
- **Semantic**: Leverages ID types and flags for analysis
- **Linter**: References operator precedence and keyword tables
- **Transformer**: Uses module records for import/export handling

The syntax crate ensures consistency and type safety across all oxc components.
