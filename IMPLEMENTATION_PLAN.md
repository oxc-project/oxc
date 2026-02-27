# Plan: Integrate `oxc_module_graph` into Rolldown (Full Replacement)

## Context

`oxc_module_graph` (in oxc repo, already implemented with 17 passing tests) provides trait-based cross-module analysis with a generic `bind_imports_and_exports` algorithm. Rolldown has its own ~940-line implementation. We want to replace Rolldown's core binding logic with oxc_module_graph's shared algorithm.

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

## Stage 1: Refactor oxc_module_graph traits to use associated types [Complete]

### SymbolGraph

```rust
pub trait SymbolGraph {
    type SymbolRef: Copy + Eq + Hash + Debug;
    fn canonical_ref_for(&self, symbol: Self::SymbolRef) -> Self::SymbolRef;
    fn link(&mut self, from: Self::SymbolRef, to: Self::SymbolRef);
    fn symbol_name(&self, symbol: Self::SymbolRef) -> &str;
}
```

### ModuleInfo — uses callback-based iteration instead of concrete collection refs

```rust
pub trait ModuleInfo {
    type ModuleIdx: Copy + Eq + Hash + Debug;
    type SymbolRef: Copy + Eq + Hash + Debug;

    fn module_idx(&self) -> Self::ModuleIdx;
    fn default_export_ref(&self) -> Self::SymbolRef;
    fn namespace_object_ref(&self) -> Self::SymbolRef;
    fn has_module_syntax(&self) -> bool;

    fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef));
    fn for_each_named_import(&self, f: &mut dyn FnMut(Self::SymbolRef, &str, usize, bool));
    fn import_record_count(&self) -> usize;
    fn import_record_resolved_module(&self, idx: usize) -> Option<Self::ModuleIdx>;
    fn for_each_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx));
    fn for_each_indirect_export(&self, f: &mut dyn FnMut(&str, &str, Self::ModuleIdx));
}
```

### ModuleStore

```rust
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

### Binding algorithm returns BindingResult with associated types

```rust
pub struct ResolvedExport<S> {
    pub symbol_ref: S,
    pub potentially_ambiguous: Option<Vec<S>>,
}

pub struct BindingResult<Idx, Sym> {
    pub resolved_exports: FxHashMap<Idx, FxHashMap<CompactString, ResolvedExport<Sym>>>,
    pub errors: Vec<BindingError<Idx>>,
}

pub fn bind_imports_and_exports<M, S>(
    store: &M,
    symbols: &mut S,
) -> BindingResult<M::ModuleIdx, M::SymbolRef>
where
    M: ModuleStore,
    S: SymbolGraph<SymbolRef = M::SymbolRef>,
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

### Status: Complete
- 17/17 tests passing
- Zero clippy warnings
- Formatting clean

## Stage 2: Implement traits in Rolldown

**Status: Not Started**

### Files to create (rolldown repo)

- `crates/rolldown_common/src/oxc_module_graph_impls.rs` — all 3 trait impls

### Files to modify (rolldown repo)

- `crates/rolldown_common/Cargo.toml` — add path dep: `oxc_module_graph = { path = "../../oxc/crates/oxc_module_graph" }`
- `crates/rolldown_common/src/lib.rs` — register module

### Trait impls sketch

