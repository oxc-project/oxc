# Oxc Syntax

Common JavaScript and TypeScript syntax definitions and utilities.

## Overview

This crate provides shared syntax definitions, constants, and utilities used across the oxc toolchain. It contains fundamental types and enums that represent JavaScript and TypeScript language constructs.

## Key Features

- **Syntax constants**: Keywords, operators, and language character definitions
- **Identifier validation**: ECMAScript identifier checking and utilities
- **Operator precedence**: Precedence tables for expression parsing
- **Module records**: Import/export relationship tracking

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
- **Character definitions**: Line terminators and JSX XML entities

#### Semantic Types

- **IDs**: Type-safe identifiers for nodes, symbols, scopes, references
- **Flags**: Classify node, scope, symbol, and reference behavior

#### Module System

- **Module records**: Describe ESM imports, exports, and requested modules
- **Source positions**: Track the spans of import/export entries

Module path resolution is provided separately by `oxc_resolver`.

### Integration

This crate serves as the foundation for:

- **Parser**: Uses syntax definitions during tokenization
- **Semantic**: Leverages ID types and flags for analysis
- **Linter**: References operator precedence and keyword tables
- **Transformer**: Uses module records for import/export handling

The syntax crate ensures consistency and type safety across all oxc components.
