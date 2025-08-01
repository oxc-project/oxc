# AGENTS.md - AI Assistant Guide for Oxc

This document provides guidance for AI assistants working with the Oxc codebase.
It contains essential information about the project structure, development workflows,
and best practices to help AI systems navigate and contribute to this repository effectively.

## 🚀 Project Overview

Oxc (The Oxidation Compiler) is a collection of high-performance tools for JavaScript and TypeScript written in Rust. The project includes:

- **Parser**: JavaScript/TypeScript parsing with full AST support
- **Linter** (oxlint): Fast ESLint-compatible linter
- **Formatter**: Code formatting tools
- **Transformer**: Code transformation and compilation
- **Minifier**: Code minification for production

## 📁 Repository Structure

The repository follows a Rust workspace structure with multiple crates:

### Key Directories

- **`crates/`** - Core Rust crates containing the main functionality
- **`apps/`** - Application binaries (currently contains `oxlint`)
- **`napi/`** - Node.js bindings for JavaScript/TypeScript integration
- **`npm/`** - npm packages and Node.js-related code
- **`tasks/`** - Development tools, testing, and automation scripts
- **`editors/`** - Editor integrations (VS Code extension, etc.)

### Important Crates Overview

- **`oxc_parser`** - JavaScript/TypeScript parser
- **`oxc_ast`** - Abstract Syntax Tree definitions and utilities
- **`oxc_linter`** - Linting engine and rules
- **`oxc_semantic`** - Semantic analysis and symbol resolution
- **`oxc_transformer`** - Code transformation utilities
- **`oxc_codegen`** - Code generation from AST
- **`oxc_minifier`** - Code minification
- **`oxc_formatter`** - Code formatting
- **`oxc_diagnostics`** - Error reporting and diagnostics
- **`oxc`** - Main crate that ties everything together

## 🛠️ Development Workflow

### Prerequisites

The project uses several tools for development:

- **Rust**: Latest stable (MSRV: 1.86.0)
- **Node.js**: Version specified in `.node-version`
- **pnpm**: Package manager for Node.js dependencies
- **just**: Command runner (alternative to make)

### Common Commands

The project uses `just` as the primary command runner. Key commands include:

```bash
# Initialize development environment
just init

# Run all checks (the "ready" command)
just ready

# Individual commands
just fmt          # Format code
just check        # Check code without building
just test         # Run tests
just lint         # Run linting
just doc          # Generate documentation
```

### Building and Testing

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Build Node.js bindings
pnpm build

# Run Node.js tests
pnpm test
```

## 🧭 Code Navigation Tips

### Understanding the AST

- Start with `oxc_ast` to understand the AST structure
- Look at `oxc_ast_visit` for traversal patterns
- Check `oxc_semantic` for symbol resolution and scoping

### Linting Rules

- Linting rules are in `crates/oxc_linter/src/rules/`
- Each rule typically has its own module with tests
- Rule implementations follow a visitor pattern

### Parser Architecture

- Parser entry point: `crates/oxc_parser/src/lib.rs`
- Lexer: `crates/oxc_parser/src/lexer/`
- Parser modules organized by language constructs

### Testing Patterns

- Unit tests are co-located with source code
- Integration tests in `tests/` directories
- Snapshot testing using `insta` crate for AST and output verification
- Conformance testing against real-world codebases

## 📝 Code Style and Conventions

### Rust Conventions

- Follow standard Rust formatting (rustfmt configuration in `.rustfmt.toml`)
- Use Clippy lints as defined in `Cargo.toml`
- Prefer explicit over implicit where it improves readability
- Use the allocator pattern (`oxc_allocator`) for memory management

### Error Handling

- Use `oxc_diagnostics` for user-facing errors
- Follow the diagnostic pattern for error reporting
- Include source location information in diagnostics

### Performance Considerations

- This is a high-performance project - consider memory allocation patterns
- Use the arena allocator (`oxc_allocator`) for AST nodes
- Be mindful of string allocations and prefer string interning where appropriate
- Benchmark performance-critical changes

## 🤝 Contributing Guidelines for AI

### Making Changes

1. **Understand the scope**: Oxc is a complex project with many interdependent parts
2. **Start small**: Focus on specific, well-defined changes
3. **Test thoroughly**: Run the full test suite and add appropriate tests
4. **Follow patterns**: Study existing code patterns before implementing new features
5. **Consider performance**: This is a performance-critical project

### Common Tasks

#### Adding a New Linting Rule

1. Create a new module in `crates/oxc_linter/src/rules/`
2. Implement the rule following the visitor pattern
3. Add tests in the same module
4. Register the rule in the appropriate category
5. Update documentation if needed

#### Modifying the Parser

1. Changes to grammar should be well-researched and tested
2. Update AST definitions in `oxc_ast` if needed
3. Ensure all existing tests pass
4. Add new tests for new language features

#### Working with Transformations

1. Understand the AST structure thoroughly
2. Use the visitor pattern for traversing
3. Be careful with node ownership and the allocator
4. Test with various input patterns

### Testing Strategy

- Run `just ready` before submitting changes
- Add unit tests for new functionality
- Use snapshot tests for complex outputs
- Test against real-world codebases when possible
- Consider edge cases and error conditions

## 🔍 Debugging Tips

### Common Issues

- **Build failures**: Check Rust version compatibility and dependencies
- **Test failures**: May indicate AST changes or semantic differences
- **Performance regressions**: Use benchmarking tools in `tasks/benchmark/`

## 📚 Additional Resources

- **Website**: [oxc.rs](https://oxc.rs) - Official documentation

## ⚠️ Important Notes

- This is a rapidly evolving project - APIs may change
- Performance is a key consideration in all changes
- Compatibility with JavaScript/TypeScript standards is critical
- The project maintains high code quality standards
- Breaking changes should be well-documented and discussed

---

This document is intended to help AI assistants understand and work effectively with the Oxc codebase. For human contributors, please refer to `CONTRIBUTING.md` and the project website for complete guidelines.
