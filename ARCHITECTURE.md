# Architecture

## System Overview

Oxc (The Oxidation Compiler) is a collection of high-performance JavaScript and TypeScript tools written in Rust. The system is designed as a modular, composable set of compiler components that can be used independently or together to build complete toolchains for JavaScript/TypeScript development.

### Core Mission

- **Performance**: Deliver faster performance than existing JavaScript tools
- **Correctness**: Maintain compatibility with JavaScript/TypeScript standards
- **Modularity**: Enable users to compose tools according to their specific needs
- **Developer Experience**: Provide excellent error messages and tooling integration

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          Applications                           │
├─────────────────────────────────────────────────────────────────┤
│  oxlint  │  Language Server  │  NAPI Bindings  │  Future Tools  │
├─────────────────────────────────────────────────────────────────┤
│                        Core Libraries                           │
├─────────────────────────────────────────────────────────────────┤
│ Parser │ Semantic │ Linter │ Transformer │ Minifier │ Codegen   │
├─────────────────────────────────────────────────────────────────┤
│                    Foundation Libraries                         │
├─────────────────────────────────────────────────────────────────┤
│    AST    │  Allocator  │  Diagnostics  │   Span   │  Syntax    │
└─────────────────────────────────────────────────────────────────┘
```

## Architecture Principles

### 1. Zero-Copy Architecture

The system is built around an arena allocator (`oxc_allocator`) that enables zero-copy operations throughout the compilation pipeline. All AST nodes are allocated in a single arena, eliminating the need for reference counting or garbage collection.

### 2. Visitor Pattern

AST traversal is implemented using the visitor pattern (`oxc_ast_visit`) with automatic visitor generation through procedural macros. This ensures type safety and performance while maintaining code clarity.

### 3. Shared Infrastructure

Common functionality like error reporting (`oxc_diagnostics`), source positions (`oxc_span`), and syntax definitions (`oxc_syntax`) are shared across all components to ensure consistency.

## Core Components

### Foundation Layer

#### oxc_allocator

- **Purpose**: Arena-based memory allocator for zero-copy operations
- **Key Features**:
  - Single allocation arena for entire compilation unit
  - Eliminates need for Rc/Arc in hot paths
  - Enables structural sharing of AST nodes
- **Dependencies**: None (foundational)

#### oxc_span

- **Purpose**: Source position tracking and text manipulation
- **Key Features**:
  - Byte-based indexing for UTF-8 correctness
  - Efficient span operations for source maps
  - Integration with diagnostic reporting
- **Dependencies**: None (foundational)

#### oxc_syntax

- **Purpose**: JavaScript/TypeScript language definitions
- **Key Features**:
  - Token definitions and keyword mappings
  - Language feature flags and compatibility
  - Shared syntax validation logic
- **Dependencies**: oxc_span

#### oxc_diagnostics

- **Purpose**: Error reporting and diagnostic infrastructure
- **Key Features**:
  - Rich error messages with source context
  - Multiple output formats (JSON, pretty-printed)
  - Integration with language server protocol
- **Dependencies**: oxc_span

#### oxc_ast

- **Purpose**: Abstract Syntax Tree definitions and utilities
- **Key Features**:
  - Complete JavaScript/TypeScript AST coverage
  - Generated visitor traits for type safety
  - Serialization support for caching
- **Dependencies**: oxc_allocator, oxc_span, oxc_syntax

##### AST Design Principles

The Oxc AST differs significantly from the [estree](https://github.com/estree/estree) AST specification by removing ambiguous nodes and introducing distinct types. While many existing JavaScript tools rely on estree as their AST specification, a notable drawback is its abundance of ambiguous nodes that often leads to confusion during development.

For example, instead of using a generic estree `Identifier`, the Oxc AST provides specific types such as:

- `BindingIdentifier` - for variable declarations and bindings
- `IdentifierReference` - for variable references
- `IdentifierName` - for property names and labels

This clear distinction greatly enhances the development experience by aligning more closely with the ECMAScript specification and providing better type safety.

### Core Processing Layer

#### oxc_parser

- **Purpose**: JavaScript/TypeScript parsing
- **Key Features**:
  - Hand-written recursive descent parser
  - Full ES2024+ and TypeScript support
  - Preservation of comments and trivia
- **Dependencies**: oxc_allocator, oxc_ast, oxc_diagnostics, oxc_span, oxc_syntax

#### oxc_semantic

- **Purpose**: Semantic analysis and symbol resolution
- **Key Features**:
  - Scope chain construction
  - Symbol table generation
  - Dead code detection
- **Dependencies**: oxc_ast, oxc_cfg, oxc_diagnostics, oxc_span, oxc_syntax

#### oxc_linter

- **Purpose**: ESLint-compatible linting engine
- **Key Features**:
  - 200+ built-in rules
  - Plugin architecture for custom rules
  - Automatic fixing for many rules
  - Configuration compatibility with ESLint
- **Dependencies**: oxc_ast, oxc_semantic, oxc_diagnostics, oxc_cfg

#### oxc_transformer

- **Purpose**: Code transformation and transpilation
- **Key Features**:
  - TypeScript to JavaScript transformation
  - Modern JavaScript feature transpilation
  - React JSX transformation
  - Babel plugin compatibility layer
- **Dependencies**: oxc_ast, oxc_semantic, oxc_allocator

#### oxc_minifier

- **Purpose**: Code size optimization
- **Key Features**:
  - Dead code elimination
  - Constant folding and propagation
  - Identifier mangling integration
  - Statement and expression optimization
- **Dependencies**: oxc_ast, oxc_semantic, oxc_mangler

#### oxc_codegen

- **Purpose**: AST to source code generation
- **Key Features**:
  - Configurable output formatting
  - Source map generation
  - Comment preservation options
  - Minified and pretty-printed output modes
- **Dependencies**: oxc_ast, oxc_span

### Application Layer

#### oxlint (apps/oxlint)

- **Purpose**: Command-line linter application
- **Key Features**:
  - File discovery and parallel processing
  - Configuration file support
  - Multiple output formats
  - Integration with CI/CD systems
- **Dependencies**: oxc_linter, oxc_parser, oxc_semantic

#### Language Server (oxc_language_server)

- **Purpose**: LSP implementation for editor integration
- **Key Features**:
  - Real-time diagnostics
  - Go-to-definition and references
  - Symbol search and completion
- **Dependencies**: All core components

#### NAPI Bindings (napi/\*)

- **Purpose**: Node.js integration layer
- **Key Features**:
  - Parser bindings for JavaScript tooling
  - Linter integration for build tools
  - Transform pipeline for bundlers
  - Async processing support
- **Dependencies**: Core components + Node.js FFI

## Data Flow

### Compilation Pipeline

1. **Input**: Source text + configuration
2. **Lexing/Parsing**: `oxc_parser` → AST + comments
3. **Semantic Analysis**: `oxc_semantic` → Symbol table + scope info
4. **Processing**: Tool-specific analysis (linting, transformation, etc.)
5. **Output**: Results (diagnostics, transformed code, etc.)

### Memory Management Flow

```
Source Text → Arena Allocator → AST Nodes → Visitors → Results
     ↓              ↓              ↓           ↓          ↓
   UTF-8          Arena         Borrowed    Zero-copy   Owned
  String         Memory         References  Processing  Output
