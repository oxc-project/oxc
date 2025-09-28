# Suggested Commands for Oxc Development

## Essential Development Commands

### Setup and Initialization

```bash
just init                    # Install all necessary tools (cargo-insta, typos-cli, etc.)
just submodules             # Clone/update test suites (test262, babel, typescript, prettier)
just install-hook           # Install git pre-commit hook for formatting
```

### Core Development Workflow

```bash
just fmt                    # Format all code (run after modifications)
just check                  # Run cargo check
just test                   # Run all unit/integration tests
just lint                   # Lint the whole project
just doc                    # Generate documentation
```

### Ready for Commit

```bash
just ready                  # Run complete CI checks (format, check, test, lint, doc, ast)
just r                      # Alias for 'ready'
```

### Crate-Specific Updates

```bash
just ast                    # Update generated AST files (after oxc_ast changes)
just minsize                # Update minifier size snapshots (after oxc_minifier changes)
just allocs                 # Update allocation snapshots (after oxc_parser changes)
```

## Testing Commands

### Unit and Integration Tests

```bash
just test                           # Run all Rust tests
cargo test -p <crate_name>          # Test specific crate
cargo test -p oxc_linter            # Example: test linter
cargo test -p oxc_parser            # Example: test parser
```

### Conformance Testing

```bash
just conformance                    # Run all conformance tests
just c                             # Alias for 'conformance'
cargo coverage                     # Alternative conformance command
cargo coverage -- parser          # Parser conformance (Test262, Babel, TypeScript)
cargo coverage -- transformer     # Transformer conformance
```

### Specialized Testing

```bash
cargo run -p oxc_transform_conformance     # Transformer Babel tests
cargo run -p oxc_prettier_conformance     # Formatter Prettier tests
just test-transform --filter <path>       # Filter transformer tests
```

### NAPI (Node.js) Testing

```bash
pnpm build-dev              # Build all NAPI packages (development)
pnpm build-test             # Build NAPI packages for testing
pnpm test                   # Test all Node.js bindings
cd napi/parser && pnpm test # Test specific NAPI package
```

## Examples and Manual Testing

### Running Examples

```bash
just example <tool> [args]                                    # Run tool examples
cargo run -p <crate> --example <example> -- [args]           # Direct cargo example

# Common examples:
cargo run -p oxc_parser --example parser -- test.js
cargo run -p oxc_linter --example linter -- src/
cargo run -p oxc_transformer --example transformer -- input.js
cargo run -p oxc --example compiler --features="full" -- test.js
```

### Watching for Changes

```bash
just watch "command"        # Watch files and re-run command
just watch-check           # Watch and run cargo check
just watch-example <args>  # Watch and run examples
just watch-oxlint <args>   # Watch and run oxlint
```

## Rule Development

### Creating New Linting Rules

```bash
just new-rule <name>              # Create ESLint rule
just new-ts-rule <name>           # Create TypeScript rule
just new-react-rule <name>        # Create React rule
just new-jest-rule <name>         # Create Jest rule
just new-vue-rule <name>          # Create Vue rule
just new-unicorn-rule <name>      # Create Unicorn rule
# ... and more plugin-specific generators
```

## Build Commands

### Rust Builds

```bash
cargo build                 # Debug build
cargo build --release      # Release build
just oxlint                # Build oxlint in release mode
```

### Node.js Package Builds

```bash
pnpm build                 # Production builds (slow, full optimization)
pnpm build-dev             # Development builds (faster, no release optimizations)
pnpm build-test            # Test builds (fastest)
```

## Utility Commands

### System Information

```bash
git status                 # Git repository status
typos                      # Check for typos in codebase
just autoinherit          # DRY up Cargo.toml manifests
```

### Benchmarking

```bash
just benchmark            # Run all benchmarks
just benchmark-one <args> # Run single component benchmark
just codecov              # Generate code coverage
```

## Platform-Specific Notes

### macOS (Darwin)

- All commands work as documented
- Use `ast-grep --lang rust -p '<pattern>'` for syntax-aware searches
- Avoid text-only tools like `grep`/`rg` for code structure searches

### Common File Operations

```bash
ls -la                    # List files (standard unix command)
find . -name "*.rs"       # Find Rust files (prefer ast-grep for code searches)
cd <directory>            # Change directory
```

**Note**: Always prefer semantic/syntax-aware tools like Serena MCP tools or ast-grep over text-based search tools for code analysis.
