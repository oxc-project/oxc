# Oxc Traverse

Advanced AST traversal with parent context and efficient tree navigation.

## Overview

This crate provides sophisticated AST traversal capabilities that allow visitors to read up the tree from any node. Unlike traditional visitors that only provide downward traversal, oxc_traverse enables accessing parent nodes and sibling contexts during traversal.

## Key Features

- **Parent context access**: Read parent nodes during traversal
- **Efficient navigation**: Navigate up and down the AST tree
- **Memory safety**: Statically prevents aliasing violations
- **Traverse context**: Rich context information during traversal
- **Generated visitors**: Most traversal code is auto-generated for consistency

## Architecture

### Traversal Context

The `TraverseCtx` provides rich information during traversal:

- **Parent stack**: Complete chain of parent nodes
- **Scope context**: Current scope and scope hierarchy
- **Symbol information**: Access to semantic analysis results
- **AST utilities**: Helper methods for common operations

### Memory Safety Design

The traversal system prevents Rust aliasing violations through:

- **Controlled access**: Only safe references are provided to visitors
- **Stack-based parents**: Parent information without direct references
- **Immutable ancestors**: Read-only access to ancestor nodes
- **Mutable current**: Safe mutable access to current node

### Code Generation

Most traversal code is generated to ensure:

- **Complete coverage**: All AST nodes have traversal methods
- **Consistency**: Uniform traversal patterns across node types
- **Performance**: Optimized traversal with minimal overhead
- **Maintainability**: Automatic updates when AST changes

### Use Cases

#### Static Analysis

- **Linting**: Check code patterns with parent context
- **Dependency analysis**: Track imports/exports with scope awareness
- **Security analysis**: Detect dangerous patterns in context

#### Code Transformation

- **Transpilation**: Transform syntax with contextual awareness
- **Optimization**: Apply optimizations based on usage patterns
- **Refactoring**: Safe code modifications with full context

#### Code Generation

- **Template processing**: Generate code with contextual information
- **Macro expansion**: Expand macros with scope awareness
- **AST construction**: Build new AST nodes with proper context

The traverse system enables sophisticated transformations that would be difficult or impossible with traditional visitor patterns, while maintaining Rust's safety guarantees.
