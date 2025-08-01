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

### Keywords and Identifiers

```rust
use oxc_syntax::{keyword::is_reserved_keyword, identifier::is_identifier_name};

// Check if a string is a reserved keyword
if is_reserved_keyword("function") {
    println!("This is a reserved keyword");
}

// Validate identifier names
if is_identifier_name("validName") {
    println!("Valid JavaScript identifier");
}
```

### Operators and Precedence

```rust
use oxc_syntax::{operator::BinaryOperator, precedence::Precedence};

let op = BinaryOperator::Addition;
let precedence = op.precedence();
println!("Addition has precedence: {:?}", precedence);
```

### Node and Symbol Management

```rust
use oxc_syntax::{
    node::{NodeId, NodeFlags},
    symbol::{SymbolId, SymbolFlags},
    scope::{ScopeId, ScopeFlags},
    reference::{ReferenceId, ReferenceFlags}
};

// These types provide type-safe identifiers for semantic analysis
let node_id: NodeId = NodeId::new(42);
let symbol_id: SymbolId = SymbolId::new(10);
```

### Module Record

```rust
use oxc_syntax::module_record::{ModuleRecord, ExportEntry};

let mut module_record = ModuleRecord::new();
// Track imports and exports for module analysis
```

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
