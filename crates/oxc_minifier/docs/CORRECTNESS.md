# Correctness Guarantees

## Our Commitment

The Oxc minifier aims to preserve JavaScript behavior under the
[documented assumptions](./ASSUMPTIONS.md). Tests provide regression coverage;
they do not prove correctness for every program.

## Validation Infrastructure

### Conformance Testing

`cargo coverage minifier` runs compression/code-generation idempotency checks on
selected test262 and Babel cases. The test262 runner skips parse-negative and
`noStrict` cases; the Babel runner skips expected failures and TypeScript sources.
The minifier has no TypeScript conformance suite or type-aware transformations.
Transform TypeScript, TSX, and JSX to JavaScript before minification.

Run conformance tests:

```bash
# Run all conformance tests
cargo coverage

# Run minifier conformance tests (test262 and Babel)
cargo coverage minifier

# Debug mode
cargo coverage minifier --debug

# Filter by test file path
cargo coverage minifier --filter "path/to/test.js"
```

### Size Tracking

Track compression performance:

```bash
# Update size benchmarks
just minsize

# Compare with esbuild
# See tasks/minsize/minsize.snap for results
```

### Runtime Testing

`cargo coverage runtime` runs eligible test262 cases through codegen, transformer,
and minifier pipelines and executes the outputs with Node.js (>= 22).
`tasks/e2e` also contains Node.js runtime tests for the transformer and minifier.
Idempotency alone does not establish behavioral equivalence.

### Semantic Preservation

Within the documented assumptions, optimizations are intended to preserve:

- Variable scoping and bindings
- Execution order and side effects
- Exception behavior
- Async/await/generator semantics
- All observable behavior

## Edge Cases Handled

### Temporal Dead Zones (TDZ)

Code must not violate a TDZ. Preserving the resulting `ReferenceError` is not
guaranteed; see [No TDZ Violation](./ASSUMPTIONS.md#no-tdz-violation).

```javascript
// Outside the minifier's assumptions: this code violates the TDZ.
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
Object.defineProperty(obj, "prop", {
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

Constant evaluation and side-effect analysis use `oxc_ecmascript` conversions,
subject to the documented assumptions about coercion purity and exceptions:

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
# From the repository root, build the bindings first
pnpm --filter "./napi/minify" --filter "./napi/transform" run build-test
pnpm --dir tasks/e2e test
```

### Idempotency Testing

The conformance runner checks whether a second compression produces the same
output. The example can also report whether a particular input is idempotent:

```bash
cargo run -p oxc_minifier --example minifier -- test.js --twice
```

## Semantic Bookkeeping Checks

Debug builds run over-prune and under-prune guards in `src/compression_pass.rs`
to catch inconsistencies between live AST references and semantic scoping.
These guards check bookkeeping, not arbitrary runtime equivalence.

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

Check the current [CI workflow](../../../.github/workflows/ci.yml) for
configured checks. Choose local verification using the repository's
[verification guidance](../../../.agents/verification.md).
