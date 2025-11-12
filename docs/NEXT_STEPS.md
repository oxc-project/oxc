# Custom Parser Implementation - Next Steps

## Current Status

✅ **90% of infrastructure is already implemented!**

- Configuration: DONE
- Parser store: DONE
- Runtime integration: DONE (missing only node stripping)
- Test infrastructure: DONE
- ember-eslint-parser proof-of-concept: DONE

## What's Left (The Last 10%)

### Critical Missing Piece: Node Stripping

**Location**: `crates/oxc_linter/src/service/runtime.rs` line ~1015

**Current**:

```rust
let parse_result = parse_with_custom_parser(...)?;
let program = convert_estree_to_oxc_program(&parse_result.buffer, ...)?;
```

**Needed**:

```rust
let parse_result = parse_with_custom_parser(...)?;
let stripped_buffer = strip_custom_nodes(&parse_result.buffer)?;  // ← NEW
let program = convert_estree_to_oxc_program(&stripped_buffer, ...)?;
```

## Implementation Steps

### Step 1: Port Node Stripper (30 min)

Copy `tests/ember-parser-test/strip-custom-nodes.js` to:

- `apps/oxlint/src-js/plugins/strip-nodes.ts`

Convert to TypeScript, no logic changes needed.

### Step 2: Add Rust Bridge (1 hour)

In `apps/oxlint/src/js_plugins/external_linter.rs`, add:

```rust
/// Strip custom nodes from ESTree AST buffer
pub fn strip_custom_nodes_from_buffer(
    buffer: &[u8],
    offset: usize,
) -> Result<Vec<u8>, String>
```

Calls JavaScript stripper, returns new buffer.

### Step 3: Integrate into Runtime (30 min)

In `crates/oxc_linter/src/service/runtime.rs` around line 1015:

```rust
// Strip custom nodes before conversion (for Rust rules)
let (buffer_to_convert, offset_to_use) = if should_strip_custom_nodes {
    let stripped = strip_custom_nodes_from_buffer(
        &parse_result.buffer,
        parse_result.estree_offset,
    )?;
    (stripped, 0)
} else {
    (parse_result.buffer.clone(), parse_result.estree_offset)
};

let program = convert_estree_to_oxc_program(
    &buffer_to_convert,
    offset_to_use,
    source_text,
    allocator,
)?;
```

### Step 4: Test (30 min)

Run `cargo test --test ember_parser_integration`

Should pass with stripped ASTs.

## Total Time Estimate

**2-3 hours to complete the core implementation!**

## Files to Create/Modify

1. **NEW**: `apps/oxlint/src-js/plugins/strip-nodes.ts`
2. **MODIFY**: `apps/oxlint/src/js_plugins/external_linter.rs`
3. **MODIFY**: `crates/oxc_linter/src/service/runtime.rs`

That's it! Just 3 files to complete the feature.

## After Core Implementation

Optional enhancements (not blocking):

- Store full AST for JS plugins (easy, 1 hour)
- Real parser loading from npm (medium, 2-3 hours)
- Documentation (easy, 1-2 hours)
- E2E tests with real parsers (easy, 1 hour each)

## Reference Implementation

All working code is in `tests/ember-parser-test/`:

- `strip-custom-nodes.js` - Tested stripper
- `test-stripper.js` - Validation
- `ANALYSIS.md` - Technical details

The proof-of-concept is **complete and tested**. We just need to integrate it into the main pipeline!
