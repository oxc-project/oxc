# Plan: Integrate `oxc_module_graph` into Rolldown (Full Replacement)

## Context

`oxc_module_graph` (in oxc repo, already implemented with 24 passing tests) provides trait-based cross-module analysis with a generic `bind_imports_and_exports` algorithm. Rolldown has its own ~1050-line implementation. We want to replace Rolldown's core binding logic with oxc_module_graph's shared algorithm.

**Challenge:** The current oxc_module_graph traits use concrete types that don't match Rolldown's types. We need to refactor the traits to use associated types so Rolldown can implement them directly.

**Rolldown's binding has 5 parts:**
1. **Core binding** (init exports → star re-exports → match imports → link symbols) — **replaceable** by oxc_module_graph
2. **CJS interop** (CommonJS detection, dynamic fallback, namespace aliases) — stays in Rolldown as `ImportMatcher` callbacks
3. **External import merging** (ESM facade symbols) — stays in Rolldown
4. **Member expression resolution** (foo_ns.bar_ns.c chains) — stays in Rolldown
5. **Ambiguity post-filtering** — stays in Rolldown

The shared algorithm replaces part (1); parts (2-5) remain as Rolldown-specific post-processing.

## Completed Stages

### Stage 1: Refactor oxc_module_graph traits to use associated types — Complete

- 17/17 tests passing
- Zero clippy warnings
- Formatting clean

### Stage 2: Implement traits in Rolldown — Complete

- `cargo check -p rolldown_common` passes
- `cargo check -p rolldown` passes (only pre-existing warnings)

### Stage 3: CJS-aware resolved exports + `build_resolved_exports` API — Complete

- 17/17 tests passing in oxc_module_graph
- Zero clippy warnings

### Stage 4: Replace Phase 1 in Rolldown's LinkStage — Complete

- All 1,680 Rolldown tests pass (0 failures)
- Replaced Phase 1 init + star re-export propagation with `oxc_module_graph::build_resolved_exports`
- Removed dead `add_exports_for_export_star()` method (~67 lines)

### Stage 5: Phase 2 — Re-export Chain Following & Import Matching in oxc — Complete

Added the generic re-export-chain-following import matcher to `oxc_module_graph`:

- 24/24 tests passing (7 new tests added)
- Zero clippy warnings
- Full workspace compiles

#### What was added to oxc_module_graph

1. **`NormalAndNamespace` variant** on `MatchImportKind` — for CJS interop / dynamic fallback (`namespace_ref.alias`)

2. **`ImportMatcher` trait** (`traits/import_matcher.rs`) — callback trait with 5 hooks:
   - `on_missing_module()` — handle external/missing modules
   - `on_before_match()` — short-circuit for CJS modules
   - `on_no_match()` — dynamic fallback when export not found
   - `on_cjs_match()` — override CJS-originated exports
   - `on_resolved()` — track re-export chains for tree-shaking
   - Plus `DefaultImportMatcher` — no-op for pure ESM

3. **`symbol_import_info()` on `ModuleInfo`** — checks if a symbol is a named import (needed for re-export chain following)

4. **`symbol_owner()` + `ModuleIdx` on `SymbolGraph`** — extracts owning module from a symbol ref (needed to look up the owner module during chain following)

5. **`match_imports()` algorithm** (`algo/binding.rs`) — full Phase 2:
   - `match_imports()` — main entry: iterates imports, handles namespace, delegates to `resolve_import`
   - `resolve_import()` — recursive chain follower with cycle detection + `ImportMatcher` callbacks
   - `resolve_ambiguous()` / `try_resolve_symbol()` — resolves potentially ambiguous exports by comparing recursive results

6. **7 new tests:**
   - Single re-export chain (A→B→C)
   - Deep 3-level chain (A→B→C→D)
   - Circular re-export detection
   - Custom `ImportMatcher` callback (CJS short-circuit)
   - Namespace imports
   - Unresolved imports
   - `on_resolved` chain tracking

#### Trait changes summary (breaking for Rolldown)

