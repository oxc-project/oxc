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
