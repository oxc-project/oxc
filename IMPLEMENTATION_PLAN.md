# Plan: Integrate `oxc_module_graph` into Rolldown (Full Replacement)

## Context

`oxc_module_graph` (in oxc repo, already implemented with 17 passing tests) provides trait-based cross-module analysis with a generic `bind_imports_and_exports` algorithm. Rolldown has its own ~1050-line implementation. We want to replace Rolldown's core binding logic with oxc_module_graph's shared algorithm.

**Challenge:** The current oxc_module_graph traits use concrete types that don't match Rolldown's types. We need to refactor the traits to use associated types so Rolldown can implement them directly.

**Rolldown's binding has 5 parts:**
1. **Core binding** (init exports → star re-exports → match imports → link symbols) — **replaceable** by oxc_module_graph
2. **CJS interop** (CommonJS detection, dynamic fallback, namespace aliases) — stays in Rolldown
3. **External import merging** (ESM facade symbols) — stays in Rolldown
4. **Member expression resolution** (foo_ns.bar_ns.c chains) — stays in Rolldown
5. **Ambiguity post-filtering** — stays in Rolldown

The shared algorithm replaces part (1); parts (2-5) remain as Rolldown-specific post-processing.

## Type Mismatches

| oxc_module_graph | Rolldown | Issue |
|---|---|---|
| `ModuleIdx` | `ModuleIdx` | Same repr (u32), different Rust types |
| `SymbolRef { owner, symbol }` | `SymbolRef { owner, symbol }` | Same repr, different ModuleIdx |
| `NamedImport { imported_name: CompactString, ... }` | `NamedImport { imported: Specifier, ... }` | Different fields |
| `LocalExport { exported_name, local_symbol }` | `LocalExport { span, referenced, came_from_commonjs }` | Different fields |
| `FxHashMap<SymbolRef, NamedImport>` | `FxIndexMap<SymbolRef, NamedImport>` | Different collection |
| `StarExportEntry` struct | `ImportRecordMeta::IsExportStar` flag | Different representation |

## Stage 1: Refactor oxc_module_graph traits to use associated types

### Status: Complete (committed)
- 17/17 tests passing
- Zero clippy warnings
- Formatting clean

### Trait signatures (final)

```rust
pub trait SymbolGraph {
    type SymbolRef: Copy + Eq + Hash + Debug;
    fn canonical_ref_for(&self, symbol: Self::SymbolRef) -> Self::SymbolRef;
    fn link(&mut self, from: Self::SymbolRef, to: Self::SymbolRef);
    fn symbol_name(&self, symbol: Self::SymbolRef) -> &str;
}

pub trait ModuleInfo {
    type ModuleIdx: Copy + Eq + Hash + Debug;
    type SymbolRef: Copy + Eq + Hash + Debug;
    fn module_idx(&self) -> Self::ModuleIdx;
    fn default_export_ref(&self) -> Self::SymbolRef;
    fn namespace_object_ref(&self) -> Self::SymbolRef;
    fn has_module_syntax(&self) -> bool;
    fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef, bool));
    fn for_each_named_import(&self, f: &mut dyn FnMut(Self::SymbolRef, &str, usize, bool));
    fn import_record_count(&self) -> usize;
    fn import_record_resolved_module(&self, idx: usize) -> Option<Self::ModuleIdx>;
    fn for_each_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx));
    fn for_each_indirect_export(&self, f: &mut dyn FnMut(&str, &str, Self::ModuleIdx));
}

pub trait ModuleStore {
    type ModuleIdx: Copy + Eq + Hash + Debug;
    type SymbolRef: Copy + Eq + Hash + Debug;
    type Module: ModuleInfo<ModuleIdx = Self::ModuleIdx, SymbolRef = Self::SymbolRef>;
    fn module(&self, idx: Self::ModuleIdx) -> Option<&Self::Module>;
    fn modules_len(&self) -> usize;
    fn for_each_module(&self, f: &mut dyn FnMut(Self::ModuleIdx, &Self::Module));
    fn for_each_dependency(&self, idx: Self::ModuleIdx, f: &mut dyn FnMut(Self::ModuleIdx));
}
```

### Files modified (oxc repo)

- `crates/oxc_module_graph/src/traits/module_info.rs`
- `crates/oxc_module_graph/src/traits/module_store.rs`
- `crates/oxc_module_graph/src/traits/symbol_graph.rs`
- `crates/oxc_module_graph/src/traits/mod.rs`
- `crates/oxc_module_graph/src/lib.rs`
- `crates/oxc_module_graph/src/algo/binding.rs`
- `crates/oxc_module_graph/src/algo/topo_sort.rs`
- `crates/oxc_module_graph/src/algo/cycles.rs`
- `crates/oxc_module_graph/src/algo/mod.rs`
- `crates/oxc_module_graph/src/types/import_export.rs`
- `crates/oxc_module_graph/src/default/module.rs`
- `crates/oxc_module_graph/src/default/graph.rs`
- `crates/oxc_module_graph/src/default/symbol_db.rs`
- `crates/oxc_module_graph/tests/integration.rs`

## Stage 2: Implement traits in Rolldown

### Status: Complete
- `cargo check -p rolldown_common` passes
- `cargo check -p rolldown` passes (only pre-existing warnings)

### Files created (rolldown repo)

- `crates/rolldown_common/src/oxc_module_graph_impls.rs` — all 3 trait impls

