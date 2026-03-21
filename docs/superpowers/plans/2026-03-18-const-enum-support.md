# Const Enum Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Evaluate enum member values in `oxc_semantic` and support const enum inlining in `oxc_transformer`.

**Architecture:** Enum member values are evaluated during semantic analysis and stored in `Scoping` as `FxHashMap<SymbolId, ConstantValue>`. The transformer reads pre-computed values instead of evaluating locally. When `optimize_const_enums` is enabled, const enum declarations are removed and references are inlined with literal values.

**Tech Stack:** Rust, oxc_syntax, oxc_semantic, oxc_transformer

**Spec:** `docs/superpowers/specs/2026-03-18-const-enum-support-design.md`

**Scope:** oxc crates only. Rolldown integration is a separate follow-up plan.

---

## File Map

| File                                                   | Action                         | Responsibility                                                        |
| ------------------------------------------------------ | ------------------------------ | --------------------------------------------------------------------- |
| `crates/oxc_syntax/src/constant_value.rs`              | Create                         | `ConstantValue` enum type                                             |
| `crates/oxc_syntax/src/lib.rs`                         | Modify (line ~8)               | Add `constant_value` module declaration                               |
| `crates/oxc_semantic/src/scoping.rs`                   | Modify (lines 88-101, 910-945) | Add `enum_member_values` field, `get/set/take` methods, clone support |
| `crates/oxc_semantic/src/enum_eval.rs`                 | Create                         | Enum member value evaluator                                           |
| `crates/oxc_semantic/src/lib.rs`                       | Modify (line 34)               | Add `enum_eval` module declaration                                    |
| `crates/oxc_semantic/src/builder.rs`                   | Modify (line 2375)             | Call `evaluate_enum_members` after visiting enum body                 |
| `crates/oxc_semantic/tests/integration/enum_values.rs` | Create                         | Tests for enum value evaluation                                       |
| `crates/oxc_semantic/tests/integration/main.rs`        | Modify                         | Add `mod enum_values;`                                                |
| `crates/oxc_transformer/src/typescript/enum.rs`        | Modify                         | Remove evaluator, read from Scoping, add const enum inlining          |
| `crates/oxc_transformer/src/typescript/options.rs`     | Modify (line 89)               | Activate `optimize_const_enums` option                                |

---

### Task 1: Add `ConstantValue` type to `oxc_syntax`

**Files:**

- Create: `crates/oxc_syntax/src/constant_value.rs`
- Modify: `crates/oxc_syntax/src/lib.rs:22`

- [ ] **Step 1: Create the `ConstantValue` type**

```rust
// crates/oxc_syntax/src/constant_value.rs
use oxc_span::CompactStr;

/// A compile-time constant value from a TypeScript enum member.
/// Per the TypeScript spec, enum members can only evaluate to numbers or strings.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Number(f64),
    String(CompactStr),
}
```

- [ ] **Step 2: Register the module in `lib.rs`**

In `crates/oxc_syntax/src/lib.rs`, add after the `class` module (line 7):

```rust
pub mod constant_value;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p oxc_syntax`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add crates/oxc_syntax/src/constant_value.rs crates/oxc_syntax/src/lib.rs
git commit -m "feat(syntax): add ConstantValue type for enum member values"
```

---

### Task 2: Add `enum_member_values` storage to `Scoping`

**Files:**

- Modify: `crates/oxc_semantic/src/scoping.rs:88-101` (struct), `crates/oxc_semantic/src/scoping.rs:910-945` (clone method)

- [ ] **Step 1: Add the field and methods to `Scoping`**

In `crates/oxc_semantic/src/scoping.rs`, add to the `Scoping` struct (after `no_side_effects` at line 95):

```rust
    /// Computed constant values for enum members.
    /// Keyed by the EnumMember's SymbolId, populated for ALL enums (const and regular).
    /// Only contains entries for members whose values could be statically evaluated.
    pub(crate) enum_member_values: FxHashMap<SymbolId, ConstantValue>,
