# Oxc Semantic

Comprehensive semantic analysis for JavaScript and TypeScript programs.

## Overview

This crate performs semantic analysis on JavaScript and TypeScript ASTs, building symbol tables, scope trees, and control flow graphs. It provides the foundation for advanced static analysis, linting, and transformation tools.

## Key Features

- **Symbol resolution**: Build complete symbol tables with binding information
- **Scope analysis**: Construct scope trees following ECMAScript scoping rules
- **Reference tracking**: Track all variable references and their relationships
- **Control flow**: Optional control flow graph construction
- **JSDoc parsing**: Extract and parse JSDoc comments
- **Module analysis**: Analyze import/export relationships

## Architecture

### Semantic Analysis Pipeline

1. **AST Traversal**: Visit all nodes to collect declarations
2. **Scope Building**: Construct scope tree following language rules
3. **Symbol Resolution**: Create symbol table with binding information
4. **Reference Analysis**: Track all identifier references
5. **Control Flow**: Optionally build control flow graphs
6. **JSDoc Processing**: Parse and attach documentation

### Key Data Structures

#### Symbol Table

- **Symbols**: All declared identifiers (variables, functions, classes, etc.)
- **References**: All uses of identifiers
- **Bindings**: Association between names and symbols in scopes

#### Scope Tree

- **Scope hierarchy**: Nested scopes following language semantics
- **Binding resolution**: How identifiers resolve to declarations
- **Closure analysis**: Capture and usage patterns

#### Control Flow Graph

- **Basic blocks**: Sequences of statements with single entry/exit
- **Flow edges**: Conditional and unconditional control flow
- **Exception handling**: Try/catch/finally flow modeling

### Applications

- **Linting**: Detect unused variables, undefined references, etc.
- **Transformation**: Safe variable renaming and scope analysis
- **Analysis**: Dead code detection, dependency analysis
- **IDE features**: Go-to-definition, find references, refactoring

The semantic analyzer provides the deep program understanding needed for sophisticated JavaScript and TypeScript tooling.
