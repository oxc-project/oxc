# AGENTS.md - AI Assistant Guide for Oxc

Oxc is a high-performance JavaScript/TypeScript toolchain written in Rust containing:

- Parser (JS/TS with AST), Linter (oxlint), Formatter (oxfmt), Transformer, Minifier

## AI Usage Policy for Contributors

**IMPORTANT**: If you are an AI assistant helping a human contributor:

- **Disclose AI usage** - Contributors must disclose when AI tools were used to reduce maintainer fatigue
- **Full responsibility** - The human contributor is responsible for all AI-generated issues or PRs they submit
- **Quality standards** - Low-quality or unreviewed AI content will be closed immediately

All AI-generated code must be thoroughly reviewed, tested, and understood by the contributor before submission. Code should meet Oxc's performance and quality standards.

## Repository Structure

Rust workspace with key directories:

- `crates/` - Core functionality (start here when exploring)
- `apps/` - Application binaries (oxlint, oxfmt)
- `napi/` - Node.js bindings
- `npm/` - npm packages
- `tasks/` - Development tools/automation
- `editors/` - Editor integrations (e.g. oxc VS Code extension)

Avoid editing `generated` subdirectories.

### Core Crates

- `oxc_parser` - JS/TS parser
- `oxc_ast` - AST definitions/utilities
- `oxc_semantic` - Semantic analysis/symbols/scopes
- `oxc_linter` - Linting engine/rules
- `oxc_formatter` - Code formatting (Prettier-like)
- `oxc_transformer` - Code transformation (Babel-like)
- `oxc_minifier` - Code minification
- `oxc_codegen` - Code generation
- `oxc_isolated_declarations` - TypeScript declaration generation
- `oxc_diagnostics` - Error reporting
- `oxc_traverse` - AST traversal utilities
- `oxc_allocator` - Memory management
- `oxc_language_server` - LSP server for editor integration
- `oxc` - Main crate

## Development Commands

Prerequisites: Rust (MSRV: 1.91), Node.js, pnpm, just

**Setup Notes:**

- All tools already installed (`cargo-insta`, `typos-cli`, `cargo-shear`, `ast-grep`)
- Rust components already installed (`clippy`, `rust-docs`, `rustfmt`)
- Run `just ready` after commits for final checks
- You run in an environment where `ast-grep` is available; whenever a search requires syntax-aware or structural matching, default to `ast-grep --lang rust -p '<pattern>'` (or set `--lang` appropriately) and avoid falling back to text-only tools like `rg` or `grep` unless I explicitly request a plain-text search.

### Essential Commands

```bash
just fmt          # Format code (run after modifications)
just test         # Run unit/integration tests
just conformance  # Run conformance tests
just ready        # Run all checks (use after commits)

# Crate-specific updates
just ast          # Update generated files (oxc_ast changes)
just minsize      # Update size snapshots (oxc_minifier changes)
just allocs       # Update allocation snapshots (oxc_parser changes)

# Useful shortcuts
just watch "command"  # Watch files and re-run command
just example tool     # Run tool example (e.g., just example linter)
```

More commands can be found in `justfile`.

## Manual Testing & Examples

Run crate examples for quick testing and debugging:

```bash
cargo run -p <crate_name> --example <example_name> -- [args]

# Common examples:
cargo run -p oxc_parser --example parser -- test.js
cargo run -p oxc_linter --example linter -- src/
cargo run -p oxc_transformer --example transformer -- input.js
cargo run -p oxc --example compiler --features="full" -- test.js
```

Modify examples in `crates/<crate_name>/examples/` to test specific scenarios.

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

1. Use rule generator: `just new-rule <name>` (ESLint rules)
   - Or plugin-specific: `just new-ts-rule`, `just new-jest-rule`, etc.
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

## Testing

Oxc uses multiple testing approaches tailored to each crate:

- **Unit/Integration tests**: Standard Rust tests in `tests/` directories
- **Conformance tests**: Against external suites (Test262, Babel, TypeScript, Prettier)
- **Snapshot tests**: Track failures and expected outputs using `insta`

### Quick Test Commands

```bash
just test                                   # Run all Rust tests
just conformance                            # Run all conformance tests (alias: cargo coverage)
cargo test -p <crate_name>                  # Test specific crate

# Conformance for specific tools
cargo coverage -- parser                    # Parser conformance
cargo coverage -- transformer               # Transformer conformance
cargo run -p oxc_transform_conformance      # Transformer Babel tests
cargo run -p oxc_prettier_conformance       # Formatter Prettier tests

# NAPI packages
pnpm test                                    # Test all Node.js bindings
```

### Crate-Specific Testing

Each crate follows distinct testing patterns:

#### oxc_parser

- **Conformance only** via `tasks/coverage`
- **Command**: `cargo coverage -- parser`
- **Suites**: Test262, Babel, TypeScript
- **Special**: `just allocs` for allocation tracking

#### oxc_linter

- **Inline tests** in rule files (`src/rules/**/*.rs`)
- **Pattern**: Use `Tester` helper with pass/fail cases

```rust
#[test]
fn test() {
    Tester::new(RuleName::NAME, RuleName::PLUGIN, pass, fail)
        .test_and_snapshot();
}
```

#### oxc_formatter

- **Prettier conformance** only (no unit tests)
- **Command**: `cargo run -p oxc_prettier_conformance`
- **Debug**: Add `-- --filter <name>`
- Compares output with Prettier's snapshots

#### oxc_minifier

- **Unit tests** in `tests/` subdirectories:
  - `ecmascript/` - Operations
  - `peephole/` - Optimizations
  - `mangler/` - Name mangling
