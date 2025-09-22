# Correctness Guarantees

## Our Commitment

The Oxc minifier maintains correctness through comprehensive testing and validation.

## Validation Infrastructure

### Conformance Testing

- **test262**: Full ECMAScript specification compliance
- **Babel**: Modern JavaScript patterns and transforms
- **TypeScript**: Type-aware transformations
- **Real-world**: Top npm packages and frameworks

Run conformance tests:

```bash
# Run all conformance tests
cargo coverage

# Run specific suite
cargo coverage js    # test262
cargo coverage babel # Babel tests
cargo coverage ts    # TypeScript tests

# Debug mode
cargo coverage -- --debug

# Filter specific tests
cargo coverage -- --filter "test-name"
```

### Size Tracking

Track compression performance:

```bash
# Update size benchmarks
just minsize

# Compare with esbuild
# See tasks/minsize/minsize.snap for results
```

### Differential Testing

- Input vs output behavioral equivalence
- Cross-validation with other minifiers
- Fuzzing with randomly generated valid JavaScript

### Semantic Preservation

Every optimization preserves:

- Variable scoping and bindings
- Execution order and side effects
- Exception behavior
- Async/await/generator semantics
- All observable behavior

## Edge Cases Handled

### Temporal Dead Zones (TDZ)

```javascript
// Correctly preserves TDZ semantics
{
  console.log(x); // ReferenceError
  let x = 1;
}
```

### Sparse Arrays

```javascript
// Preserves sparse array behavior
const arr = [1, , 3];
arr.length; // 3
1 in arr; // false
```

### BigInt Operations

```javascript
// Maintains BigInt semantics
1n + 2n; // 3n
1n + 2; // TypeError
```

### Property Descriptors

```javascript
// Respects property descriptors
Object.defineProperty(obj, 'prop', {
  writable: false,
  value: 1,
});
```

### Iterator/Generator Protocols

```javascript
// Preserves iteration semantics
function* gen() {
  yield 1;
  yield 2;
}
```

### Type Coercion

All JavaScript coercion rules are preserved using `oxc_ecmascript` conversions:

- String coercion (`ToJsString`)
- Number coercion (`ToNumber`)
- Boolean coercion (`ToBoolean`)
- Symbol handling
- Object to primitive conversion

## Testing Strategy

### Unit Tests

Located in `crates/oxc_minifier/tests/`:

```bash
cargo test -p oxc_minifier
```

### Integration Tests

Real-world code testing:

```bash
cd tasks/e2e
pnpm test
```

### Idempotency Testing

Minifying twice produces same result:

```bash
cargo run -p oxc_minifier --example minifier test.js --twice
```

## Correctness Tools

### Semantic Validator

Validates that transformations preserve semantics.

### AST Differ

Shows exact changes made to the AST.

### Behavior Comparison

Compares runtime behavior of original vs minified code.

### Side Effect Tracker

Ensures side effects occur in correct order.

## Known Limitations

The minifier makes certain assumptions (see [ASSUMPTIONS.md](./ASSUMPTIONS.md)) that are true for standard JavaScript but may not hold for unusual code patterns.

## Bug Reporting

If you find correctness issues:

1. **Minimize the test case**: Find the smallest code that reproduces the issue
2. **Verify assumptions**: Check if code violates documented assumptions
3. **Test with other minifiers**: See if issue is common
4. **File an issue**: Include:
   - Original code
   - Minified output
   - Expected behavior
   - Actual behavior

## Continuous Validation

Our CI runs:

- Full conformance test suite
- Size benchmarks
- Real-world package tests
- Fuzzing tests

Every PR must pass all correctness tests.