```rust
// SymbolGraph: added ModuleIdx associated type + symbol_owner method
pub trait SymbolGraph {
    type ModuleIdx: Copy + Eq + Hash + Debug;  // NEW
    type SymbolRef: Copy + Eq + Hash + Debug;
    fn canonical_ref_for(&self, symbol: Self::SymbolRef) -> Self::SymbolRef;
    fn link(&mut self, from: Self::SymbolRef, to: Self::SymbolRef);
    fn symbol_name(&self, symbol: Self::SymbolRef) -> &str;
    fn symbol_owner(&self, symbol: Self::SymbolRef) -> Self::ModuleIdx;  // NEW
}

// ModuleInfo: added symbol_import_info method
pub trait ModuleInfo {
    // ... existing methods ...
    fn symbol_import_info(&self, symbol: Self::SymbolRef)  // NEW
        -> Option<(&str, usize, bool)>;
}

// ImportMatcher: callbacks now receive importer context
pub trait ImportMatcher {
    fn on_missing_module(&mut self, importer_idx, record_idx, target_idx, name, is_ns);
    fn on_before_match(&mut self, importer_idx, record_idx, target_idx, name, is_ns);
    fn on_resolved(&mut self, importer_idx, local_symbol, resolved, chain);
    // on_no_match and on_cjs_match unchanged
}
```

## Stage 6: Implement `ImportMatcher` in Rolldown + Replace Phase 2

### Status: Complete

### Goal

Replace Rolldown's `match_imports_with_exports` + `match_import_with_export` + `advance_import_tracker` (~280 lines) with a single call to `oxc_module_graph::match_imports(store, symbols, resolved_exports, &mut matcher)`.

### Pre-requisites

Update Rolldown's existing trait impls to match the new trait signatures from Stage 5.

### 6a: Update Rolldown trait impls for new trait methods

**Files to modify (rolldown repo):**
- `crates/rolldown_common/src/oxc_module_graph_impls.rs`

**Changes:**
1. Add `type ModuleIdx = ModuleIdx;` to `SymbolGraph for SymbolRefDb`
2. Add `fn symbol_owner(&self, symbol: SymbolRef) -> ModuleIdx` — return `symbol.owner`
3. Add `fn symbol_import_info(&self, symbol: SymbolRef) -> Option<(&str, usize, bool)>` to `ModuleInfo for NormalModule`:
   - Look up `self.ecma_view.named_imports.get(&symbol)`
   - Return `(imported_name, record_idx, is_namespace)`

### 6b: Create `RolldownImportMatcher`

**Files to modify (rolldown repo):**
- `crates/rolldown/src/stages/link_stage/bind_imports_and_exports.rs`

Create a struct implementing `ImportMatcher`:

```rust
struct RolldownImportMatcher<'a> {
    index_modules: &'a IndexModules,
    metas: &'a mut LinkingMetadataVec,
    symbols: &'a SymbolRefDb,
    options: &'a SharedOptions,
    // Collect results that need post-processing
    normal_and_namespace_results: Vec<(SymbolRef, SymbolRef, CompactStr)>,  // local, ns, alias
}

impl ImportMatcher for RolldownImportMatcher<'_> {
    type ModuleIdx = ModuleIdx;
    type SymbolRef = SymbolRef;

    fn on_missing_module(&mut self, importer, record_idx, target, name, is_ns) -> Option<MatchImportKind> {
        // Module::External → namespace/normal based on output format
        // Use importer + record_idx to look up import_record.namespace_ref
        match &self.index_modules[target] {
            Module::External(ext) => {
                if is_ns {
                    Some(MatchImportKind::Namespace { namespace_ref: ext.namespace_ref })
                } else if self.options.format.keep_esm_import_export_syntax() {
                    // ESM output: create/use external symbol
                    Some(MatchImportKind::Normal { symbol_ref: ... })
                } else {
                    Some(MatchImportKind::NormalAndNamespace {
                        namespace_ref: ext.namespace_ref,
                        alias: name.into(),
                    })
                }
            }
            _ => None,
        }
    }

    fn on_before_match(&mut self, importer, record_idx, target, name, is_ns) -> Option<MatchImportKind> {
        // ExportsKind::CommonJs → NormalAndNamespace/Namespace
        // Use importer + record_idx to look up import_record.namespace_ref
        let module = self.index_modules[target].as_normal()?;
        if module.exports_kind == ExportsKind::CommonJs {
            let ns_ref = module.namespace_object_ref;
            Some(if is_ns {
                MatchImportKind::Namespace { namespace_ref: ns_ref }
            } else {
                MatchImportKind::NormalAndNamespace {
                    namespace_ref: ns_ref,
                    alias: name.into(),
                }
            })
        } else {
            None
        }
    }

    fn on_no_match(&mut self, target, name) -> Option<MatchImportKind> {
        // has_dynamic_exports → namespace fallback
        let meta = &self.metas[target];
        if meta.has_dynamic_exports {
            let module = self.index_modules[target].as_normal()?;
            Some(MatchImportKind::NormalAndNamespace {
                namespace_ref: module.namespace_object_ref,
                alias: name.into(),
            })
        } else {
            None
        }
    }

    fn on_cjs_match(&mut self, target, name, symbol) -> Option<MatchImportKind> {
        // DynamicFallbackWithCommonjsReference behavior
        let meta = &mut self.metas[target];
        meta.included_commonjs_export_symbol.insert(symbol);
        let module = self.index_modules[target].as_normal()?;
        Some(MatchImportKind::NormalAndNamespace {
            namespace_ref: module.namespace_object_ref,
            alias: name.into(),
        })
    }

    fn on_resolved(&mut self, local, resolved, chain) {
        // Record re-export chain for tree-shaking side-effect deps
        // Store NormalAndNamespace results for post-processing
    }
}
```

