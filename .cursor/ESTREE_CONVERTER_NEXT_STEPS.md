# ESTree Converter - Next Steps & Roadmap

## Executive Summary

The ESTree to oxc AST converter (`crates/oxc_linter/src/estree_converter.rs`) has reached a mature state with core functionality implemented. This document outlines the remaining work, testing strategy, and future enhancements needed to make it production-ready for ESLint custom parser support.

## Current State Assessment

### ✅ Completed Features

1. **Core AST Node Conversion**
   - All major statement types (if, for, while, switch, try, etc.)
   - All major expression types (binary, unary, call, member, etc.)
   - TypeScript-specific nodes (TSAsExpression, TSSatisfiesExpression, etc.)
   - Import/Export declarations with attributes
   - Class declarations and expressions
   - Function declarations and expressions

2. **TypeScript Type System**
   - Basic types (TSNumberKeyword, TSStringKeyword, etc.)
   - Complex types (TSUnionType, TSIntersectionType, etc.)
   - Generic types (TSTypeReference, TSGenericType)
   - Tuple types with named members (TSNamedTupleMember)
   - JSDoc types (JSDocNullableType, JSDocNonNullableType, JSDocUnknownType)
   - Interface and enum declarations
   - Module declarations

3. **Advanced Features**
   - Assignment targets (MemberExpression, ArrayPattern, ObjectPattern)
   - Rest elements in patterns and assignments
   - Getter/setter properties
   - BigInt and RegExp literals
   - Import attributes/assertions (WithClause)
   - Directive conversion

4. **Code Quality**
   - Removed unused imports
   - Improved error messages
   - Helper functions for code reuse
   - All 818 tests passing

### ⚠️ Known Limitations

1. **Unsupported Node Types**
   - Some edge case ESTree node types may still return `UnsupportedNodeType` errors
   - Need comprehensive audit of all ESTree node types vs. implemented conversions

2. **Error Handling**
   - Error messages could be more descriptive
   - Missing context about which parser generated the ESTree AST
   - No recovery strategies for partial failures

3. **Testing Coverage**
   - Unit tests exist but may not cover all edge cases
   - No integration tests with real ESLint parsers
   - No performance benchmarks

## Priority 1: Comprehensive Testing & Validation

### 1.1 Integration Testing with Real Parsers

**Goal**: Verify the converter works correctly with actual ESLint parsers.

**Tasks**:

- [ ] Create integration test suite using `@typescript-eslint/parser`
- [ ] Create integration test suite using `espree`
- [ ] Test with various TypeScript versions (4.x, 5.x)
- [ ] Test with various ECMAScript versions (ES2015, ES2020, ES2022, etc.)
- [ ] Test edge cases from real-world codebases

**Test Cases to Cover**:

```typescript
// TypeScript-specific features
- Decorators
- Private fields
- Optional chaining
- Nullish coalescing
- Top-level await
- Import assertions

// Complex patterns
- Nested destructuring
- Complex generics
- Conditional types
- Template literal types
- Mapped types

// Edge cases
- Sparse arrays
- Null/undefined handling
- Circular references
- Very large ASTs
```

**Implementation**:

```rust
// Create: crates/oxc_linter/tests/estree_integration_test.rs
// Test structure:
// 1. Parse TypeScript/JavaScript with external parser
// 2. Convert ESTree to oxc AST
// 3. Verify structure matches expected oxc AST
// 4. Check for conversion errors
```

### 1.2 Conformance Testing

**Goal**: Ensure converter handles all ESTree node types correctly.

**Tasks**:

- [ ] Audit all `EstreeNodeType` variants
- [ ] Create test cases for each node type
- [ ] Verify all match arms in `convert_statement` and `convert_expression`
- [ ] Document any intentionally unsupported node types

**Node Types to Verify**:

- [ ] All statement types (27+ variants)
- [ ] All expression types (40+ variants)
- [ ] All pattern types (Identifier, ObjectPattern, ArrayPattern, RestElement)
- [ ] All TypeScript type nodes (50+ variants)
- [ ] All declaration types

### 1.3 Error Case Testing

**Goal**: Ensure graceful error handling for invalid/malformed ESTree ASTs.

**Tasks**:

- [ ] Test with missing required fields
- [ ] Test with wrong field types
- [ ] Test with null/undefined values
- [ ] Test with circular references
- [ ] Test with extremely large ASTs
- [ ] Verify error messages are helpful

## Priority 2: Code Quality & Maintainability

### 2.1 Documentation

**Goal**: Make the converter easy to understand and maintain.

**Tasks**:

- [ ] Add module-level documentation explaining the conversion process
- [ ] Document each major conversion function
- [ ] Add examples of ESTree → oxc AST conversions
- [ ] Document known limitations and workarounds
- [ ] Create architecture diagram showing conversion flow

**Documentation Structure**:

