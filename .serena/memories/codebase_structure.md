# Oxc Codebase Structure

## Top-Level Directories

### Core Implementation

- **`crates/`** - Core Rust functionality (start here when exploring)
- **`apps/`** - Application binaries (primarily oxlint)
- **`napi/`** - Node.js native bindings
- **`npm/`** - npm packages

### Development and Tools

- **`tasks/`** - Development tools and automation scripts
- **`editors/`** - Editor integrations (VS Code, etc.)
- **`wasm/`** - WebAssembly builds

### Testing and Conformance

- **`tasks/coverage/`** - External test suites (test262, babel, typescript)
- **`tasks/prettier_conformance/`** - Prettier formatting tests
- **`tasks/transform_conformance/`** - Transformation conformance tests

## Core Crates Structure

### Essential Crates

- **`oxc`** - Main umbrella crate exporting all public APIs
- **`oxc_allocator`** - Memory management and arena allocation
- **`oxc_ast`** - AST definitions and utilities
- **`oxc_parser`** - JavaScript/TypeScript parser
- **`oxc_semantic`** - Semantic analysis, symbols, scopes

### Tool Crates

- **`oxc_linter`** - Linting engine and rules implementation
- **`oxc_formatter`** - Code formatting (Prettier-like)
- **`oxc_transformer`** - Code transformation (Babel-like)
- **`oxc_minifier`** - Code minification
- **`oxc_codegen`** - Code generation from AST
- **`oxc_isolated_declarations`** - TypeScript declaration generation

### Utility Crates

- **`oxc_diagnostics`** - Error reporting with source locations
- **`oxc_traverse`** - AST traversal utilities and visitors
- **`oxc_syntax`** - Syntax utilities and definitions
- **`oxc_regular_expression`** - Regular expression parsing
- **`oxc_ecmascript`** - ECMAScript standard operations

## Generated Code Locations

**AVOID EDITING** these directories - they contain generated code:

- Any subdirectory named `generated/`
- Files with `generated` in the path
- Auto-generated derive implementations

## Key File Patterns

### Linting Rules

- **Location**: `crates/oxc_linter/src/rules/`
- **Organization**: By plugin (eslint/, typescript/, react/, etc.)
- **Pattern**: One rule per file, visitor pattern implementation
- **Tests**: Inline tests in same files using `Tester` helper

### Parser Components

- **Main**: `crates/oxc_parser/src/lib.rs`
- **Lexer**: `crates/oxc_parser/src/lexer/`
- **Tests**: Co-located with source + integration in `tests/`

### Examples

- **Location**: `crates/<crate_name>/examples/`
- **Purpose**: Quick testing and debugging of individual crates
- **Usage**: `cargo run -p <crate> --example <example> -- [args]`

## Configuration Files

### Rust Configuration

- **`Cargo.toml`** - Workspace configuration and dependencies
- **`.rustfmt.toml`** - Rust formatting configuration
- **`.clippy.toml`** - Clippy linting configuration

### Development Configuration

- **`justfile`** - Task runner commands and automation
- **`deny.toml`** - Dependency license and security checks
- **`dprint.json`** - Code formatting configuration

### Editor Configuration

- **`.vscode/`** - VS Code settings and extensions
- **`.editorconfig`** - Cross-editor configuration
