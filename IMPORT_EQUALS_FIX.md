# Fix for Unused Import Equals Removal

## Problem
Currently, oxc does not properly remove chains of unused `import foo = bar.baz` declarations. 

Example:
```typescript
import x = foo.x  // Should be removed
import y = x.y    // Should be removed  
import z = y.z    // Should be removed

export let bar = 1
```

Current behavior: Only `z` is removed
Expected behavior: All three should be removed (matching TypeScript, esbuild, SWC)

## Root Cause
The transformer processes import equals declarations in document order (top-to-bottom) using the `enter_declaration` hook:

1. When processing `import x = foo.x`, we check if `x` has any value references
2. At this point, `import y = x.y` creates a Read reference to `x`
3. So we keep `x` and transform it to `var x = foo.x`
4. Then we process `import y = x.y`, which has a reference from `import z = y.z`, so we keep it
5. Finally we process `import z = y.z`, which has no references, so we remove it

The code DOES mark the reference as Type when removing an import (lines 147-149 in module.rs), but this happens AFTER we've already processed the earlier imports.

## Solution Approaches

### Approach 1: Process in Reverse Order (Attempted)
Try to use `exit_declaration` or `exit_statement` to process declarations bottom-up.

**Problem**: The traversal architecture doesn't call exit hooks for top-level TSImportEqualsDeclaration, or when it does (exit_statement), the references haven't been transformed yet so they're all Type references, causing even used imports to be removed.

### Approach 2: Fixed-Point Iteration (Recommended)
Process import equals declarations iteratively until no more can be removed:

1. Collect all TSImportEqualsDeclarations in the current scope
2. Mark those that have value references from non-import code  
3. Iteratively mark imports that are referenced by already-marked imports
4. Remove all unmarked imports
5. Transform remaining imports

This can be implemented in `exit_program` or a custom traversal pass.

### Approach 3: Multi-Pass Transformation
Run the transformer multiple times until the output stabilizes. This is simple but potentially inefficient.

## Implementation Notes
- The existing code in `transform_ts_import_equals` already has the logic to check references and mark them as Type
- The issue is purely about the order of processing
- Must handle both top-level and namespace-scoped import equals
- Must not break existing conformance tests

## Test Cases
Added test case in `tasks/transform_conformance/tests/oxc/test/fixtures/typescript-remove-unused-import-equals/`

## Related Code
- `crates/oxc_transformer/src/typescript/module.rs` - Main transformation logic
- Lines 100-126: Check if import should be removed and mark references as Type
- Lines 136-172: Transform import equals to variable declaration
