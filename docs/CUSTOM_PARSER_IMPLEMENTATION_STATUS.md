# Custom Parser Implementation Status

## Overview

This document tracks the implementation status of custom parser support in oxc. Based on investigation of the codebase after the ember-eslint-parser proof-of-concept.

## What's Already Implemented ✅

### 1. Configuration Infrastructure
**Location**: `crates/oxc_linter/src/config/oxlintrc.rs`

- ✅ `parser` field in Oxlintrc (lines 86-101)
- ✅ `parserOptions` field (lines 102-105)
- ✅ Parser deserialization (string or object format, lines 320-360)
- ✅ Parser serialization
- ✅ Path resolution relative to config file (lines 197-199)

**Status**: **COMPLETE** - Configuration loading is fully implemented

### 2. Parser Store
**Location**: `crates/oxc_linter/src/external_parser_store.rs`

- ✅ `ExternalParserStore` struct
- ✅ Parser registration tracking
- ✅ Parser name lookup
- ✅ Empty/length checks

**Status**: **COMPLETE** - Store implementation is done

### 3. Runtime Integration
**Location**: `crates/oxc_linter/src/service/runtime.rs` (lines 960-1089)

- ✅ Check for custom parser configuration
- ✅ Load parser via `ExternalLinter`
- ✅ Call `parse_with_custom_parser` callback
- ✅ Store parser services (line 1001-1006)
- ✅ Store visitor keys (line 1008-1013)
- ✅ Convert ESTree to oxc AST (line 1018-1029)
- ✅ Build semantic analysis (line 1060-1064)
- ✅ Continue with normal linting flow

**Status**: **90% COMPLETE** - Integration is mostly done, missing node stripping

### 4. ExternalLinter Interface
**Location**: `crates/oxc_linter/src/external_linter.rs`

- ✅ `ExternalLinterLoadParserCb` callback type
- ✅ `ExternalLinterParseWithCustomParserCb` callback type
- ✅ `ParserLoadResult` struct
- ✅ `ParseResult` struct (with services, visitor_keys, scope_manager)
- ✅ Buffer format for AST transfer (u32 length + JSON)

**Status**: **COMPLETE** - Interface is well-designed

### 5. Test Infrastructure
**Location**: `crates/oxc_linter/tests/custom_parser_e2e_test.rs`

- ✅ Mock external linter creation
- ✅ Simulated parser loading
- ✅ ESTree AST mocking
- ✅ Integration test flow
- ✅ Realistic parser resolution test

**Status**: **COMPLETE** - Comprehensive test infrastructure

### 6. JavaScript Integration (Partial)
**Location**: `apps/oxlint/src/js_plugins/external_linter.rs`

- ✅ `load_parser` function (lines ?)
- ✅ `parse_with_custom_parser` function (lines ?)
- ✅ Parser resolution
- ✅ AST buffer serialization

**Status**: **PARTIAL** - Basic infrastructure, missing node stripping

## What's Missing ⏳

### 1. Custom Node Stripping (CRITICAL)
**Where**: Between parse result and ESTree conversion in `runtime.rs`

**Current flow**:
```rust
let parse_result = parse_with_custom_parser(...)?;  // Line 991
// ... store services/keys ...
let program = convert_estree_to_oxc_program(...)?;  // Line 1018
```

**Needed flow**:
```rust
let parse_result = parse_with_custom_parser(...)?;  // Line 991
// ... store services/keys ...

// ⏳ NEW: Strip custom nodes before conversion
let stripped_buffer = strip_custom_nodes_from_buffer(
    &parse_result.buffer,
    parse_result.estree_offset
)?;

let program = convert_estree_to_oxc_program(
    &stripped_buffer,  // Use stripped buffer
    0,  // Offset is now 0 after stripping
    source_text,
    allocator,
)?;
```

**Implementation**:
- Option A: Call JavaScript stripper from Rust
- Option B: Implement stripper in Rust
- **Recommendation**: Option A (reuse tested JS code from ember-parser-test)

**Priority**: **HIGH** - Blocks Rust rules from working with custom parsers

### 2. Node Stripper JS-Rust Bridge
**Location**: `apps/oxlint/src/js_plugins/external_linter.rs` (new function)

**Needed**:
```rust
/// Strip custom (non-ESTree) nodes from an AST buffer
pub fn strip_custom_nodes_from_buffer(
    buffer: &[u8],
    offset: usize,
) -> Result<Vec<u8>, String> {
    // 1. Extract JSON from buffer
    // 2. Call JavaScript stripper
    // 3. Re-serialize to buffer format
    // 4. Return new buffer
}
```

**Files to create/modify**:
- `apps/oxlint/src-js/plugins/strip-nodes.ts` - Port stripper from test
- `apps/oxlint/src/js_plugins/external_linter.rs` - Add Rust bridge function

**Priority**: **HIGH** - Required for node stripping

### 3. Full AST Pass-Through for JS Plugins
**Location**: `crates/oxc_linter/src/service/runtime.rs`

**Current**: Only converted (stripped) AST is available
**Needed**: Pass full AST to JS plugins for framework-aware rules