### 6c: Replace Phase 2 call site

**In `bind_imports_and_exports()`** (rolldown's `bind_imports_and_exports.rs`):

Replace:
```rust
self.match_imports_with_exports(&resolved_exports);
```

With:
```rust
let mut matcher = RolldownImportMatcher { ... };
let errors = oxc_module_graph::match_imports(
    &self.module_table,
    &mut self.symbols,
    &resolved_exports,  // convert from Rolldown format to oxc format
    &mut matcher,
);
// Post-process NormalAndNamespace results
// Handle errors (shim_missing_exports, diagnostics)
```

### 6d: Remove dead code

**Remove from Rolldown:**
- `match_imports_with_exports()` method
- `match_import_with_export()` method
- `advance_import_tracker()` method
- `ImportTracker` struct
- `MatchingContext` struct
- `ImportStatus` enum
- `MatchImportKindNormal` struct

### Estimated code changes
- **Remove from Rolldown:** ~280 lines
- **Add to Rolldown:** ~100 lines (`RolldownImportMatcher` impl + call site)
- **Net reduction:** ~180 lines

### Note on `resolved_exports` format conversion

Rolldown currently uses `FxHashMap<CompactStr, ResolvedExport>` per module (stored in `LinkingMetadata`), while `oxc_module_graph::match_imports` expects `FxHashMap<ModuleIdx, FxHashMap<CompactString, ResolvedExport<SymbolRef>>>`.

Options:
1. **Convert at call site** — map Rolldown's per-module `resolved_exports` into the oxc format before calling `match_imports`. Simple but allocates.
2. **Store oxc format directly** — change `LinkingMetadata` to store the oxc format from `build_resolved_exports`, avoid the Rolldown format entirely. Requires updating downstream consumers.
3. **Dual storage** — keep both formats temporarily. Wasteful but safe for incremental migration.

Recommend option (2) for clean integration, but (1) is fine for initial PR.

### Verification

```bash
# rolldown repo
cargo check -p rolldown           # Compiles
cargo test -p rolldown            # All tests pass
```

## Summary of all changes across both repos

### oxc repo (Stages 1, 3, 5)
| File | Change |
|------|--------|
| `traits/module_info.rs` | Associated types, `for_each_*` callbacks, `symbol_import_info` |
| `traits/module_store.rs` | Associated types, `for_each_*` callbacks |
| `traits/symbol_graph.rs` | Associated types, `symbol_owner` |
| `traits/import_matcher.rs` | **NEW** — `ImportMatcher` trait + `DefaultImportMatcher` |
| `traits/mod.rs` | Re-export `ImportMatcher`, `DefaultImportMatcher` |
| `types/import_export.rs` | `came_from_cjs` on `ResolvedExport`, `NormalAndNamespace` on `MatchImportKind` |
| `algo/binding.rs` | CJS-aware star re-exports, `build_resolved_exports`, `match_imports`, `resolve_import`, `resolve_ambiguous` |
| `algo/mod.rs` | Re-export `build_resolved_exports`, `match_imports` |
| `lib.rs` | Re-export all new public items |
| `default/module.rs` | `symbol_import_info` impl |
| `default/symbol_db.rs` | `symbol_owner` impl, `ModuleIdx` associated type |
| `tests/integration.rs` | 24 tests (7 new for match_imports) |

### rolldown repo (Stages 2, 4, 6)
| File | Change |
|------|--------|
| `rolldown_common/src/oxc_module_graph_impls.rs` | All trait impls: `ModuleInfo`, `ModuleStore`, `SymbolGraph` |
| `rolldown_common/Cargo.toml` | `oxc_module_graph` dependency |
| `rolldown/Cargo.toml` | `oxc_module_graph` dependency |
| `rolldown/src/stages/link_stage/bind_imports_and_exports.rs` | Phase 1 replaced (Stage 4), Phase 2 to be replaced (Stage 6) |