```

Add the import at the top of the file:

```rust
use oxc_syntax::constant_value::ConstantValue;
```

Add methods in the main `impl Scoping` block:

```rust
    /// Get the computed constant value for an enum member symbol.
    pub fn get_enum_member_value(&self, symbol_id: SymbolId) -> Option<&ConstantValue> {
        self.enum_member_values.get(&symbol_id)
    }

    /// Set a computed constant value for an enum member symbol.
    pub(crate) fn set_enum_member_value(&mut self, symbol_id: SymbolId, value: ConstantValue) {
        self.enum_member_values.insert(symbol_id, value);
    }

    /// Take all enum member values out of this Scoping.
    /// Used by rolldown to extract values before scoping is rebuilt.
    pub fn take_enum_member_values(&mut self) -> FxHashMap<SymbolId, ConstantValue> {
        std::mem::take(&mut self.enum_member_values)
    }
```

- [ ] **Step 2: Update `Default` impl**

Find the `impl Default for Scoping` block and add `enum_member_values: FxHashMap::default()` to the struct initialization.

- [ ] **Step 3: Update `clone_in_with_semantic_ids_with_another_arena`**

At `crates/oxc_semantic/src/scoping.rs:917`, add to the `Self { ... }` block:

```rust
            enum_member_values: self.enum_member_values.clone(),
```

(After `no_side_effects: self.no_side_effects.clone(),` on line 920)

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p oxc_semantic`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add crates/oxc_semantic/src/scoping.rs
git commit -m "feat(semantic): add enum_member_values storage to Scoping"
```

---

### Task 3: Implement enum value evaluator in `oxc_semantic`

**Files:**

- Create: `crates/oxc_semantic/src/enum_eval.rs`
- Modify: `crates/oxc_semantic/src/lib.rs:34`
- Modify: `crates/oxc_semantic/src/builder.rs:2375`

This is the largest task. The evaluator is reimplemented from `crates/oxc_transformer/src/typescript/enum.rs` lines 357-549, adapted to use `Scoping` instead of `TraverseCtx`.

- [ ] **Step 1: Create `enum_eval.rs` with the evaluator**

Create `crates/oxc_semantic/src/enum_eval.rs`. The evaluator must handle:

- Numeric/string literals
- Template literals (single quasi and interpolated)
- Parenthesized expressions
- Unary operators: `+`, `-`, `~` (including on strings)
- Binary operators: all arithmetic and bitwise
- String concatenation via `+`
- Identifier references resolved via `scoping.get_reference(ref_id).symbol_id()` → `enum_member_values` lookup
- Member expressions (`Enum.Member`) resolved via object identifier → enum members
- `Infinity`, `NaN` identifiers
- Auto-increment (no initializer → previous + 1, or 0 if first)

Guard against `static_name()` panics: only call for `Identifier` and `String`/`ComputedString` variants; skip `ComputedTemplateString` with substitutions.

Reference implementation: `crates/oxc_transformer/src/typescript/enum.rs` lines 357-549.

Key differences from transformer version:

- Uses `Scoping` for symbol lookup instead of `IdentHashMap` + `TraverseCtx`
- Uses `CompactStr` instead of arena-allocated `Atom<'a>`
- Cross-enum/cross-declaration references resolved via `scoping.get_reference(ref_id).symbol_id()` → `get_enum_member_value()`
- String building uses `String`/`CompactStr::from()` instead of `StringBuilder`

```rust
// crates/oxc_semantic/src/enum_eval.rs
use oxc_ast::ast::*;
use oxc_ecmascript::{ToInt32, ToUint32};
use oxc_span::CompactStr;
use oxc_syntax::{
    constant_value::ConstantValue,
    number::ToJsString,
    operator::{BinaryOperator, UnaryOperator},
};

use crate::scoping::Scoping;

/// Evaluates all members of an enum declaration and stores results in Scoping.
pub(crate) fn evaluate_enum_members(decl: &TSEnumDeclaration<'_>, scoping: &mut Scoping) {
    let Some(scope_id) = decl.body.scope_id.get() else { return };
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
            let member_name = match &member.id {
                TSEnumMemberName::Identifier(ident) => Some(ident.name.as_str()),
                TSEnumMemberName::String(lit) | TSEnumMemberName::ComputedString(lit) => {
                    Some(lit.value.as_str())
                }
                TSEnumMemberName::ComputedTemplateString(_) => None,
            };
            if let Some(name) = member_name {
                if let Some(symbol_id) = scoping.get_binding(scope_id, name) {
                    scoping.set_enum_member_value(symbol_id, val.clone());
                }
            }
        }

        prev_value = value;
    }
}
```

