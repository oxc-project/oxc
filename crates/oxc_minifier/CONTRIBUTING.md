# Contributing to OXC Minifier

Thank you for your interest in contributing to the OXC JavaScript minifier! This guide will help you understand the codebase and contribute effectively.

## Architecture Overview

The minifier is organized into several key components:

```
crates/oxc_minifier/
├── src/
│   ├── lib.rs              # Main API and coordination
│   ├── compressor.rs       # Compression orchestration  
│   ├── options.rs          # Configuration structures
│   ├── peephole/           # Individual optimizations
│   │   ├── mod.rs          # Optimization coordination
│   │   ├── fold_constants.rs
│   │   ├── minimize_*.rs   # Various minimization passes
│   │   └── remove_*.rs     # Dead code elimination passes
│   └── ctx.rs              # Traversal context utilities
├── examples/               # Usage examples
├── tests/                  # Integration tests
└── README.md              # User documentation
```

## Adding New Optimizations

### Step 1: Create the Optimization Module

Create a new file in `src/peephole/` for your optimization:

```rust
//! Brief description of what this optimization does.
//!
//! ## Examples
//!
//! ```javascript
//! // Before
//! if (true) { console.log('hello'); }
//!
//! // After  
//! console.log('hello');
//! ```

use oxc_ast::ast::*;
use oxc_traverse::{Traverse, traverse_mut_with_ctx};

use crate::ctx::{Ctx, TraverseCtx};

pub struct YourOptimization;

impl YourOptimization {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> Traverse<'a> for YourOptimization {
    // Implement the optimization logic
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Your optimization logic here
    }
}
```

### Step 2: Add Tests

Add comprehensive tests in your optimization module:

```rust
#[cfg(test)]  
mod tests {
    use crate::tester;

    #[test]
    fn test_your_optimization() {
        // Test basic functionality
        tester::test("input_code", "expected_output");
        
        // Test edge cases
        tester::test_same("code_that_should_not_change");
    }
}
```

### Step 3: Register the Optimization

Add your optimization to the main peephole optimization loop in `src/peephole/mod.rs`:

```rust
mod your_optimization;
use your_optimization::YourOptimization;

// In the Traverse implementation:
impl<'a> Traverse<'a> for PeepholeOptimizations {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // ... existing optimizations ...
        YourOptimization::new().build(program, ctx);
    }
}
```

## Testing Guidelines

### Unit Tests
- Each optimization should have comprehensive unit tests
- Test both positive cases (code that should be optimized) and negative cases (code that should remain unchanged)
- Include edge cases and error conditions

### Integration Tests  
- Add end-to-end tests in the `tests/` directory
- Test interactions between different optimizations
- Verify that optimizations don't break semantic correctness

### Running Tests
```bash
# Run all minifier tests
cargo test --package oxc_minifier

# Run tests for a specific optimization
cargo test --package oxc_minifier fold_constants

# Run with output to see test details
cargo test --package oxc_minifier -- --nocapture
```

## Safety Considerations

### Semantic Preservation
- **Never change program semantics** - optimizations must preserve the original behavior
- Be careful with side effects - expressions that look unused might have side effects
- Consider edge cases like `NaN`, `undefined`, and type coercion

### Common Pitfalls
- **Hoisting**: Be aware of variable hoisting rules
- **Temporal Dead Zone**: Let/const variables have different scoping rules than var
- **Side Effects**: Method calls, property access, and operators can have side effects
- **Type Coercion**: JavaScript's type coercion rules are complex

### Testing for Safety
```rust
// Always test that semantics are preserved
tester::test("console.log(1 + 2)", "console.log(3)");

// Test edge cases
tester::test_same("obj[sideEffectFunction()]"); // Don't optimize if there are side effects
tester::test_same("x++"); // Don't optimize expressions with side effects
```

## Performance Guidelines

### AST Traversal
- Minimize AST traversals - combine related optimizations when possible
- Use the existing traversal infrastructure in `oxc_traverse`
- Be mindful of memory allocations during optimization

### Fixed-Point Iteration
- Optimizations run in a fixed-point loop until no changes are made
- Mark when your optimization makes changes using the context
- Avoid infinite loops by being careful about what constitutes a "change"

## Code Style

### Documentation
- Add rustdoc comments to public functions and structs
- Include examples showing before/after code transformations
- Document any assumptions or limitations

### Naming Conventions
- Use descriptive names for optimization passes
- Follow the existing naming pattern: `minimize_*`, `remove_*`, `fold_*`
- Use `test_*` for test function names

### Error Handling
- Use `debug_assert!` for development-time checks
- Avoid panicking in release builds - prefer to skip optimization instead
- Log warnings for unusual but valid code patterns

## Getting Started

1. **Explore existing optimizations** in `src/peephole/` to understand patterns
2. **Run the examples** to see the minifier in action
3. **Pick a simple optimization** to implement (e.g., constant folding for a specific operator)
4. **Write tests first** to clarify the expected behavior
5. **Implement incrementally** and test frequently

## Resources

- [Terser documentation](https://github.com/terser/terser) - Many optimizations are inspired by Terser
- [ESTree spec](https://github.com/estree/estree) - Understanding AST node types
- [ECMAScript spec](https://tc39.es/ecma262/) - Understanding JavaScript semantics
- [OXC AST documentation](../oxc_ast/) - Understanding the OXC AST structure

## Questions?

Feel free to:
- Open an issue for questions about the architecture
- Start a discussion for design decisions
- Ask for code review on optimization implementations

We're here to help make the OXC minifier even better!