```rust
//! # ESTree to oxc AST Converter
//!
//! This module converts ESTree AST nodes (from ESLint-compatible parsers)
//! into oxc's native AST format.
//!
//! ## Conversion Process
//!
//! 1. ESTree JSON is parsed from the transfer buffer
//! 2. Each node is validated and converted recursively
//! 3. Context is maintained for identifier kind disambiguation
//! 4. oxc AST nodes are allocated using the arena allocator
//!
//! ## Key Concepts
//!
//! - **Identifier Kinds**: ESTree has one Identifier type, oxc has four:
//!   - `BindingIdentifier`: Variable declarations, function parameters
//!   - `IdentifierReference`: Variable references, function calls
//!   - `IdentifierName`: Property names, method names
//!   - `LabelIdentifier`: Statement labels, break/continue targets
//!
//! - **Context-Aware Conversion**: The `ConversionContext` tracks parent
//!   node type and field name to determine correct identifier kind.
```

### 2.2 Error Message Improvements

**Goal**: Provide actionable error messages for debugging.

**Tasks**:

- [ ] Include source location in all error messages
- [ ] Suggest fixes for common errors
- [ ] Include context about which parser generated the AST
- [ ] Add examples of correct ESTree structure

**Example Improvements**:

```rust
// Current:
ConversionError::UnsupportedNodeType {
    node_type: format!("{:?}", node_type),
    span: self.get_node_span(estree),
}

// Improved:
ConversionError::UnsupportedNodeType {
    node_type: format!("{:?}", node_type),
    span: self.get_node_span(estree),
    context: format!(
        "Node at {}:{} in field '{}' of parent '{}'",
        line, column, field, parent
    ),
    suggestion: "This node type may not be supported yet. Please file an issue.",
}
```

### 2.3 Code Organization

**Goal**: Improve maintainability through better organization.

**Tasks**:

- [ ] Group related conversion functions (e.g., all pattern conversions together)
- [ ] Extract common patterns into helper functions
- [ ] Create separate modules for:
  - Statement conversions
  - Expression conversions
  - TypeScript type conversions
  - Pattern conversions
  - Helper utilities

**Proposed Structure**:

```
estree_converter/
├── mod.rs                    # Main entry point
├── statements.rs             # Statement conversions
├── expressions.rs            # Expression conversions
├── types.rs                  # TypeScript type conversions
├── patterns.rs               # Pattern conversions
├── helpers.rs                # Helper functions
└── error.rs                 # Error types and messages
```

## Priority 3: Performance Optimization

### 3.1 Performance Profiling

**Goal**: Identify and optimize performance bottlenecks.

**Tasks**:

- [ ] Profile conversion of large codebases (1000+ files)
- [ ] Measure memory allocation patterns
- [ ] Identify hot paths in conversion
- [ ] Benchmark against baseline (if available)

**Metrics to Track**:

- Conversion time per AST node
- Memory allocations per conversion
- Peak memory usage
- Cache hit rates (if caching is added)

### 3.2 Optimization Opportunities

**Potential Optimizations**:

- [ ] Cache frequently accessed ESTree node properties
- [ ] Reuse allocated AST nodes where possible
- [ ] Batch allocations for arrays/vectors
- [ ] Optimize string interning (Atom creation)
- [ ] Reduce context cloning overhead

## Priority 4: Feature Completeness

### 4.1 Missing ESTree Node Types

**Goal**: Support all ESTree node types used by common parsers.

**Tasks**:

- [ ] Audit `UnsupportedNodeType` errors in real-world usage
- [ ] Implement missing node types as they're encountered
- [ ] Prioritize based on parser usage (TypeScript parser > espree > others)

**Potential Missing Types**:

- [ ] ChainExpression (optional chaining)
- [ ] ImportExpression (dynamic imports)
- [ ] PrivateIdentifier (class private fields)
- [ ] StaticBlock (class static blocks)
- [ ] Any other types discovered during integration testing

### 4.2 TypeScript 5.x Features

**Goal**: Support latest TypeScript features.

**Tasks**:

- [ ] Decorators (already partially supported, verify completeness)
- [ ] `const` type parameters
- [ ] `satisfies` operator (TSSatisfiesExpression - verify)
- [ ] Import attributes (WithClause - verify)
- [ ] Any new features in TS 5.x

### 4.3 ECMAScript 2023+ Features

**Goal**: Support latest JavaScript features.

**Tasks**:

- [ ] Verify support for all ES2023 features
- [ ] Add support for ES2024 features as they're standardized
- [ ] Test with experimental features (stage 3 proposals)

## Priority 5: Developer Experience

### 5.1 Debugging Tools

**Goal**: Make it easier to debug conversion issues.

**Tasks**:

- [ ] Add debug logging mode (trace conversion steps)
- [ ] Create AST comparison tool (ESTree vs oxc)
- [ ] Add visualization for conversion failures
- [ ] Improve error messages with AST snippets

### 5.2 Testing Utilities

**Goal**: Make it easier to write tests.