The `evaluate_expression`, `evaluate_ref`, `eval_binary_expression`, and `eval_unary_expression` functions follow the same logic as the transformer (lines 375-549 of `enum.rs`), with these adaptations:

- `evaluate_ref` for `Identifier`: check `Infinity`/`NaN` first, then resolve via `scoping.get_reference(ref_id).symbol_id()` → `scoping.get_enum_member_value()`
- `evaluate_ref` for `MemberExpression`: resolve object identifier's symbol, then check if it's an enum, iterate scope bindings to find the member
- String building in `eval_binary_expression` and template literals: use `String` + `CompactStr::from()` instead of `StringBuilder`

- [ ] **Step 2: Register the module**

In `crates/oxc_semantic/src/lib.rs`, add after `diagnostics` (around line 34):

```rust
mod enum_eval;
```

- [ ] **Step 3: Wire into `SemanticBuilder`**

In `crates/oxc_semantic/src/builder.rs`, find `visit_ts_enum_declaration` (line 2368). Add the evaluator call before `self.leave_node(kind)` (line 2375):

```rust
    fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
        let kind = AstKind::TSEnumDeclaration(self.alloc(decl));
        self.enter_node(kind);
        decl.bind(self);
        self.visit_span(&decl.span);
        self.visit_binding_identifier(&decl.id);
        self.visit_ts_enum_body(&decl.body);
        // Evaluate enum member values after all members are bound
        crate::enum_eval::evaluate_enum_members(decl, &mut self.scoping);
        self.leave_node(kind);
    }
```

Add the import at the top if needed.

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p oxc_semantic`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add crates/oxc_semantic/src/enum_eval.rs crates/oxc_semantic/src/lib.rs crates/oxc_semantic/src/builder.rs
git commit -m "feat(semantic): implement enum member value evaluation"
```

---

### Task 4: Add semantic tests for enum value evaluation

**Files:**

- Create: `crates/oxc_semantic/tests/integration/enum_values.rs`
- Modify: `crates/oxc_semantic/tests/integration/main.rs`

- [ ] **Step 1: Check existing test infrastructure**

Read `crates/oxc_semantic/tests/integration/main.rs` to understand the test module structure. Read an existing test file (e.g., `crates/oxc_semantic/tests/integration/scopes.rs`) to understand test patterns — how `SemanticBuilder` is used, how to get `Scoping`, etc.

- [ ] **Step 2: Write tests for basic enum evaluation**

Create `crates/oxc_semantic/tests/integration/enum_values.rs` with tests covering:

```rust
// Test helper: parse source, run semantic, return Scoping
// Use the same pattern as existing integration tests

// Test: auto-increment
// enum A { X, Y, Z } → X=0, Y=1, Z=2

// Test: explicit numeric
// enum A { X = 10, Y, Z } → X=10, Y=11, Z=12

// Test: string members
// enum A { X = "hello" } → X="hello"

// Test: binary expressions
// enum A { X = 1 + 2, Y = 1 << 3 } → X=3, Y=8

// Test: unary expressions
// enum A { X = -1, Y = ~0 } → X=-1, Y=-1

// Test: cross-member references
// enum A { X = 1, Y = X + 1 } → X=1, Y=2

// Test: Infinity/NaN
// enum A { X = Infinity, Y = NaN } → X=Infinity, Y=NaN

// Test: string concat
// enum A { X = "a" + "b" } → X="ab"

// Test: cross-enum member expression
// enum A { X = 1 } enum B { Y = A.X + 1 } → A.X=1, B.Y=2

// Test: unary plus
// enum A { X = +"5" } → X=NaN (string to number)

// Test: unresolvable
// enum A { X = foo() } → X absent from map

// Test: const enum (same evaluation, different SymbolFlags)
// const enum A { X, Y } → X=0, Y=1 + flags check
```

- [ ] **Step 3: Add module to `main.rs`**