### Files modified (rolldown repo)

- `crates/rolldown_common/Cargo.toml` — added `oxc_module_graph = { path = "../../../oxc/crates/oxc_module_graph" }`
- `crates/rolldown_common/src/lib.rs` — registered `oxc_module_graph_impls` module

### Trait impls (actual)

- `SymbolGraph for SymbolRefDb` — direct delegation to existing methods
- `ModuleInfo for NormalModule` — adapts field names (`imported_as` for `local_symbol`, `Specifier` for imported name, `ImportRecordIdx::from_raw` for index conversion), passes `came_from_commonjs` from `LocalExport`
- `ModuleStore for ModuleTable` — skips `Module::External` variants, iterates `Module::Normal` only

## Stage 3: CJS-aware resolved exports + `build_resolved_exports` API

### Status: Complete
- 17/17 tests passing in oxc_module_graph
- Zero clippy warnings
- `cargo check -p rolldown` passes

### Design: Modular algorithm split

Instead of replacing Rolldown's entire binding at once, we split the algorithm:

**Phase 1 (shared, `build_resolved_exports`):**
- Initialize resolved exports from local exports (with `came_from_cjs` flag)
- Propagate star re-exports with CJS-aware semantics
- Returns `FxHashMap<ModuleIdx, ResolvedExportsMap<SymbolRef>>`

**Phase 2 (consumer-specific):**
- Import matching — stays in Rolldown due to CJS/external/dynamic fallback complexity
- The default `bind_imports_and_exports` provides a simple ESM-only Phase 2

### Changes made (oxc repo)

1. **Extended `ModuleInfo::for_each_named_export`** with `came_from_cjs: bool` parameter
   - Trait signature: `fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef, bool))`
   - Default impl passes `false` (pure ESM modules)

2. **Added `came_from_cjs: bool` to `ResolvedExport<S>`**
   - Tracks CJS origin through star re-export chains

3. **CJS-aware star re-export logic in `add_exports_for_star`:**
   - ESM "default" exports skipped during `export *` (standard behavior)
   - CJS "default" exports propagated through `export *`
   - Ambiguity detection suppressed when existing export `came_from_cjs`

4. **Extracted `build_resolved_exports` as public function**
   - Phase 1 only: init exports + star re-export propagation
   - Rolldown can call this directly and do its own Phase 2 import matching
   - `bind_imports_and_exports` now delegates to `build_resolved_exports` internally

### Changes made (rolldown repo)

- Updated `for_each_named_export` impl to pass `export.came_from_commonjs`

### Files modified

**oxc repo:**
- `crates/oxc_module_graph/src/traits/module_info.rs` — extended callback signature
- `crates/oxc_module_graph/src/types/import_export.rs` — added `came_from_cjs` to `ResolvedExport`
- `crates/oxc_module_graph/src/algo/binding.rs` — CJS-aware logic, extracted `build_resolved_exports`
- `crates/oxc_module_graph/src/algo/mod.rs` — re-export `build_resolved_exports`
- `crates/oxc_module_graph/src/lib.rs` — re-export at crate root, updated doc comments
- `crates/oxc_module_graph/src/default/module.rs` — updated impl to pass `false`
- `crates/oxc_module_graph/tests/integration.rs` — updated closure signatures

**rolldown repo:**
- `crates/rolldown_common/src/oxc_module_graph_impls.rs` — updated `for_each_named_export`

## Stage 4: Replace Phase 1 in Rolldown's LinkStage

### Status: Complete
- All 1,680 Rolldown tests pass (0 failures)

- `cargo check -p rolldown` passes (only pre-existing warnings)
- Replaced Phase 1 init + star re-export propagation with `oxc_module_graph::build_resolved_exports`
- Removed dead `add_exports_for_export_star()` method (~67 lines)
- Removed unused `EcmaModuleAstUsage` import

### Changes made (rolldown repo)

1. **Extended `for_each_star_export`** in `oxc_module_graph_impls.rs` to also yield `IsCjsReexport` targets
2. **Added `oxc_module_graph` dep** to `crates/rolldown/Cargo.toml`
3. **Replaced Phase 1** in `bind_imports_and_exports()` with `oxc_module_graph::build_resolved_exports(&self.module_table)` call
4. **Converted result** from `FxHashMap<ModuleIdx, FxHashMap<CompactString, ResolvedExport<SymbolRef>>>` to Rolldown's `FxHashMap<CompactStr, ResolvedExport>` format
5. **Removed** `add_exports_for_export_star()` method and unused `EcmaModuleAstUsage` import

### Files modified (rolldown repo)

- `crates/rolldown_common/src/oxc_module_graph_impls.rs` — extended `for_each_star_export` with IsCjsReexport
- `crates/rolldown/Cargo.toml` — added `oxc_module_graph` path dependency
- `crates/rolldown/src/stages/link_stage/bind_imports_and_exports.rs` — replaced Phase 1, removed dead code

### What stays in Rolldown
- Phase 2 import matching (CJS interop, external handling, dynamic fallback, shim missing exports)
- External import binding merger
- Ambiguity resolution filter
- `update_cjs_module_meta()`
- `resolve_member_expr_refs()`

### Test results
- 53 unit tests passed
- 834 fixture tests passed (58 ignored — pre-existing)
- 745 rollup compat tests passed (11 ignored — pre-existing)
- 47 JSX tests passed
- 1 test262 test passed
