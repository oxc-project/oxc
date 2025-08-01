# Oxc Codegen

High-performance code generation from AST back to JavaScript/TypeScript source code.

## Overview

This crate converts AST nodes back into source code strings, supporting JavaScript, TypeScript, and JSX. It's designed for speed and correctness, producing clean, readable output.

## Key Features

- **Fast code generation**: Optimized for performance with minimal allocations
- **Source map support**: Generate accurate source maps during output
- **Configurable formatting**: Control whitespace, semicolons, and other formatting options
- **Comment preservation**: Maintain comments during code generation
- **Binary expression optimization**: Intelligent parentheses insertion

## Usage

```rust
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_ast::ast::Program;

let allocator = Allocator::default();
let options = CodegenOptions::default();

// Generate code from AST
let mut codegen = Codegen::new(&allocator, &options);
codegen.build(&program);

let generated_code = codegen.into_source_text();
println!("{}", generated_code);
```

### With Source Maps

```rust
use oxc_codegen::CodegenOptions;
use oxc_sourcemap::SourcemapBuilder;

let mut sourcemap_builder = SourcemapBuilder::default();
let options = CodegenOptions {
    source_map_path: Some("output.js.map".into()),
    ..Default::default()
};

let mut codegen = Codegen::new(&allocator, &options);
codegen.build(&program);

let code = codegen.into_source_text();
let sourcemap = std::mem::take(codegen.sourcemap_builder()).into_sourcemap();
```

## Architecture

### Code Generation Pipeline
1. **AST Traversal**: Walk through AST nodes systematically
2. **Token Generation**: Convert nodes to appropriate tokens/strings  
3. **Formatting**: Apply whitespace, indentation, and style rules
4. **Source Mapping**: Track original source positions if enabled

### Design Principles
- **Correctness**: Generated code must be functionally equivalent to original
- **Performance**: Minimize string allocations and copying
- **Readability**: Produce clean, well-formatted output
- **Fidelity**: Preserve semantic meaning and behavior

The codegen is adapted from esbuild's approach, optimized for Rust and oxc's AST structure.