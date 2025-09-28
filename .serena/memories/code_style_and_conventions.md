# Oxc Code Style and Conventions

## Rust Code Style

### Formatting Configuration (.rustfmt.toml)

- **Style Edition**: 2024
- **Line Length**: Use wide screens (`use_small_heuristics = "Max"`)
- **Field Initialization**: Use shorthand when possible (`use_field_init_shorthand = true`)
- **Module Ordering**: Reorder modules alphabetically (`reorder_modules = true`)

### Memory Management Conventions

- **Always use `oxc_allocator`** for memory management
- **Avoid unnecessary allocations** - performance is critical
- **Use arena allocation** via `bumpalo` for AST nodes
- **Be mindful of heap allocations** in performance-critical paths

### Error Handling

- **Use `oxc_diagnostics`** for errors with source locations
- **Include span information** for user-facing errors
- **Provide clear, actionable error messages**

### Naming Conventions

- **Descriptive names**: Avoid generic type names like `T` unless truly generic
- **Follow Rust naming conventions**: snake_case for functions/variables, PascalCase for types
- **AST nodes**: Follow ECMAScript specification naming

## Linting Rules Architecture

### Visitor Pattern

- **All rules use visitor pattern** for AST traversal
- **Implement specific visit methods** for relevant AST nodes
- **Maintain rule state** in struct fields when needed

### Rule Structure Example

```rust
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

#[derive(Debug, Default, Clone)]
pub struct RuleName;

declare_oxc_lint!(
    RuleName,
    eslint,  // plugin category
    correctness,  // rule category
    "Rule description"
);

impl Rule for RuleName {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Implementation using visitor pattern
    }
}
```

### Testing Conventions

- **Inline tests** in rule files using `Tester` helper
- **Separate pass/fail test cases**
- **Use snapshot testing** with `test_and_snapshot()`

```rust
#[test]
fn test() {
    Tester::new(RuleName::NAME, RuleName::PLUGIN, pass, fail)
        .test_and_snapshot();
}
```

## AST and Parser Conventions

### AST Design Principles

- **Performance-first**: Optimized for fast traversal and memory efficiency
- **Ownership model**: Use references and arena allocation
- **Span tracking**: Every node has source location information
- **Type safety**: Leverage Rust's type system for correctness

### Parser Implementation

- **Incremental parsing**: Support for partial parsing where possible
- **Error recovery**: Continue parsing after syntax errors when safe
- **Conformance**: 100% compatibility with JavaScript/TypeScript standards

## Testing Conventions

### Test Organization

- **Co-located tests**: Unit tests alongside source code
- **Integration tests**: In `tests/` directories
- **Conformance tests**: External test suites via git submodules
- **Snapshot testing**: Use `insta` crate for output validation

### Test Categories

- **Unit Tests**: Individual function/module testing
- **Integration Tests**: Component interaction testing
- **Conformance Tests**: Standards compliance (Test262, Babel, TypeScript)
- **Performance Tests**: Allocation tracking and benchmarks

### Snapshot Testing

- **Track failures, not successes**: Snapshots show failing tests
- **Update with `cargo insta review`** after intended changes
- **Located in `tasks/coverage/snapshots/`** and conformance directories

## Documentation Standards

### Code Documentation

- **Public APIs**: Must have comprehensive doc comments
- **Examples**: Include usage examples in doc comments
- **Safety**: Document unsafe code thoroughly
- **Architecture**: Maintain high-level architecture documentation

### Comment Style

- **Explain why, not what**: Focus on reasoning and context
- **Enclose names in backticks**: Variables, types, functions in comments
- **Example**: `// Iterate over \`nodes\` to process each \`AstNode\` separately`

## Performance Guidelines

### Critical Performance Areas

- **Parser performance**: Bottleneck for all downstream tools
- **Memory allocations**: Use allocator, avoid Vec/HashMap when possible
- **Traversal efficiency**: Optimize visitor implementations
- **Parallel processing**: Design for multi-core systems

### Allocation Guidelines

- **Prefer stack allocation** when possible
- **Use arena allocators** for temporary data
- **Track allocations** via `just allocs` for parser changes
- **Profile memory usage** in performance-critical paths

## Contribution Guidelines

### Before Submitting

- **Run `just ready`** to ensure all checks pass
- **Add tests** for new functionality
- **Update documentation** for public API changes
- **Follow existing patterns** in the codebase

### Code Quality Standards

- **No commented-out code** in final submissions
- **No TODO comments** for basic cleanup
- **Consistent naming** within files and modules
- **Edge case consideration** for all new features
- **Follow established patterns** rather than inventing new approaches

### Git Practices

- **Atomic commits**: One logical change per commit
- **Descriptive messages**: Explain the why, not just the what
- **Test before committing**: All tests must pass
- **Format before committing**: Use pre-commit hooks