```rust
// SymbolGraph for SymbolRefDb — direct delegation
impl oxc_module_graph::SymbolGraph for SymbolRefDb {
    type SymbolRef = crate::SymbolRef;
    fn canonical_ref_for(&self, s: Self::SymbolRef) -> Self::SymbolRef { self.canonical_ref_for(s) }
    fn link(&mut self, from: Self::SymbolRef, to: Self::SymbolRef) { self.link(from, to); }
    fn symbol_name(&self, s: Self::SymbolRef) -> &str { s.name(self) }
}

// ModuleInfo for NormalModule — adapt field names
impl oxc_module_graph::ModuleInfo for NormalModule {
    type ModuleIdx = crate::ModuleIdx;
    type SymbolRef = crate::SymbolRef;
    fn module_idx(&self) -> Self::ModuleIdx { self.idx }
    fn namespace_object_ref(&self) -> Self::SymbolRef { self.ecma_view.namespace_object_ref }
    fn default_export_ref(&self) -> Self::SymbolRef { self.ecma_view.default_export_ref }
    fn has_module_syntax(&self) -> bool { self.ecma_view.def_format.is_esm() }
    fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef)) {
        for (name, export) in &self.ecma_view.named_exports {
            f(name.as_str(), export.referenced);
        }
    }
    fn for_each_named_import(&self, f: &mut dyn FnMut(Self::SymbolRef, &str, usize, bool)) {
        for (sym, import) in &self.ecma_view.named_imports {
            let (name, is_ns) = match &import.imported {
                Specifier::Star => ("*", true),
                Specifier::Literal(s) => (s.as_str(), false),
            };
            f(*sym, name, import.record_idx.index(), is_ns);
        }
    }
    fn import_record_count(&self) -> usize { self.ecma_view.import_records.len() }
    fn import_record_resolved_module(&self, idx: usize) -> Option<Self::ModuleIdx> {
        self.ecma_view.import_records.get(ImportRecordIdx::from_usize(idx))?.resolved_module
    }
    fn for_each_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx)) {
        for rec in self.ecma_view.import_records.iter() {
            if rec.meta.contains(ImportRecordMeta::IsExportStar) {
                if let Some(m) = rec.resolved_module { f(m); }
            }
        }
    }
    fn for_each_indirect_export(&self, _f: &mut dyn FnMut(&str, &str, Self::ModuleIdx)) {
        // Rolldown handles indirect exports differently (through named_exports + named_imports)
        // This may need adjustment based on how Rolldown represents `export { x } from './foo'`
    }
}

// ModuleStore for ModuleTable — skip External modules
impl oxc_module_graph::ModuleStore for ModuleTable {
    type ModuleIdx = crate::ModuleIdx;
    type SymbolRef = crate::SymbolRef;
    type Module = NormalModule;
    fn module(&self, idx: Self::ModuleIdx) -> Option<&NormalModule> {
        match &self.modules[idx] { Module::Normal(m) => Some(m), _ => None }
    }
    fn modules_len(&self) -> usize { self.modules.len() }
    fn for_each_module(&self, f: &mut dyn FnMut(Self::ModuleIdx, &NormalModule)) {
        for (idx, m) in self.modules.iter_enumerated() {
            if let Module::Normal(m) = m { f(idx, m); }
        }
    }
    fn for_each_dependency(&self, idx: Self::ModuleIdx, f: &mut dyn FnMut(Self::ModuleIdx)) {
        if let Module::Normal(m) = &self.modules[idx] {
            for rec in m.ecma_view.import_records.iter() {
                if let Some(target) = rec.resolved_module { f(target); }
            }
        }
    }
}
```

## Stage 3: Replace core binding in LinkStage

**Status: Not Started**

### File to modify
- `crates/rolldown/src/stages/link_stage/bind_imports_and_exports.rs`

### What changes

**Before** (current Rolldown, ~240 lines for core binding):
```rust
pub(super) fn bind_imports_and_exports(&mut self) {
    // 1. Initialize resolved_exports (parallel)     ← REPLACE
    // 2. Add exports for export stars               ← REPLACE
    // 3. Match imports with exports (BindContext)    ← REPLACE
    // 4. External import binding merger              ← KEEP (Rolldown-specific)
    // 5. Ambiguity resolution                        ← KEEP (Rolldown-specific)
    // 6. update_cjs_module_meta                      ← KEEP (Rolldown-specific)
    // 7. resolve_member_expr_refs                    ← KEEP (Rolldown-specific)
}
```

**After:**
```rust
pub(super) fn bind_imports_and_exports(&mut self) {
    // 1-3. Core binding via shared algorithm
    let result = oxc_module_graph::bind_imports_and_exports(
        &self.module_table,
        &mut self.symbols,
    );
    // Store resolved_exports into metas
    for (idx, exports) in result.resolved_exports.into_iter().enumerate() {
        self.metas[ModuleIdx::from_usize(idx)].resolved_exports = exports;
    }

    // 4. External import binding merger (Rolldown-specific)
    // ... existing code ...

    // 5. Ambiguity resolution (Rolldown-specific)
    // ... existing code ...

    // 6-7. CJS meta + member expr resolution
    self.update_cjs_module_meta();
    self.resolve_member_expr_refs(...);
}
```

### What gets removed from Rolldown
- `add_exports_for_export_star()` static method (~70 lines)
- `BindImportsAndExportsContext` struct and `match_imports_with_exports()` (~150 lines)
- `advance_import_tracker()` (~65 lines)
- `match_import_with_export()` (~120 lines)
- `ImportTracker`, `MatchingContext`, `MatchImportKind`, `ImportStatus` types

### What stays in Rolldown
- External import binding merger (~20 lines)
- Ambiguity resolution filter (~20 lines)
- `update_cjs_module_meta()` (~50 lines)
- `resolve_member_expr_refs()` (~200 lines)

## Verification

```bash
# In oxc repo — ensure refactored traits still pass
cargo test -p oxc_module_graph
cargo clippy -p oxc_module_graph

# In rolldown repo — ensure all existing tests pass
cargo test -p rolldown_common
cargo test -p rolldown
# Run the full test suite to validate no regressions
pnpm test
```
