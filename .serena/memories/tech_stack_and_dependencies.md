# Oxc Tech Stack and Dependencies

## Core Technologies

### Rust

- **Edition**: 2024
- **MSRV**: 1.88.0 (Minimum Supported Rust Version, N-2 policy)
- **Workspace**: Cargo workspace with resolver "3"
- **Lints**: Comprehensive clippy and rust lints configuration

### Node.js Ecosystem

- **Package Manager**: pnpm (required, not npm)
- **Node.js**: Required for NAPI packages and TypeScript integration
- **Package Manager Version**: pnpm@10.17.0

### Build Tools

- **just**: Task runner (justfile for commands)
- **cargo**: Rust build system
- **NAPI-RS**: Node.js native bindings (@napi-rs/cli)

## Key Rust Dependencies

### Memory Management

- **oxc_allocator**: Custom allocator for performance
- **bumpalo**: Arena allocator for AST nodes

### Language Support

- **oxc_ast**: AST definitions and utilities
- **oxc_parser**: JavaScript/TypeScript parser
- **oxc_semantic**: Semantic analysis, symbols, scopes
- **oxc_traverse**: AST traversal utilities

### Development Tools

- **cargo-insta**: Snapshot testing
- **typos-cli**: Typo checking
- **cargo-shear**: Dependency analysis
- **dprint**: Code formatting
- **ast-grep**: Syntax-aware code search
- **clippy**: Rust linter
- **rustfmt**: Rust formatter

## External Test Suites (Git Submodules)

- **test262**: ECMAScript conformance suite (TC39)
- **babel**: Babel compiler test suite
- **typescript**: Microsoft TypeScript test suite
- **prettier**: Prettier formatting test suite
- **acorn-test262**: ESTree format validation tests

## Node.js Dependencies

- **TypeScript**: For type checking and compilation
- **Vitest**: Test runner for NAPI packages
- **oxlint**: Self-hosted linting (uses own linter)
- **emnapi**: WebAssembly bindings support

## Platform Support

- **Primary**: macOS (Darwin), Linux, Windows
- **WebAssembly**: Supported via emnapi
- **Architecture**: Optimized for multi-core systems
