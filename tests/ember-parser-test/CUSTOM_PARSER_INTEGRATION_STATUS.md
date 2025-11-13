# Custom Parser Integration Status

## Overview

This document describes the current state of custom parser integration for `.gjs` and `.gts` files in `oxlint`, specifically focusing on the handling of unknown/custom AST nodes.

## Goal

Make `tests/ember-parser-test` work correctly so that `.gjs` and `.gts` files are properly linted by:
1. Built-in Rust rules (e.g., `no-unused-vars`, `no-debugger`, `no-console`)
2. Custom JavaScript plugin rules (future work)

## Current Problem: Unknown Node Handling

**The core issue is that we're not handling unknown/custom nodes generically enough.**

### The Challenge

Custom parsers (like `ember-eslint-parser`) produce AST nodes that are not part of the standard ESTree specification:
- `GlimmerTemplate` - Template blocks in class bodies
- `GlimmerPathExpression` - JavaScript property paths like `this.count`
- `GlimmerElementNode` - HTML elements
- `GlimmerMustacheStatement` - `{{expression}}` syntax
- And many more...

These custom nodes need to be:
1. **Stripped** for Rust rules (which only understand standard ESTree)
2. **Preserved** for JavaScript plugin rules (which can understand custom nodes)
3. **Analyzed** for semantic information (variable usage, references, etc.)

### Current Approach

We've implemented a **dual-path architecture**:

#### Path 1: Rust Rules (Stripped AST)
1. Custom parser produces full AST with custom nodes
2. `stripCustomNodes()` removes custom nodes and replaces them with standard ESTree equivalents
3. Stripped AST is converted to oxc AST
4. Rust linting rules run on the oxc AST

#### Path 2: JavaScript Plugin Rules (Full AST)
1. Full AST (with custom nodes) is stored separately
2. JavaScript plugins can access the full AST with custom nodes
3. (Not yet fully implemented)

### Issues with Current Unknown Node Handling

#### 1. **Heuristic-Based Path Conversion**

We use heuristics to convert custom path-like nodes to standard ESTree:

```typescript
// apps/oxlint/src-js/plugins/strip-nodes.ts
function convertPathLikeToMemberExpression(node: any, base: any): any {
  // We look for "path" or "original" properties
  // This is a heuristic, not a generic solution
  if (node.path) { /* ... */ }
  else if (node.original) { /* ... */ }
}
```

**Problem**: This assumes custom parsers use specific property names (`path`, `original`). If a parser uses different properties, expressions won't be extracted.

**Impact**: Rules like `no-unused-vars` won't detect variable usage in templates if the path structure doesn't match our heuristics.

#### 2. **Limited Expression Extraction**

We only extract expressions that:
- Are already standard ESTree nodes (embedded in custom nodes)
- Match our path-like node heuristics

**Problem**: Custom parsers might represent JavaScript expressions in ways we don't recognize:
- Different property names
- Nested structures we don't traverse
- Custom node types that wrap expressions differently

**Impact**: Some variable references in templates may be missed, leading to false "unused variable" warnings.

#### 3. **Class Body Context Assumptions**

We assume custom nodes in class bodies should be converted to synthetic methods:

```typescript
if (context === 'classBody' || (parent && parent.type === 'ClassBody' && key === 'body')) {
  const expressions = extractJSExpressionsFromCustomNode(node);
  if (expressions.length > 0) {
    return createSyntheticMethodForExpressions(expressions, node);
  }
  return null;
}
```

**Problem**:
- Not all custom nodes in class bodies are templates
- Some might be valid class elements in the custom syntax
- We're losing information by converting everything to a synthetic method

**Impact**:
- May create unnecessary synthetic methods
- May miss valid custom class elements that should be preserved differently

#### 4. **No Generic Node Type Registry**

We maintain a hardcoded list of "known" ESTree types:

```typescript
const STANDARD_ESTREE_TYPES = new Set([
  'Program', 'ExpressionStatement', 'Identifier', /* ... */
]);
```

**Problem**:
- Any node type not in this list is considered "custom"
- We don't have a way for custom parsers to declare which of their nodes contain JavaScript expressions
- We can't distinguish between "custom syntax node" vs "custom wrapper around standard expression"

**Impact**:
- May incorrectly strip nodes that should be preserved
- May miss expressions in nodes we don't recognize

## What We've Implemented

### File Processing
- ✅ Files with unrecognized extensions are processed if a custom parser is configured
- ✅ `ModuleRecord` is built manually for custom parsers
- ✅ Tokio runtime issues fixed for async JS parser calls

### Expression Extraction
- ✅ Generic traversal of custom nodes to find embedded standard ESTree expressions
- ✅ Heuristic-based conversion of path-like nodes to `MemberExpression`
- ✅ Deduplication of extracted expressions
- ✅ Synthetic method creation to preserve expressions for semantic analysis

### Error Handling
- ✅ Enhanced error messages for ESTree conversion failures
- ✅ Graceful handling of circular references in parser results
- ✅ Fallback handling for `ExpressionStatement` in class bodies

## What's Working

✅ `.gjs/.gts` files are now processed by `oxlint`
✅ Rust linting rules run on these files
✅ `unused_variable` in `demo-with-issues.gjs` is correctly detected as used (referenced in template)
✅ No "unused expression" warnings from synthetic methods
✅ Basic semantic analysis works (scopes, symbols, references)

## What's Not Working Well

❌ **Generic unknown node handling** - We rely on heuristics that may not work for all custom parsers
❌ **Expression extraction** - May miss expressions in custom nodes with non-standard structures
❌ **Custom node type detection** - Hardcoded list of known types, no extensibility
❌ **Parser-specific knowledge** - Some logic assumes specific parser structures

## Recommendations

### Short Term
1. **Document the heuristics** - Make it clear what patterns we look for
2. **Add logging** - Help debug when expressions aren't extracted
3. **Test with multiple parsers** - Verify genericity with Vue, Svelte, etc.

### Long Term
1. **Parser metadata API** - Allow parsers to declare:
   - Which node types contain JavaScript expressions
   - How to extract expressions from custom nodes
   - Which nodes should be stripped vs preserved

2. **Generic expression visitor** - Instead of heuristics, use a visitor pattern that:
   - Recursively searches for standard ESTree expression patterns
   - Works regardless of custom node structure
   - Can be extended by parser-specific plugins

3. **AST transformation registry** - Allow parsers to register:
   - Transformations from custom nodes to standard ESTree
   - Expression extraction strategies
   - Node preservation rules

## Test File

`demo-with-issues.gjs` contains intentional issues to verify the integration:
- `unused_variable` - Should NOT be flagged (used in template via `{{this.unused_variable}}`)
- `oldStyle` - SHOULD be flagged (unused, only in method)
- `debugger` - Should be flagged
- `console.log` - Should be flagged
- `var` - Should be flagged

## Key Files

- `apps/oxlint/src-js/plugins/strip-nodes.ts` - Generic expression extraction (needs improvement)
- `crates/oxc_linter/src/service/runtime.rs` - File processing and module record building
- `crates/oxc_linter/src/estree_converter.rs` - ESTree to oxc AST conversion
- `apps/oxlint/src/lint.rs` - Linter orchestration
- `apps/oxlint/src/js_plugins/external_linter.rs` - JS parser integration

## Next Steps

1. **Improve generic node handling** - Reduce reliance on heuristics
2. **Add parser metadata support** - Allow parsers to declare their node structures
3. **Test with other parsers** - Vue, Svelte, etc. to verify genericity
4. **Document expected patterns** - Help parser authors understand what we look for
5. **Consider AST transformation plugins** - Allow parser-specific extraction logic

