# Oxc Data Structures

Common data structures and utilities used across oxc crates.

## Overview

This crate provides specialized data structures and utilities that are used throughout the oxc toolchain. These structures are optimized for the specific needs of compiler and tooling workloads.

## Key Features

- **Code buffer**: Efficient string building with segment tracking
- **Inline strings**: Memory-efficient string storage for short strings
- **Pointer extensions**: Utilities for safe pointer manipulation
- **Slice iterators**: Enhanced iteration capabilities for slices
- **Rope data structure**: Efficient text manipulation for large documents

## Available Components

### Code Buffer
```rust
use oxc_data_structures::code_buffer::CodeBuffer;

let mut buffer = CodeBuffer::new();
buffer.push_str("function ");
buffer.push_str("hello() {}");
let code = buffer.into_string();
```

### Inline String
```rust
use oxc_data_structures::inline_string::InlineString;

// Efficiently stores short strings inline, avoiding heap allocation
let short_string = InlineString::from("hello");
```

### Pointer Extensions
```rust
use oxc_data_structures::pointer_ext::PointerExt;

// Utilities for safe pointer operations in low-level code
```

### Slice Iterator Extensions
```rust
use oxc_data_structures::slice_iter_ext::SliceIterExt;

let items = vec![1, 2, 3, 4, 5];
// Additional iteration methods for slices
```

## Architecture

These data structures are designed with specific compiler requirements in mind:

- **Performance**: Optimized for common patterns in parsing and code generation
- **Memory efficiency**: Minimize allocations and memory overhead
- **Safety**: Provide safe abstractions over potentially unsafe operations
- **Ergonomics**: Easy to use APIs that integrate well with other oxc components

The structures complement Rust's standard library with domain-specific optimizations for JavaScript/TypeScript tooling.