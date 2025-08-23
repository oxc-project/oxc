# Oxc NAPI

Node.js Native API bindings for oxc tools.

## Overview

This crate provides Node.js bindings for oxc tools, enabling JavaScript/TypeScript applications to use oxc's high-performance parsing, linting, and transformation capabilities through native Node.js modules.

## Key Features

- **Native performance**: Direct access to oxc's Rust implementation
- **UTF-16 compatibility**: Automatic span conversion for JavaScript strings
- **Error handling**: Proper JavaScript error propagation
- **Comment preservation**: Maintains comments during processing
- **Type safety**: TypeScript definitions for all APIs

## Architecture

### UTF-8 to UTF-16 Conversion

JavaScript uses UTF-16 string encoding, while Rust uses UTF-8. This crate handles the conversion:

- **Span conversion**: Updates all source positions to UTF-16 offsets
- **Comment handling**: Preserves comment positions during conversion
- **Error mapping**: Ensures error positions are correct in JavaScript

### Node.js Integration

The bindings are designed for:

- **npm packages**: Used by `@oxc-project/` npm packages
- **Build tools**: Integration with Webpack, Vite, and other bundlers
- **Editor support**: Language server and editor extension features
- **CLI tools**: Command-line interfaces for Node.js environments

### Error Handling

Provides JavaScript-friendly error handling:

- **OxcError**: Rust errors converted to JavaScript exceptions
- **Diagnostic integration**: Rich error messages with source context
- **Stack traces**: Proper error propagation to JavaScript

### Performance Considerations

- **Zero-copy**: Minimizes data copying between Rust and JavaScript
- **Efficient conversion**: Optimized UTF-8 to UTF-16 conversion
- **Memory management**: Proper cleanup of Rust resources
- **Async support**: Non-blocking operations where appropriate

This crate enables the broader JavaScript ecosystem to benefit from oxc's performance while maintaining familiar JavaScript APIs.