- **Size tracking**: `just minsize`

#### oxc_transformer

- **Multiple approaches**:
  - Unit tests: `tests/integrations/`
  - Conformance: `tasks/transform_conformance/`
  - Babel plugins: `tasks/transform_conformance/tests/babel-plugin-*/`
- **Commands**:

```bash
cargo test -p oxc_transformer                    # Unit tests
cargo run -p oxc_transform_conformance          # Conformance
just test-transform --filter <path>             # Filter tests
```

#### oxc_codegen

- **Integration tests** in `tests/integration/`
- Test files: `js.rs`, `ts.rs`, `sourcemap.rs`, `comments.rs`

#### oxc_isolated_declarations

- **Snapshot testing** with `insta`
- Input: `tests/fixtures/*.{ts,tsx}`
- Output: `tests/snapshots/*.snap`
- Update: `cargo insta review`

#### oxc_semantic

- **Multiple testing approaches**:
  - **Conformance tests** (`tests/conformance/`) - Contract-as-code tests for symbols and identifier references
  - **Integration tests** (`tests/integration/`) - Tests for scopes, symbols, modules, classes, CFG
  - **Snapshot tests** (`tests/main.rs`) - Verifies scoping data correctness (scope trees, bindings, symbols, references) using `insta` snapshots from `fixtures/`
  - **Coverage tests** - Via `tasks/coverage` using Test262, Babel, TypeScript suites
- **Command**: `cargo test -p oxc_semantic`
- **Update snapshots**: `cargo insta review`

#### Other Crates

- **oxc_traverse**: AST traversal - `cargo test -p oxc_traverse`
- **oxc_ecmascript**: ECMAScript operations - `cargo test -p oxc_ecmascript`
- **oxc_regular_expression**: Regex parsing - `cargo test -p oxc_regular_expression`
- **oxc_syntax**: Syntax utilities - `cargo test -p oxc_syntax`
- **oxc_language_server**: Editor integration - `cargo test -p oxc_language_server`

### Conformance Testing Foundation

**CRITICAL**: These external test suites are the CORE of Oxc's testing strategy, providing thousands of real-world test cases from mature JavaScript ecosystem projects. They ensure Oxc correctly handles the full complexity of JavaScript/TypeScript.

Git submodules managed via `just submodules`:

| Submodule            | Description                                                                                                                                        | Location                              | Used by Crates                                           |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- | -------------------------------------------------------- |
| `test262`            | **ECMAScript Conformance Suite**<br>Official JavaScript test suite from TC39, testing compliance with the ECMAScript specification                 | `tasks/coverage/test262`              | parser, semantic, codegen, transformer, minifier, estree |
| `babel`              | **Babel Test Suite**<br>Comprehensive transformation and parsing tests from the Babel compiler, covering modern JavaScript features and edge cases | `tasks/coverage/babel`                | parser, semantic, codegen, transformer, minifier         |
| `typescript`         | **TypeScript Test Suite**<br>Microsoft's TypeScript compiler tests, ensuring correct handling of TypeScript syntax and semantics                   | `tasks/coverage/typescript`           | parser, semantic, codegen, transformer, estree           |
| `prettier`           | **Prettier Formatting Tests**<br>Prettier's comprehensive formatting test suite, ensuring code formatting matches industry standards               | `tasks/prettier_conformance/prettier` | formatter (conformance)                                  |
| `estree-conformance` | **ESTree Conformance Tests**<br>Test262, TypeScript, and acorn-jsx suites adapted for ESTree format validation, ensuring correct AST structure     | `tasks/coverage/estree-conformance`   | estree                                                   |

**These suites provide:**

- **Thousands of battle-tested cases** from real-world usage
- **Edge case coverage** that would be impossible to write manually
- **Industry standard compliance** ensuring compatibility
- **Continuous validation** against evolving JavaScript standards

Run all conformance tests with `cargo coverage` or `just conformance`.

### Snapshot Testing

- Uses `insta` crate for snapshot testing
- Snapshots track **failing** tests, not passing ones
- Located in `tasks/coverage/snapshots/` and conformance directories
- Update with `cargo insta review` after changes
- Formats: `.snap` (counts), `.snap.md` (detailed failures)

### NAPI (Node.js Bindings) Testing

NAPI packages use **Vitest** for testing Node.js bindings:

```bash
pnpm build-dev    # Build all NAPI packages
pnpm test         # Run all NAPI tests
```

**Package-specific commands:**

- `oxc-parser`: `cd napi/parser && pnpm test` (also has `pnpm test-browser`)
- `oxc-minify`: `cd napi/minify && pnpm test`
- `oxc-transform`: `cd napi/transform && pnpm test`

Tests are TypeScript files in each package's `test/` directory.

### Where to Add Tests

| Crate                 | Location                                |
| --------------------- | --------------------------------------- |
| Parser                | `tasks/coverage/misc/pass/` or `fail/`  |
| Linter                | Inline in rule files                    |
| Formatter             | Prettier conformance suite              |
| Minifier              | `tests/` subdirectories                 |
| Transformer           | `tests/integrations/` or Babel fixtures |
| Codegen               | `tests/integration/`                    |
| Isolated Declarations | `tests/fixtures/*.ts`                   |
| Semantic              | `tests/` directory                      |
| NAPI packages         | `test/` directory (Vitest)              |
| Language Server       | Inline and `/fixtures`                  |

## Notes

- Rapidly evolving project - APIs may change
- Performance is critical for all changes
- Maintain JS/TS standard compatibility
- Breaking changes need documentation and discussion

---

For human contributors see `CONTRIBUTING.md` and [oxc.rs](https://oxc.rs)