```

## Quality Attributes

### Performance

- **Target**: 10-100x faster than comparable tools
- **Strategies**:
  - Arena allocation for memory efficiency
  - Zero-copy data structures
  - Parallel processing where possible
  - Minimal allocations in hot paths

#### Parser Performance Implementation

- AST is allocated in a memory arena ([bumpalo](https://crates.io/crates/bumpalo)) for fast AST memory allocation and deallocation
- Short strings are inlined by [CompactString](https://crates.io/crates/compact_str)
- No other heap allocations are done except the above two
- Scope binding, symbol resolution and some syntax errors are not done in the parser, they are delegated to the semantic analyzer

#### Linter Performance Implementation

- Oxc parser is used for optimal performance
- AST visit is a fast operation due to linear memory scan from the memory arena
- Files are linted in a multi-threaded environment, so scales with the total number of CPU cores
- Every single lint rule is tuned for performance

### Correctness

- **Target**: 100% compatibility with language standards
- **Strategies**:
  - Comprehensive test suites
  - Real-world codebase testing
  - Conformance testing against official specs
  - Conservative error handling

### Maintainability

- **Target**: Clear, reviewable, extensible codebase
- **Strategies**:
  - Strong type system usage
  - Procedural macro code generation
  - Clear separation of concerns
  - Comprehensive documentation

### Usability

- **Target**: Drop-in replacement for existing tools
- **Strategies**:
  - Configuration compatibility
  - Familiar CLI interfaces
  - Rich error messages
  - Editor integration

## Technical Constraints

### Language Choice

- **Rust**: Chosen for memory safety, performance, and zero-cost abstractions
- **MSRV**: N-2 policy for stability

### Memory Model

- **Arena Allocation**: Single arena per compilation unit
- **Lifetime Management**: Explicit lifetimes tied to arena
- **No Garbage Collection**: Manual memory management for predictable performance

### Threading Model

- **File-level Parallelism**: Multiple files processed in parallel
- **Single-threaded Pipeline**: Each file processed by single thread
- **Shared State**: Minimal shared state to avoid synchronization overhead

### Compatibility Requirements

- **JavaScript**: ES2024+ compatibility
- **TypeScript**: Latest TypeScript syntax support
- **Node.js**: LTS versions through NAPI bindings
- **Editors**: LSP compatibility for all major editors

## Design Decisions

### Arena Allocator Choice

**Decision**: Use custom arena allocator instead of Rc/Arc
**Rationale**:

- Eliminates reference counting overhead
- Enables zero-copy string operations
- Simplifies memory management
- Improves cache locality

**Trade-offs**:

- ✅ 10-50% performance improvement
- ✅ Simplified ownership model
- ❌ Requires lifetime management
- ❌ Less flexible memory patterns

### Hand-written Parser

**Decision**: Implement recursive descent parser instead of parser generator
**Rationale**:

- Easier debugging and maintenance
- More efficient generated code
- Faster compilation times

**Trade-offs**:

- ✅ Better performance and error messages
- ✅ More maintainable code
- ❌ More manual implementation work
- ❌ Higher risk of parser bugs

### Visitor Pattern

**Decision**: Use visitor pattern with procedural macros
**Rationale**:

- Type-safe AST traversal
- Automatic visitor generation
- Consistent patterns across tools
- Efficient dispatch

**Trade-offs**:

- ✅ Type safety and performance
- ✅ Reduced boilerplate code
- ❌ Compile-time complexity
- ❌ Learning curve for contributors

## Future Considerations

### Planned Extensions

- **Formatter**: Complete code formatting tool
- **Bundler**: Integration with bundling workflows
- **Type Checker**: Full TypeScript type checking
- **Plugin System**: User-defined transformations

### Scalability Concerns

- **Large Codebases**: Processing optimization improvements
- **Memory Usage**: Streaming processing for huge files
- **Parallel Processing**: Fine-grained parallelization

### Technology Evolution

- **Rust Evolution**: Leveraging new language features
- **JavaScript Standards**: Keeping pace with TC39 proposals
- **Editor Integration**: Advanced IDE features

## Development Infrastructure

### Test Infrastructure

Correctness and reliability are taken extremely seriously in Oxc. We spend significant effort on strengthening the test infrastructure to prevent problems from propagating to downstream tools:

- **Conformance Testing**: Test262, Babel, and TypeScript conformance suites
- **Fuzzing**: Extensive fuzzing to discover edge cases
- **Snapshot Testing**: Linter diagnostic snapshots for regression prevention
- **Ecosystem CI**: Testing against real-world codebases
- **Idempotency Testing**: Ensuring transformations are stable
- **Code Coverage**: Comprehensive coverage tracking
- **End-to-End Testing**: Testing against top 3000 npm packages

### Build and Development Tools

- **Rust**: MSRV 1.86.0+ with clippy and rustfmt integration
- **Just**: Command runner for development tasks (`just --list` for available commands)
- **Performance Monitoring**: Continuous benchmarking and performance regression detection
- **Cross-platform**: Support for Linux, macOS, and Windows
- **CI/CD**: Automated testing, building, and publishing pipelines

For detailed development guidelines, see [CONTRIBUTING.md](./CONTRIBUTING.md) and [AGENTS.md](./AGENTS.md).

---

This architecture document follows the [architecture.md](https://architecture.md/) format for documenting software architecture decisions and system design.
