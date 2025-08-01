# Oxc Parser

High-performance JavaScript and TypeScript parser with comprehensive language support.

## Overview

This crate provides a fast, spec-compliant parser for JavaScript and TypeScript that produces a complete Abstract Syntax Tree (AST). It supports all modern language features including JSX, TSX, and the latest ECMAScript proposals.

## Key Features

- **Complete language support**: JavaScript, TypeScript, JSX, and TSX
- **Latest ECMAScript**: All stable ECMAScript features plus stage 3+ proposals
- **High performance**: Significantly faster than traditional parsers
- **Error recovery**: Continues parsing after errors to provide complete AST
- **Comprehensive AST**: Detailed node information with accurate source positions

## Usage

### Basic Parsing

```rust
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

let allocator = Allocator::default();
let source_text = r#"
import React from 'react';

export const Component: React.FC = () => {
    return <div>Hello World</div>;
};
"#;

let source_type = SourceType::from_path("component.tsx").unwrap();
let ParserReturn { program, errors, trivias, panicked } = 
    Parser::new(&allocator, source_text, source_type).parse();

if panicked {
    eprintln!("Parser panicked - invalid syntax");
} else if errors.is_empty() {
    println!("Parsed successfully!");
    // Use the AST...
} else {
    for error in errors {
        eprintln!("Parse error: {}", error);
    }
}
```

### Source Type Detection

```rust
use oxc_span::SourceType;

// Automatic detection from file extension
let ts_type = SourceType::from_path("file.ts").unwrap();
let jsx_type = SourceType::from_path("component.jsx").unwrap(); 
let tsx_type = SourceType::from_path("component.tsx").unwrap();

// Manual specification
let source_type = SourceType::typescript()
    .with_module(true)
    .with_jsx(true);
```

## Architecture

### Parser Design

- **Recursive descent**: Traditional recursive descent parser architecture
- **Error recovery**: Sophisticated error recovery for IDE-friendly parsing
- **Memory efficient**: Uses arena allocation for optimal performance
- **Streaming**: Processes source text in a single pass

### Language Support

- **JavaScript**: Full ECMAScript 2024+ support
- **TypeScript**: Complete TypeScript syntax including decorators
- **JSX/TSX**: React JSX and TypeScript JSX syntax
- **Proposals**: Stage 3 decorators and other advancing proposals

### AST Structure

The parser produces an AST that closely follows ECMAScript specifications:

- **Accurate positions**: Every node has precise source location information
- **Complete information**: Preserves all syntactic details including trivia
- **Type-safe**: Leverages Rust's type system for correctness
- **Visitor-friendly**: Designed for easy traversal and transformation

The parser is designed as the foundation for all other oxc tools, providing the high-quality AST needed for analysis, transformation, and code generation.
