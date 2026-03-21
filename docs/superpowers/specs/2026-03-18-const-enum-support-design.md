# Const Enum Support Design

**Date:** 2026-03-18
**Issues:** [#11844](https://github.com/oxc-project/oxc/issues/11844), [#6073](https://github.com/oxc-project/oxc/issues/6073)

## Overview

Add const enum inlining support across oxc_transformer (single-file) and rolldown (cross-module). Enum member values are evaluated once in `oxc_semantic` and consumed by both downstream tools via the `Scoping` object.

## Approach

**Semantic-First, Shared Data:** `oxc_semantic` evaluates ALL enum member values (const and regular) during semantic analysis and stores them in `Scoping`. The transformer reads pre-computed values for single-file const enum inlining. Rolldown reads them for cross-module inlining via its existing `constant_export_map` pipeline.

### Why evaluate all enums, not just const?

Rolldown wants to inline all enum member values cross-module (matching esbuild behavior). The evaluation logic is identical for const and regular enums. The `SymbolFlags::ConstEnum` flag lets consumers distinguish when needed.

### Why semantic, not transformer?

Semantic analysis runs early in the pipeline, before both the transformer and rolldown's AstScanner. Evaluating in semantic means values are available to all consumers without pipeline restructuring.

### Rolldown pipeline constraint

In rolldown's `pre_process_ecma_ast.rs`, the pipeline is:

1. `SemanticBuilder` runs → `Scoping` (with `enum_member_values`)
2. Transformer runs → converts TS to JS
3. `recreate_scoping(&mut None, program)` → **always rebuilds** `Scoping` from the post-transform AST

The transformer handles regular enums (→ IIFE) and const enums (→ `export var X = {}` placeholder) as normal JS. The rebuilt `Scoping` at step 3 won't have `enum_member_values` (no enum declarations remain), so enum member values are extracted from step 1's `Scoping` before the transformer runs and passed through `ParseToEcmaAstResult` as a name-based `FxHashMap<CompactStr, Vec<(CompactStr, ConstantValue)>>` keyed by `(enum_name, member_name)`. The `AstScanner` then resolves enum declaration names to the new `Scoping`'s SymbolIds.

## Section 1: Data Types (`oxc_syntax`)

New file: `crates/oxc_syntax/src/constant_value.rs`

```rust
/// A compile-time constant value from a TypeScript enum member.
/// Per the TypeScript spec, enum members can only evaluate to numbers or strings.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Number(f64),
    String(CompactStr),
}
```

Only `Number` and `String` — matching what TypeScript allows as enum member values. Rolldown maps this to its richer `rolldown_common::ConstantValue` (which also has `Boolean`, `Null`, etc.) in its `AstScanner`.

**Note on `CompactStr`:** We use `CompactStr` (heap-allocated, owned) instead of `Atom<'a>` (arena-allocated) because `Scoping` is arena-independent — it outlives the allocator used during parsing. `CompactStr` is already available via `oxc_span::CompactStr` (re-exported from `oxc_str`) — no new crate dependency needed.

**Note on `f64` equality:** `PartialEq` means `NaN != NaN`. This is acceptable — the map is keyed by `SymbolId`, not by `ConstantValue`. If deduplication is ever needed, use `f64::to_bits()` for NaN-safe comparison.

## Section 2: Semantic Storage (`oxc_semantic`)

Add enum member values to `Scoping` using the sparse map pattern (like `symbol_redeclarations`):

```rust
// In crates/oxc_semantic/src/scoping.rs

pub struct Scoping {
    // ... existing fields ...

    /// Computed constant values for enum members.
    /// Keyed by the EnumMember's SymbolId, populated for ALL enums (const and regular).
    /// Only contains entries for members whose values could be statically evaluated.
    enum_member_values: FxHashMap<SymbolId, ConstantValue>,
}

impl Scoping {
    /// Get the computed constant value for an enum member symbol.
    pub fn get_enum_member_value(&self, symbol_id: SymbolId) -> Option<&ConstantValue> {
        self.enum_member_values.get(&symbol_id)
    }

    /// Set a computed constant value for an enum member symbol.
    pub(crate) fn set_enum_member_value(&mut self, symbol_id: SymbolId, value: ConstantValue) {
        self.enum_member_values.insert(symbol_id, value);
    }
}
```

**Why sparse `FxHashMap`:** Most symbols are not enum members. A dense array would waste memory. Follows the pattern of other `FxHashMap`/`FxHashSet` fields on `Scoping` (like `no_side_effects`). Empty hashmap for non-TypeScript files has negligible overhead. Since `ConstantValue` contains `CompactStr` (heap-allocated, not arena), this field goes on `Scoping` directly (not `ScopingInner`).

**What gets stored:** ALL enum members (const and regular) whose values can be statically evaluated. Members with unresolvable initializers are absent from the map. Auto-incremented members get their computed value stored.

## Section 3: Enum Value Evaluation (`oxc_semantic`)

New file: `crates/oxc_semantic/src/enum_eval.rs`

The evaluation logic is **reimplemented** based on `oxc_transformer/src/typescript/enum.rs`'s evaluator. It cannot be a direct move because the transformer's version is coupled to `TraverseCtx`, arena-allocated `Atom<'a>`, and `IdentHashMap` — none of which are available in semantic analysis. The new implementation uses `Scoping` for symbol lookup and `CompactStr` for strings.

The evaluation runs during `SemanticBuilder`'s `visit_ts_enum_declaration`, after members are bound.

```rust
pub(crate) fn evaluate_enum_members(
    decl: &TSEnumDeclaration<'_>,
    scoping: &mut Scoping,
) {
    let scope_id = decl.body.scope_id.get().unwrap();
    let mut prev_value: Option<ConstantValue> = None;

    for member in &decl.body.members {
        let value = if let Some(init) = &member.initializer {
            evaluate_expression(init, scope_id, scoping)
        } else {
            match &prev_value {
                None => Some(ConstantValue::Number(0.0)),
                Some(ConstantValue::Number(n)) => Some(ConstantValue::Number(n + 1.0)),
                Some(ConstantValue::String(_)) => None,
            }
        };

        if let Some(ref val) = value {
            // Resolve member's SymbolId via scope lookup (TSEnumMember does not store SymbolId)
            let member_name = member.id.static_name();
            if let Some(symbol_id) = scoping.get_binding(scope_id, member_name) {
                scoping.set_enum_member_value(symbol_id, val.clone());
            }
        }

        prev_value = value;
    }
}
```

**Member SymbolId resolution:** `TSEnumMember` AST nodes do not store the `SymbolId` created during binding (this is a known limitation — see `test_symbol_declaration.rs` line 118). We resolve member symbols by name via `scoping.get_binding(scope_id, member_name)` using the enum body's scope ID.

### Expression evaluation

Handles (reimplemented from transformer's existing code):

- Numeric/string literals
- Template literals (including interpolated templates where all expressions are evaluable)
- Parenthesized expressions (unwrapped)
- Unary operators: `+`, `-`, `~` (including on string values: `-"hello"` → `NaN`, `~"hello"` → `-1`)
- Binary operators: `+`, `-`, `*`, `/`, `%`, `**`, `|`, `&`, `^`, `<<`, `>>`, `>>>`
- String concatenation via `+` when either operand is a string
- References to previously-defined enum members (same enum via scope lookup, other enums via symbol resolution)
- `Infinity`, `NaN` identifiers

### Enum declaration merging

TypeScript allows enum declaration merging:

```typescript
enum Foo {
  A = 1,
}
enum Foo {
  B = A + 1,
} // B can reference A from the first declaration
```

The evaluator handles cross-declaration references through normal identifier resolution. When `evaluate_expression` encounters an `IdentifierReference` (like `A` in the second declaration):

1. Get the `ReferenceId` from the `IdentifierReference` node
2. Look up the resolved `SymbolId` via `scoping.get_reference(ref_id).symbol_id()`
3. Check `enum_member_values` for that `SymbolId` — returns the pre-computed value from the first declaration

This works because semantic analysis resolves identifier references through the scope chain before the evaluator runs. By the time the evaluator processes the second `Foo` declaration, `A`'s reference has already been resolved to the SymbolId from the first declaration's enum body scope.

**Note:** `scoping.get_binding(scope_id, member_name)` is only used for mapping the _current_ member being evaluated to its SymbolId. Cross-references (to other members or other enums) go through identifier reference resolution.

### `declare enum` and ambient enums

The evaluator runs for ALL enum declarations, including `declare enum` and `declare const enum`. Even though `declare` enums have no runtime representation, their member values must still be computed — `declare const enum` values are needed for inlining at reference sites. The transformer already skips emitting IIFEs for `declare` enums (line 78-80 of `enum.rs`); this is unchanged.

**Integration point:** In `SemanticBuilder::visit_ts_enum_declaration` (`builder.rs` line ~2368), insert the call between `self.visit_ts_enum_body(&decl.body)` and `self.leave_node(kind)`: `evaluate_enum_members(decl, &mut self.scoping)`. At this point all members are bound and have SymbolIds.

**Computed member names:** `TSEnumMemberName::static_name()` can panic on computed template strings. The evaluator must guard by checking `member.id.static_name()` returns a valid name before proceeding — use `TSEnumMemberName::static_name()` only for `Identifier` and `StringLiteral` variants; skip `ComputedProperty` members entirely since their values cannot be statically evaluated.

**Implementation note:** Adding `enum_member_values` to `Scoping` requires updating `clone_in_with_semantic_ids_with_another_arena` (scoping.rs line ~913) to clone the map. Rolldown uses this method — forgetting it would silently drop enum values.

## Section 4: Transformer Changes (`oxc_transformer`)

The transformer handles all enum → JS conversion. Enum member values are read from `Scoping` (pre-computed by semantic analysis) instead of being evaluated locally.

### Regular enums (all modes)

Transformed to IIFEs as before. The evaluator code in `enum.rs` is **removed** — the transformer reads pre-computed values from `scoping.get_enum_member_value()` to generate IIFE initializers.

### Const enums

Behavior depends on `optimize_const_enums`:

#### `optimize_const_enums: false` (default)

Const enums transform to IIFEs, same as regular enums (current behavior, matching Babel default).

#### `optimize_const_enums: true`

Two sub-modes depending on context:

**Standalone mode (default):**

1. **Declaration removal:** Const enum declarations are removed entirely (`return None`). No placeholder — all same-file references are inlined, so the binding is not needed. Cross-file const enum references are not supported in standalone/isolatedModules mode — this matches TypeScript's behavior.

   If `preserve_const_enums` is also true, emit the full IIFE instead of removing.

2. **Reference inlining:** When visiting a `StaticMemberExpression` that resolves to a const enum member:
   - Look up the member's `SymbolId` via semantic references
   - Read the pre-computed value from `scoping.get_enum_member_value(symbol_id)`
   - Replace the expression with the literal value + comment annotation: `0 /* Direction.Up */`

**Rolldown mode** (`emit_const_enum_placeholder: true`):

1. **Declaration → placeholder:** Const enum declarations are replaced with an empty object to preserve the export binding for cross-module resolution:

   ```ts
   export const enum Direction {
     Up = 0,
     Down = 1,
   }
   ```

   becomes:

   ```js
   export var Direction = {};
   ```

   Rolldown's finalizer removes these placeholders after inlining (see Section 5).

   If `preserve_const_enums` is also true, emit the full IIFE instead of the placeholder.

2. **No reference inlining** — rolldown handles cross-module inlining at the bundler level.

### Deleted code

- `ConstantValue` type (moved to `oxc_syntax`)
- `computed_constant_value()`, `evaluate()`, `eval_binary_expression()`, `eval_unary_expression()`, `evaluate_ref()` (reimplemented in `oxc_semantic`)
- `enums: IdentHashMap` tracking (no longer needed)

## Section 5: Rolldown Integration

### Pipeline flow in `pre_process_ecma_ast.rs`

1. **Step 1** (`SemanticBuilder`): Runs on the TS AST. `enum_member_values` are computed and stored in `Scoping`.
2. **Extract enum values**: Before the transformer runs, extract enum values from step 1's `Scoping` into the name-based format for `ParseToEcmaAstResult`. For each enum declaration symbol, iterate its body scope's bindings, look up each member's computed value, and build `FxHashMap<CompactStr, Vec<(CompactStr, ConstantValue)>>`. Also collect const enum names into `FxHashSet<CompactStr>`. Add a `pub fn take_enum_member_values(&mut self) -> FxHashMap<SymbolId, ConstantValue>` method to `Scoping` for extraction.
3. **Step 3** (Transformer with `optimize_const_enums: true`, `emit_const_enum_placeholder: true`): Regular enums → IIFEs. Const enums → `var X = {}` placeholder.
4. **Step 6** (`recreate_scoping`): Scoping is rebuilt from the JS AST. Const enum member symbols no longer exist, but the enum declaration symbol survives as a variable (`var Direction = {}`).
5. **Pass through**: The extracted enum values and const enum names are passed through `ParseToEcmaAstResult` alongside the rebuilt `Scoping`.

```rust
pub struct ParseToEcmaAstResult {
    pub ast: EcmaAst,
    pub scoping: Scoping,
    // ... existing fields ...

    /// Enum member constant values, keyed by enum declaration name.
    /// Each entry maps member names to their computed constant values.
    /// Used by AstScanner to populate constant_export_map.
    pub enum_member_values: FxHashMap<CompactStr, Vec<(CompactStr, ConstantValue)>>,

    /// Names of const enum declarations (for placeholder removal in finalizer).
    /// The finalizer matches these against SymbolRef names to identify placeholders.
    pub const_enum_names: FxHashSet<CompactStr>,
}
```

**Re-mapping to SymbolIds:** `AstScanner` resolves enum declaration names to the new `Scoping`'s SymbolIds by looking up the variable declaration (the `var Direction = {}` placeholder or IIFE result) by name. For const enums, it also marks the SymbolRef in `SymbolRefDb` so the finalizer can identify and remove placeholders.

**Non-exported const enums:** The transformer inlines all same-file references during transformation (standalone mode). The placeholder `var X = {}` is still emitted for non-exported const enums but is also removed by the finalizer. Cross-module inlining only applies to exported enums — `constant_export_map` entries for non-exported enums are simply ignored by rolldown's export resolution.

### AstScanner consumes enum values

`AstScanner` receives the enum member values from `ParseToEcmaAstResult` and populates `constant_export_map`. For each enum, it matches the enum declaration's SymbolId (which exists in the new Scoping as a variable) and maps member names to the constant values. Rolldown's existing namespace alias inlining (`try_inline_constant_from_namespace_alias`) handles `Direction.Up` → `0` by resolving the property name against the enum's member values.

This feeds into rolldown's existing pipeline:

- `constant_export_map` per module (existing)
- `global_constant_symbol_map` at link stage (existing)
- `cross_module_optimization()` for transitive resolution (existing)
- `finalized_expr_for_symbol_ref()` for codegen inlining (existing)

No changes to link stage, cross-module optimization, or finalization.

### Const enum placeholder removal

The `export var Direction = {};` placeholders are removed **explicitly by rolldown's finalizer**, not by tree-shaking. This is a correctness requirement — const enums have no runtime representation — so it must work regardless of whether tree-shaking is enabled.

The mechanism:

1. `ParseToEcmaAstResult.const_enum_names` carries the set of const enum declaration names from step 1.
2. `AstScanner` resolves these names to SymbolRefs in the new Scoping and sets `SymbolRefFlags::ConstEnumPlaceholder` on them in `SymbolRefDb`.
3. The finalizer checks `SymbolRefFlags::ConstEnumPlaceholder` when emitting variable declarations and removes matching placeholders.

This also handles non-exported const enums — the placeholder `var X = {}` is emitted regardless of export status and removed by the finalizer.

## Section 6: Testing Strategy

### oxc_semantic tests

Unit tests for enum value evaluation:

- Numeric auto-increment (`enum A { X, Y, Z }` → 0, 1, 2)
- Explicit numeric initializers, mixed with auto-increment
- String members and the must-initialize-after-string rule
- Binary/unary expressions (`1 << 2`, `~0`, `1 + 2`)
- String concatenation (`"a" + "b"`, `"x" + 1`)
- Unary operators on strings (`-"hello"` → `NaN`, `~"hello"` → `-1`)
- Parenthesized expressions
- Template literals with interpolation
- Cross-member references within same enum (`enum A { X = 1, Y = X + 1 }`)
- Cross-enum references (`const enum A { X = 1 } const enum B { Y = A.X }`)
- Enum declaration merging (`enum Foo { A = 1 } enum Foo { B = A + 1 }`)
- `declare enum` and `declare const enum` (values computed, no runtime emit)
- Unresolvable initializers (function calls, runtime values) → absent from map
- `Infinity`, `NaN` handling
- Computed member names

### oxc_transformer tests

Babel conformance fixtures for `optimizeConstEnums`:

- Const enum declaration removed
- References replaced with literals + comment annotation
- `preserveConstEnums` emits IIFE + still inlines
- Regular enums unaffected by the option
- Interaction with exports (`export const enum`)
- `declare const enum` references inlined
- Existing enum transform tests still pass

### rolldown tests

- Existing esbuild test fixtures (`ts_enum_cross_module_inlining_*`) should start passing or improve
- Const enum values flow through the cross-module pipeline
- Module A exports const enum → module B imports and uses member → value inlined in bundle output

## `preserve_const_enums` Interaction

When `preserve_const_enums` is true AND `optimize_const_enums` is true, the transformer emits a full IIFE instead of a placeholder. Enum member values are still extracted and passed through `ParseToEcmaAstResult` — rolldown still inlines cross-module member accesses. The IIFE remains in the output for debugging/runtime access purposes.

## Type References

TypeScript allows using const enums as types (`let x: Direction`). These type references are stripped by the transformer's existing TypeScript annotation removal, before any enum-specific handling. No special treatment needed.

## `isolatedModules` Consideration

When `isolatedModules: true` (the default for most build tools), TypeScript disallows cross-file const enum references unless the const enum is `declare`d or has `preserveConstEnums`. Oxc's single-file inlining in the transformer naturally aligns with `isolatedModules` — it only inlines const enum members defined in the same file. Cross-module inlining is handled exclusively by rolldown at the bundler level, which has full cross-module visibility regardless of `isolatedModules`.

## Data Flow Diagrams

### Standalone (oxc_transformer)

```
Source Code
    |
    v
oxc_parser (AST with TSEnumDeclaration)
    |
    v
oxc_semantic (SemanticBuilder)
    |-- Binder: SymbolFlags::ConstEnum/RegularEnum, binds members
    |-- enum_eval: evaluates member values, stores in Scoping
    |
    v
Scoping { enum_member_values: FxHashMap<SymbolId, ConstantValue> }
    |
    v
oxc_transformer
    |-- regular enums → IIFE (reads values from Scoping)
    |-- const enums (optimize off) → IIFE
    |-- const enums (optimize on) → remove decl + inline refs
```

### Rolldown (bundler)

```
Source Code
    |
    v
[Step 1] SemanticBuilder → Scoping (enum_member_values computed)
    |
    v
[Step 1.5] Extract enum_member_values from Scoping (name-based mapping)
    |
    v
[Step 3] Transformer (optimize_const_enums: true)
          regular enums → IIFE
          const enums → export var X = {} (placeholder)
    |
    v
[Step 6] recreate_scoping() → fresh Scoping (no enum members)
    |
    v
ParseToEcmaAstResult { scoping, enum_member_values (by enum name + member name), const_enum_names }
    |
    v
AstScanner: maps enum_member_values → constant_export_map (via SymbolId matching)
    |
    v
LinkStage → global_constant_symbol_map (existing)
    |
    v
CrossModuleOptimization: transitive constant discovery (existing)
    |
    v
Codegen: inlines enum member values at use sites (existing)
         finalizer removes const enum placeholders (SymbolRefFlags::ConstEnumPlaceholder)
```
