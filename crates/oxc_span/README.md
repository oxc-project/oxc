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
