# Oxc Span

Source position utilities and span management for precise error reporting.

## Overview

This crate provides essential utilities for managing source positions and spans in JavaScript and TypeScript code. It enables precise error reporting, source mapping, and location-aware transformations across all oxc tools.

## Key Features

- **Precise positions**: Accurate byte-level source positions
- **Span utilities**: Source ranges with start and end positions
- **Source type detection**: Automatic language and module detection
- **Atom interning**: Efficient string storage for identifiers
- **Content comparison**: Semantic equality for AST nodes

## Usage

### Working with Spans

```rust
use oxc_span::{Span, SPAN};

// Create a span from start and end positions
let span = Span::new(10, 25);
println!("Span from {} to {}", span.start, span.end);

// Empty/default span for generated nodes
let empty_span = SPAN;

// Check if span contains a position
if span.contains_inclusive(20) {
    println!("Position 20 is within the span");
}
```

### Source Type Detection

```rust
use oxc_span::SourceType;

// Automatic detection from file path
let js_type = SourceType::from_path("file.js").unwrap();
let ts_type = SourceType::from_path("file.ts").unwrap();
let jsx_type = SourceType::from_path("component.jsx").unwrap();
let tsx_type = SourceType::from_path("component.tsx").unwrap();

// Manual construction
let source_type = SourceType::typescript()
    .with_module(true)
    .with_jsx(true);

// Check properties
if source_type.is_typescript() {
    println!("TypeScript file");
}
```

### Atom Management

```rust
use oxc_span::Atom;

// Create interned strings for efficient storage
let name1 = Atom::from("identifier");
let name2 = Atom::from("identifier");

// Atoms with same content are identical (pointer equality)
assert!(std::ptr::eq(name1.as_str(), name2.as_str()));
```

### GetSpan Trait

```rust
use oxc_span::GetSpan;

// Most AST nodes implement GetSpan
fn print_location<T: GetSpan>(node: &T) {
    let span = node.span();
    println!("Node at {}..{}", span.start, span.end);
}
```

## Architecture

### Span System
- **Byte positions**: All positions are UTF-8 byte offsets
- **Inclusive ranges**: Spans include start position, exclude end position
- **Efficiency**: Uses u32 for positions, supporting files up to 4GB
- **Precision**: Enables precise error highlighting and source maps

### Source Type System
The source type encodes important metadata:
- **Language**: JavaScript vs TypeScript
- **Module system**: ESM vs Script
- **JSX support**: JSX vs plain syntax
- **Variant**: Standard vs definition files (.d.ts)

### Atom Interning
Atoms provide memory-efficient string storage:
- **Deduplication**: Identical strings share memory
- **Fast comparison**: Pointer equality for identical content
- **Compact representation**: Reduced memory usage for identifiers

### Integration Points
- **Parser**: Creates spans during tokenization and parsing
- **Semantic**: Tracks spans for all symbols and references  
- **Linter**: Uses spans for precise error reporting
- **Codegen**: Maintains spans for source map generation
- **Transformer**: Preserves spans during AST manipulation

This crate provides the foundation for precise source location tracking throughout the oxc toolchain.