In `crates/oxc_semantic/tests/integration/main.rs`, add:

```rust
mod enum_values;
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p oxc_semantic -- enum_values`
Expected: all tests PASS

- [ ] **Step 5: Add edge case tests**

Add tests for:

- Enum declaration merging (`enum Foo { A = 1 } enum Foo { B = A + 1 }`)
- `declare enum` and `declare const enum` (values computed)
- Template literals with interpolation
- Parenthesized expressions
- Unary on strings (`-"hello"` → NaN)
- Auto-increment after string (must have initializer → None)
- Computed member names (skipped)

- [ ] **Step 6: Run all tests**

Run: `cargo test -p oxc_semantic -- enum_values`
Expected: all tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/oxc_semantic/tests/integration/enum_values.rs crates/oxc_semantic/tests/integration/main.rs
git commit -m "test(semantic): add enum value evaluation tests"
```

---

### Task 5: Refactor transformer to read enum values from Scoping

**Files:**

- Modify: `crates/oxc_transformer/src/typescript/enum.rs`

This task replaces the transformer's local evaluator with reads from `Scoping`. The existing IIFE generation logic is preserved — only the value computation changes.

- [ ] **Step 1: Read the transformer's current enum code**

Read `crates/oxc_transformer/src/typescript/enum.rs` fully (657 lines). Key areas:

- `transform_ts_enum()` (line 72-207): main entry point, generates IIFE from enum declaration
- `transform_ts_enum_members()` (line 209-354): iterates members, calls `computed_constant_value` to get values, generates IIFE body statements. The value is used at line ~234 to decide the initializer expression.
- `enums: IdentHashMap` (lines 25-27): tracks member values across enum declarations for merging
- `ConstantValue<'a>` (lines 357-361): local enum type
- Evaluator methods (lines 363-549): `computed_constant_value`, `evaluate_ref`, `evaluate`, `eval_binary_expression`, `eval_unary_expression`
- `IdentifierReferenceRename` (lines 552-656): renames unqualified enum member references to `EnumName.member`. This struct uses `prev_members` (a `&PrevMembers`) to check if an identifier is an enum member. After refactoring, this should use `Scoping` to check `SymbolFlags::EnumMember` instead.

- [ ] **Step 2: Refactor in one pass — replace evaluator with Scoping lookups**

This is a combined step because the changes are interdependent and won't compile independently. Do all of the following together:

1. Add import: `use oxc_syntax::constant_value::ConstantValue;`

2. In `transform_ts_enum_members()`, where `computed_constant_value()` is called (~line 234), replace with a Scoping lookup:

   ```rust
   let scope_id = decl.body.scope_id.get().unwrap();
   let value = ctx.scoping().get_binding(scope_id, &member_name)
       .and_then(|sym_id| ctx.scoping().get_enum_member_value(sym_id))
       .cloned();
   ```

3. Update value → expression conversion: the new `ConstantValue` uses `CompactStr` instead of `Atom<'a>`, so `ConstantValue::String(s)` needs `ctx.ast.atom(&s)` when creating string literal AST nodes.

4. Update `IdentifierReferenceRename`: replace the `prev_members: &PrevMembers` field with a check against `SymbolFlags::EnumMember` via `ctx.scoping()`. When an identifier reference resolves to a symbol with `EnumMember` flag, rename it.

5. Delete the old evaluator code:
   - `ConstantValue<'a>` enum (lines 357-361)
   - `computed_constant_value()`, `evaluate_ref()`, `evaluate()`, `eval_binary_expression()`, `eval_unary_expression()` (lines 363-549)
   - `PrevMembers` type alias (line 20)
   - `enums: IdentHashMap` field (line 26) and its population logic in `enter_statement`

6. Simplify `TypeScriptEnum` struct to have no fields (or only config fields).

- [ ] **Step 3: Verify existing tests pass**

Run: `cargo check -p oxc_transformer`
Expected: compiles

Run: `cargo test -p oxc_transformer -- enum`
Expected: all existing enum tests PASS

Run: `just test-transform`
Expected: no new failures

- [ ] **Step 4: Commit**

```bash
git add crates/oxc_transformer/src/typescript/enum.rs
git commit -m "refactor(transformer): read enum values from Scoping instead of evaluating locally"
```

