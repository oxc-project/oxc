# Oxc Style and Conventions

## Language & Stack
- **Primary Language**: Rust
- **Build System**: Cargo (workspace)
- **Task Runner**: Just (justfile)
- **Node.js Tools**: pnpm for package management

## Rust Conventions

### Code Style
- Follow `.rustfmt.toml` configuration
- Use `cargo fmt` and `dprint fmt` for formatting
- No unnecessary allocations (performance critical)
- Use `oxc_allocator` for memory management

### Error Handling
- Use `oxc_diagnostics` for errors with source locations
- Provide meaningful error messages

### Testing
- Use `insta` for snapshot testing
- Co-locate tests with source code
- Integration tests in `tests/` directories

## Naming Conventions
- Crates prefixed with `oxc_`
- Standard Rust naming: snake_case for functions/variables, CamelCase for types
- Descriptive names for tests

## Documentation
- Document public APIs
- Use `cargo doc` to verify documentation builds

## Code Organization
- Core functionality in `crates/`
- Application binaries in `apps/`
- Node.js bindings in `napi/`
- Development tools in `tasks/`

## Generated Code
- Avoid editing `generated` subdirectories
- Run `just ast` after AST changes
