# Oxc Minifier - AI Assistant Guide

## Mission

Build the best JavaScript minifier: smallest output, 100% correct, reasonably fast.

## Quick Reference

### Architecture

- Fixed-point iteration in `PeepholeOptimizations::run_in_loop`
- Arena allocation via `oxc_allocator`
- 17+ optimizations in `src/peephole/`

### Key Files

```
src/
├── lib.rs           # Entry point
├── compressor.rs    # Pipeline orchestration
├── ctx.rs           # Shared utilities
├── state.rs         # Optimization state
├── peephole/
│   ├── mod.rs       # Optimization dispatch
│   └── *.rs         # Individual optimizations
```

### Adding New Optimizations

1. **Research**: Study implementation in other minifiers (Terser, Closure, esbuild)
2. **Design**: Determine transformation rules and edge cases
3. **Implement**: Add to `src/peephole/` following existing patterns
4. **Hook up**: Add to traversal in `peephole/mod.rs`
5. **Test**: Add comprehensive tests
6. **Measure**: Check size impact with `just minsize`
7. **Validate**: Run conformance with `cargo coverage`

### Testing Commands

```bash
just test           # Unit tests
just minsize        # Size benchmarks (PRIMARY METRIC)
cargo coverage      # Conformance tests
just e2e           # End-to-end tests
just ready         # Run before committing
```

### Common Patterns

#### Optimization Structure

```rust
pub fn optimize_something(expr: &mut Expression, ctx: &mut Ctx) {
    // Check if optimization applies
    if !can_optimize(expr, ctx) {
        return;
    }

    // Apply transformation
    *expr = create_optimized_form(ctx);
    ctx.state.changed = true;
}
```

#### Traversal Hook

```rust
// In peephole/mod.rs
fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
    let ctx = &mut Ctx::new(ctx);
    match expr {
        Expression::BinaryExpression(_) => {
            Self::optimize_binary(expr, ctx);
        }
        // ... other cases
    }
}
```

### Size Optimization Tips

- Always prefer shorter syntax (`!0` over `true`)
- Combine related operations
- Remove all redundancy
- Fold constants aggressively
- Eliminate dead code completely
- Use comma operators for sequences
- Prefer operators over keywords

### Performance Tips

- Minimize AST traversals
- Use arena allocation (`ctx.ast`)
- Batch related transformations
- Early exit when possible
- Check side effects early

### Common Pitfalls to Avoid

1. **Side Effects**: Always check `may_have_side_effects()`
2. **Scope Issues**: Use `scoping()` for variable lookups
3. **TDZ**: Be careful with `let`/`const` transformations
4. **Evaluation Order**: Preserve execution order
5. **Edge Cases**: Test with `NaN`, `Infinity`, `-0`, etc.

### Debugging

```bash
# Debug conformance failures
cargo coverage -- --debug

# Test single file
cargo run -p oxc_minifier --example minifier test.js

# Test idempotency
cargo run -p oxc_minifier --example minifier test.js --twice
```

### Key Utilities in `ctx.rs`

- `eval_binary()` - Evaluate binary expressions
- `value_to_expr()` - Convert constant to AST node
- `is_expression_undefined()` - Check for undefined
- `expr_eq()` - Compare expressions
- `scoping()` - Access scope information
- `ast` - AST builder

### Development Flow

1. **Find optimization opportunity**
   - Check minsize benchmarks
   - Study other minifiers
   - Look for patterns in real code

2. **Implement safely**
   - Follow existing patterns
   - Handle all edge cases
   - Preserve semantics

3. **Test thoroughly**
   ```bash
   cargo test -p oxc_minifier
   cargo coverage
   just minsize
   ```

4. **Document clearly**
   - Add comments explaining the optimization
   - Document any assumptions
   - Add examples in tests

### AST Manipulation

```rust
// Use TakeIn for moving nodes
use oxc_allocator::TakeIn;
let moved = node.take_in(ctx.ast);

// Use arena for allocations
ctx.ast.expression_boolean_literal(span, true)

// Check node types
if let Expression::BinaryExpression(e) = expr {
    // work with binary expression
}
```

### Remember

- Size is the primary goal
- Correctness is non-negotiable
- Document assumptions clearly
- Test with real-world code
- Learn from other minifiers
