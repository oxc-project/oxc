# Oxc Suggested Commands

## Core Development Commands

```bash
# Complete check before submitting
just ready          # Run all CI checks

# Individual steps
just fmt            # Format code
just check          # Run cargo check
just test           # Run all tests
just lint           # Lint with warnings as errors
just doc            # Build documentation

# Fix issues
just fix            # Fix all auto-fixable issues
```

## Testing Commands

```bash
# All tests
cargo test --all-features

# Crate-specific tests
cargo test -p oxc_isolated_declarations  # For isolated declarations
cargo test -p oxc_parser
cargo test -p oxc_linter
cargo test -p oxc_transformer

# Conformance tests
just conformance         # Run Test262, Babel, TypeScript suites
cargo coverage           # Alias for conformance

# Snapshot testing
cargo insta review       # Review and approve snapshot changes
```

## Isolated Declarations Specific

```bash
# Run tests for isolated_declarations
cargo test -p oxc_isolated_declarations

# Snapshot tests
# Input: tests/fixtures/*.{ts,tsx}
# Output: tests/snapshots/*.snap
cargo insta review       # Review snapshot changes
```

## Quick Examples

```bash
cargo run -p oxc_parser --example parser -- test.js
cargo run -p oxc_linter --example linter -- src/
cargo run -p oxc --example compiler --features="full" -- test.js
```

## Watch Mode

```bash
just watch "command"              # Watch and re-run any command
just watch-check                  # Watch and run cargo check + clippy
```

## AST/Codegen

```bash
just ast           # Regenerate AST boilerplate after oxc_ast changes
just minsize       # Update minifier size snapshots
just allocs        # Update allocation snapshots
```