**Implementation**:
```rust
// Store full AST for JS plugin access
if self.linter.external_linter.is_some() {
    self.full_ast_for_js_plugins
        .lock()
        .expect("full_ast mutex poisoned")
        .insert(path.to_path_buf(), parse_result.buffer.clone());
}
```

**Priority**: **MEDIUM** - Required for JS plugin path, but Rust path works without it

### 4. Parser Loading from npm Packages
**Location**: `apps/oxlint/src-js/plugins/parser.ts` (new file)

**Current**: Parser paths are resolved but not actually loaded
**Needed**:
- Dynamic import of parser packages
- Caching loaded parsers
- Error handling for missing parsers

**Example**:
```typescript
export async function loadParser(
  parserPath: string,
  packageName?: string
): Promise<CustomParser> {
  // Try to resolve and import
  const parser = await import(parserPath);

  // Validate interface
  if (!parser.parseForESLint && !parser.parse) {
    throw new Error("Invalid parser");
  }

  return parser;
}
```

**Priority**: **MEDIUM** - Test infrastructure mocks this, but real usage needs it

### 5. Documentation
**Needed**:
- User guide for configuring custom parsers
- API documentation for parser interface
- Migration guide from ESLint
- Examples for common parsers

**Priority**: **LOW** - Can be done after implementation

### 6. E2E Tests with Real Parsers
**Location**: `apps/oxlint/test/` (new test files)

**Needed**:
- Test with `espree`
- Test with `@typescript-eslint/parser`
- Test with `@babel/eslint-parser`
- Test with `ember-eslint-parser`
- Test with Vue/Svelte parsers

**Priority**: **MEDIUM** - Important for validation, but unit tests cover basics

## Implementation Priority

### Phase 1: Make Rust Rules Work (1-2 days)
1. ✅ Port `strip-custom-nodes.js` to `apps/oxlint/src-js/plugins/`
2. ✅ Add `strip_custom_nodes_from_buffer` Rust bridge
3. ✅ Integrate stripper into runtime flow
4. ✅ Test with ember-eslint-parser stripped ASTs

**Deliverable**: Rust built-in rules work with custom parsers

### Phase 2: Make JS Plugins Work (1-2 days)
1. ✅ Store full (unstripped) AST for JS plugins
2. ✅ Pass full AST to JS plugin interface
3. ✅ Ensure visitor keys are used
4. ✅ Test with eslint-plugin-ember rules

**Deliverable**: JS plugin rules see full AST with custom nodes

### Phase 3: Real Parser Loading (2-3 days)
1. ✅ Implement dynamic parser loading
2. ✅ Add parser caching
3. ✅ Handle loading errors
4. ✅ Test with npm-installed parsers

**Deliverable**: Can use parsers from node_modules

### Phase 4: Polish & Docs (2-3 days)
1. ✅ Write user documentation
2. ✅ Add E2E tests with real parsers
3. ✅ Performance optimization
4. ✅ Error message improvements

**Deliverable**: Production-ready feature

## Key Files to Modify

### High Priority
1. `crates/oxc_linter/src/service/runtime.rs` - Add node stripping (lines 1015-1018)
2. `apps/oxlint/src-js/plugins/strip-nodes.ts` - Port stripper from test (NEW FILE)
3. `apps/oxlint/src/js_plugins/external_linter.rs` - Add strip bridge function

### Medium Priority
4. `apps/oxlint/src-js/plugins/parser.ts` - Dynamic parser loading (NEW FILE)
5. `crates/oxc_linter/src/service/runtime.rs` - Store full AST for JS plugins

### Low Priority
6. Documentation files
7. E2E test files

## Success Criteria

### Minimum Viable Product (MVP)
- ✅ Configuration loads custom parser settings
- ✅ Parser is called for configured files
- ✅ Custom nodes are stripped
- ✅ ESTree conversion succeeds
- ✅ Rust rules execute on standard JS/TS
- ⏳ Test passes with ember-eslint-parser

### Full Feature
- ✅ JS plugins receive full AST with custom nodes
- ✅ Real parsers can be loaded from npm
- ✅ Multiple parsers can be used (via overrides)
- ✅ Error messages are helpful
- ✅ Documentation is complete
- ⏳ E2E tests with 3+ different parsers

## Estimated Timeline

- **Phase 1 (MVP)**: 1-2 days
- **Phase 2 (JS Plugins)**: 1-2 days
- **Phase 3 (Real Parsers)**: 2-3 days
- **Phase 4 (Polish)**: 2-3 days

**Total**: 6-10 days of focused development

## Notes

- The ember-eslint-parser test in `tests/ember-parser-test/` is an excellent reference
- The `strip-custom-nodes.js` implementation is already tested and working
- The dual-path approach (stripped for Rust, full for JS) is architecturally sound
- Most infrastructure is already in place - we're in the "last mile" of implementation

## Next Immediate Steps

1. Create `apps/oxlint/src-js/plugins/strip-nodes.ts` by porting from test
2. Add `strip_custom_nodes_from_buffer` bridge in `external_linter.rs`
3. Modify `runtime.rs` to call stripper before conversion
4. Run ember_parser_integration tests to validate
5. Create E2E test with full ember-eslint-parser flow
