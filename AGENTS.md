# AGENTS.md - AI Assistant Guide for Oxc

Oxc is a high-performance JavaScript/TypeScript toolchain written in Rust containing:
- Parser (JS/TS with AST), Linter (oxlint), Formatter, Transformer, Minifier

## Repository Structure

Rust workspace with key directories:
- `crates/` - Core functionality (start here when exploring)
- `apps/` - Application binaries (oxlint)
- `napi/` - Node.js bindings
- `npm/` - npm packages
- `tasks/` - Development tools/automation
- `editors/` - Editor integrations

Avoid editing `generated` subdirectories.

### Core Crates
- `oxc_parser` - JS/TS parser
- `oxc_ast` - AST definitions/utilities  
- `oxc_linter` - Linting engine/rules
- `oxc_semantic` - Semantic analysis/symbols
- `oxc_transformer` - Code transformation
- `oxc_codegen` - Code generation
- `oxc_minifier` - Code minification
- `oxc_formatter` - Code formatting
- `oxc_diagnostics` - Error reporting
- `oxc` - Main crate

## Development Commands

Prerequisites: Rust (MSRV: 1.87.0), Node.js, pnpm, just

**Setup Notes:**
- `just init` has already been run, all tools (`cargo-insta`, `typos-cli`, `cargo-shear`, `dprint`) are already installed, do not run `just init`.
- Rust and `cargo` components `clippy`, `rust-docs` and `rustfmt` has already been installed, do not install them.
- Always run `just ready` as the last step after code has been committed to the repository.

Key commands (tools already installed):
```bash
just ready     # Run all checks (use after commits)
just fmt       # Format code (run after modifications)
just check     # Check code
just test      # Run tests  
just lint      # Run linting
just ast       # Update generated files (when oxc_ast changes)
just conformance # Conformance tests (when Rust code changes)

pnpm build-dev # Build Node.js bindings
pnpm test      # Node.js tests
```

## Examples

Most crates have examples in their `examples/` directories:
```bash
cargo run -p <crate_name> --example <example_name> [filename]
```

Key examples:
- `oxc_parser --example parser` - Basic parsing/AST
- `oxc_linter --example linter` - Linting demo  
- `oxc_semantic --example semantic` - Semantic analysis
- `oxc_transformer --example transformer` - Code transformation
- `oxc --example compiler --features="full"` - Full pipeline

## Code Navigation

### Key Locations
- AST: Start with `oxc_ast`, use `oxc_ast_visit` for traversal
- Linting rules: `crates/oxc_linter/src/rules/` (visitor pattern)
- Parser: `crates/oxc_parser/src/lib.rs`, lexer in `src/lexer/`
- Tests: Co-located with source, integration in `tests/`, uses `insta` for snapshots

### Conventions
- Use `oxc_allocator` for memory management
- Follow rustfmt config in `.rustfmt.toml`
- Use `oxc_diagnostics` for errors with source locations
- Performance-critical: avoid unnecessary allocations

## Common Tasks

### Adding Linting Rule
1. Create module in `crates/oxc_linter/src/rules/`
2. Implement using visitor pattern
3. Add tests in same module
4. Register in appropriate category

### Parser Changes
1. Research and test grammar changes thoroughly
2. Update AST definitions in `oxc_ast` if needed
3. Ensure existing tests pass
4. Add tests for new features

### Working with Transformations
1. Understand AST structure first
2. Use visitor pattern for traversal
3. Handle node ownership/allocator carefully
4. Test with various input patterns

## Notes
- Rapidly evolving project - APIs may change
- Performance is critical for all changes
- Maintain JS/TS standard compatibility
- Breaking changes need documentation and discussion

---
For human contributors see `CONTRIBUTING.md` and [oxc.rs](https://oxc.rs)