---

### Task 6: Implement const enum inlining in transformer (standalone mode)

**Files:**

- Modify: `crates/oxc_transformer/src/typescript/enum.rs`
- Modify: `crates/oxc_transformer/src/typescript/options.rs:89`
- Modify: `crates/oxc_transformer/src/typescript/mod.rs`

- [ ] **Step 1: Activate the `optimize_const_enums` option**

In `crates/oxc_transformer/src/typescript/options.rs`, remove the `/// Unused.` comment from `optimize_const_enums` (line 89). Wire it through to `TypeScriptEnum` so the enum transformer can check it.

- [ ] **Step 2: Add `emit_const_enum_placeholder` option**

In the same options file, add:

```rust
    /// When true, replace const enum declarations with `var X = {}` placeholder
    /// instead of removing them entirely. Used by rolldown for cross-module resolution.
    pub emit_const_enum_placeholder: bool,
```

Default: `false`.

- [ ] **Step 3: Implement const enum declaration handling**

In `enum.rs`, modify `transform_ts_enum()` (around line 72). When `optimize_const_enums` is true and the enum is `const`:

**Standalone mode** (`emit_const_enum_placeholder: false`):

- Return `None` to remove the declaration entirely
- Unless `preserve_const_enums` is true → emit IIFE as usual

**Rolldown mode** (`emit_const_enum_placeholder: true`):

- Replace with `var EnumName = {};`
- Unless `preserve_const_enums` is true → emit IIFE as usual

- [ ] **Step 4: Implement reference inlining for standalone mode**

In `crates/oxc_transformer/src/typescript/mod.rs`, the `TypeScript` struct implements `Traverse` with various visitor methods. Add a new method `enter_expression` (or extend it if it exists) that handles `StaticMemberExpression` inlining.

When the object of a `StaticMemberExpression` resolves to a const enum symbol and `optimize_const_enums` is true and `emit_const_enum_placeholder` is false:

1. Check if the expression is `Expression::StaticMemberExpression`
2. Get the object's `IdentifierReference` → resolve to SymbolId via `ctx.scoping()`
3. Check `SymbolFlags::ConstEnum` on the symbol
4. Get the property name (member name)
5. Find the enum body scope (stored as a child scope of the enum declaration's scope), look up member by name → get `ConstantValue` from `ctx.scoping().get_enum_member_value()`
6. Replace the entire `StaticMemberExpression` with a literal expression (number or string)
7. Attach a trailing comment: `/* EnumName.MemberName */` using `ctx.ast.add_leading_comment()` or the codegen comment API

- [ ] **Step 5: Run existing tests**

Run: `cargo test -p oxc_transformer -- enum`
Expected: all existing tests PASS (optimize_const_enums defaults to false)

- [ ] **Step 6: Add transform conformance test fixtures**

Add test fixtures for const enum inlining. Check existing fixtures at `tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/` for the pattern. Add:

- `const-enum-inline/input.ts` + `output.js`: basic const enum removal + inlining
- `const-enum-preserve/input.ts` + `output.js`: with `preserveConstEnums`
- `const-enum-placeholder/input.ts` + `output.js`: rolldown mode

- [ ] **Step 7: Run conformance tests**

Run: `just test-transform`
Expected: new fixtures PASS, no regressions

- [ ] **Step 8: Commit**

```bash
git add crates/oxc_transformer/src/typescript/enum.rs crates/oxc_transformer/src/typescript/options.rs crates/oxc_transformer/src/typescript/mod.rs
git commit -m "feat(transformer): implement const enum inlining and placeholder mode"
```

---

### Task 7: Final validation

- [ ] **Step 1: Run full test suite**

Run: `just ready`
Expected: all checks pass

- [ ] **Step 2: Run conformance tests**

Run: `just conformance`
Expected: no regressions (pass count should only increase or stay the same)

- [ ] **Step 3: Verify NAPI builds**

Run: `cargo check -p oxc_napi_transform`
Expected: compiles (the NAPI layer currently hardcodes `optimize_const_enums: false`)

- [ ] **Step 4: Commit any fixes**

If any issues found, fix and commit.