**Tasks**:

- [ ] Create test helper macros for common patterns
- [ ] Add snapshot testing for AST conversions
- [ ] Create fixtures for common test cases
- [ ] Add property-based testing (generate random valid ESTree ASTs)

**Example Test Helper**:

```rust
macro_rules! test_conversion {
    ($estree_json:expr, $expected_oxc:expr) => {
        // Parse ESTree JSON
        // Convert to oxc AST
        // Compare with expected
        // Provide helpful diff on failure
    };
}
```

## Priority 6: Long-Term Enhancements

### 6.1 Incremental Conversion

**Goal**: Support incremental AST updates (for language servers).

**Tasks**:

- [ ] Design API for incremental conversion
- [ ] Track which nodes have changed
- [ ] Only re-convert changed subtrees
- [ ] Maintain mapping between ESTree and oxc nodes

### 6.2 Bidirectional Conversion

**Goal**: Convert oxc AST back to ESTree (for compatibility).

**Tasks**:

- [ ] Design reverse conversion API
- [ ] Implement oxc → ESTree conversion
- [ ] Ensure round-trip compatibility
- [ ] Handle oxc-specific features gracefully

### 6.3 Streaming Conversion

**Goal**: Convert large ASTs without loading entire tree into memory.

**Tasks**:

- [ ] Design streaming API
- [ ] Implement incremental parsing
- [ ] Support partial AST conversion
- [ ] Handle errors in streaming context

## Implementation Timeline

### Phase 1: Testing & Validation (Weeks 1-2)

- Integration tests with real parsers
- Conformance testing
- Error case testing
- **Deliverable**: Comprehensive test suite with >90% coverage

### Phase 2: Code Quality (Weeks 3-4)

- Documentation improvements
- Error message enhancements
- Code organization refactoring
- **Deliverable**: Well-documented, maintainable codebase

### Phase 3: Performance (Weeks 5-6)

- Performance profiling
- Optimization implementation
- Benchmarking
- **Deliverable**: <10% performance overhead vs. native parsing

### Phase 4: Feature Completeness (Weeks 7-8)

- Missing node type implementation
- TypeScript 5.x feature support
- ECMAScript 2023+ feature support
- **Deliverable**: 100% support for common parser node types

### Phase 5: Developer Experience (Weeks 9-10)

- Debugging tools
- Testing utilities
- **Deliverable**: Easy-to-use development tools

## Success Criteria

### Must Have (MVP)

- ✅ All core ESTree node types supported
- ✅ All tests passing
- ✅ Basic error handling
- ⏳ Integration tests with real parsers
- ⏳ Documentation

### Should Have

- ⏳ Performance within 10% of native parsing
- ⏳ Comprehensive error messages
- ⏳ Code organization improvements
- ⏳ Debugging tools

### Nice to Have

- ⏳ Incremental conversion
- ⏳ Bidirectional conversion
- ⏳ Streaming conversion
- ⏳ Advanced debugging features

## Risk Assessment

### High Risk

- **Parser Compatibility**: Different parsers may generate slightly different ESTree ASTs
  - **Mitigation**: Extensive integration testing with multiple parsers
- **Performance**: Conversion overhead may be too high for large codebases
  - **Mitigation**: Early performance profiling and optimization

### Medium Risk

- **TypeScript Version Compatibility**: New TS versions may introduce new node types
  - **Mitigation**: Version-specific testing, extensible architecture
- **Maintenance Burden**: Converter needs to stay in sync with ESTree spec
  - **Mitigation**: Automated testing, clear documentation

### Low Risk

- **Edge Cases**: Rare ESTree patterns may not be handled
  - **Mitigation**: Comprehensive test coverage, graceful error handling

## Dependencies

### External

- ESLint parsers (`@typescript-eslint/parser`, `espree`)
- TypeScript compiler (for testing)
- Test frameworks

### Internal

- `oxc_ast`: AST definitions
- `oxc_allocator`: Memory management
- `oxc_estree`: ESTree deserialization utilities
- `oxc_span`: Source location tracking

## Open Questions

1. **Error Recovery**: Should the converter attempt to recover from errors, or fail fast?
2. **Validation**: Should we validate ESTree AST structure before conversion?
3. **Caching**: Should we cache conversion results for performance?
4. **Versioning**: How do we handle ESTree spec version differences?
5. **Performance vs. Correctness**: Trade-offs for optimization?

## References

- [ESTree Specification](https://github.com/estree/estree)
- [TypeScript AST Viewer](https://ts-ast-viewer.com/)
- [oxc AST Documentation](https://oxc.rs/docs/ast)
- [ESLint Custom Parser Guide](https://eslint.org/docs/latest/developer-guide/working-with-custom-parsers)

## Notes

- This document should be updated as work progresses
- Priorities may shift based on user feedback and real-world usage
- Some features may be deferred if not needed for MVP
- Performance optimizations should be data-driven (profile first, optimize second)
