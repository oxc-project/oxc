# Oxc Parser

High-performance JavaScript and TypeScript parser with comprehensive language support.

## Overview

This crate provides a fast, spec-compliant parser for JavaScript and TypeScript that produces a complete Abstract Syntax Tree (AST). It supports JavaScript, TypeScript, JSX, TSX, and selected ECMAScript proposals.

## Key Features

- **Complete language support**: JavaScript, TypeScript, JSX, and TSX
- **Latest ECMAScript**: Stable ECMAScript syntax plus selected Stage 3 proposals
- **High performance**: Significantly faster than traditional parsers
- **Error recovery**: Recovers from some syntax errors; fatal errors return an empty program
- **Comprehensive AST**: Detailed node information with accurate source positions

## Architecture

### Parser Design

- **Recursive descent**: Traditional recursive descent parser architecture
- **Error recovery**: Sophisticated error recovery for IDE-friendly parsing
- **Memory efficient**: Uses arena allocation for optimal performance
- **Source input**: Parses a complete UTF-8 source string, with lookahead where needed

### Language Support

- **JavaScript**: Full ECMAScript 2024+ support
- **TypeScript**: Complete TypeScript syntax including decorators
- **JSX/TSX**: React JSX and TypeScript JSX syntax
- **Proposals**: Stage 3 decorators and other advancing proposals

### AST Structure

The parser produces an AST that closely follows ECMAScript specifications:

- **Accurate positions**: Every node has precise source location information
- **Comments and source text**: Records comments and source spans; whitespace is available through the original source text
- **Type-safe**: Leverages Rust's type system for correctness
- **Visitor-friendly**: Designed for easy traversal and transformation

The parser is designed as the foundation for all other oxc tools, providing the high-quality AST needed for analysis, transformation, and code generation.
