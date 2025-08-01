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

## Architecture

These data structures are designed with specific compiler requirements in mind:

- **Performance**: Optimized for common patterns in parsing and code generation
- **Memory efficiency**: Minimize allocations and memory overhead
- **Safety**: Provide safe abstractions over potentially unsafe operations
- **Ergonomics**: Easy to use APIs that integrate well with other oxc components

The structures complement Rust's standard library with domain-specific optimizations for JavaScript/TypeScript tooling.
