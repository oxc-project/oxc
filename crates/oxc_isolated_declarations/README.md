# Oxc Isolated Declarations

TypeScript isolated declarations transformer for generating `.d.ts` files.

## Overview

This crate implements TypeScript's isolated declarations feature, which generates TypeScript declaration files (`.d.ts`) from source code without requiring full type checking. This enables faster builds and better incremental compilation.

## Key Features

- **Fast declaration generation**: Generate `.d.ts` files without full type checking
- **TypeScript 5.5+ compatibility**: Implements the latest isolated declarations specification
- **Incremental builds**: Enables faster TypeScript compilation workflows
- **Comprehensive support**: Handles classes, functions, interfaces, and complex types

## Architecture

### Isolated Declarations Concept

Isolated declarations allow generating TypeScript declaration files without full type inference by requiring that:

- Types can be determined from the current file without following imports
- Exported declarations have explicit annotations where their types cannot be inferred locally
- Supported local inference includes simple literal initializers and some function return expressions

### Implementation Details

- **AST transformation**: Convert implementation AST to declaration AST
- **Type extraction**: Extract and preserve type information
- **Export analysis**: Determine what needs to be included in declarations
- **Error reporting**: Provide helpful diagnostics for missing type annotations

### Benefits

- **Faster builds**: No full type checking required
- **Incremental compilation**: Each file can be processed independently
- **Parallel processing**: Multiple files can be processed simultaneously
- **Simplified tooling**: Easier to integrate into build systems

This implementation follows the TypeScript compiler's approach while leveraging oxc's performance-oriented architecture.
