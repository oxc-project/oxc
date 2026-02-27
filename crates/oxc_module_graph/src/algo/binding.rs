use compact_str::CompactString;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::traits::{ModuleInfo, ModuleStore, SymbolGraph};
use crate::types::{MatchImportKind, ModuleIdx, ResolvedExport, SymbolRef};

/// Resolved exports for each module: export_name → ResolvedExport.
type ResolvedExportsMap = FxHashMap<CompactString, ResolvedExport>;

/// Result of the binding phase.
pub struct BindingResult {
    /// Resolved exports per module.
    pub resolved_exports: FxHashMap<ModuleIdx, ResolvedExportsMap>,
    /// Errors: (module, import_name, kind).
    pub errors: Vec<BindingError>,
}

/// An error from the binding phase.
#[derive(Debug)]
pub enum BindingError {
    /// An import could not be matched to any export.
    UnresolvedImport { module: ModuleIdx, import_name: CompactString },
    /// An import is ambiguous (multiple `export *` provide the same name).
    AmbiguousImport { module: ModuleIdx, import_name: CompactString, candidates: Vec<SymbolRef> },
}

/// Resolve all imports to exports across the module graph.
///
/// This is the main cross-module linking algorithm:
/// 1. Build resolved exports from local exports
/// 2. Propagate star re-exports (merge, detect ambiguity)
/// 3. Match each import to target's resolved exports
/// 4. Link symbols via `SymbolGraph::link()`
pub fn bind_imports_and_exports<S, M>(store: &M, symbols: &mut S) -> Vec<BindingError>
where
    S: SymbolGraph,
    M: ModuleStore,
{
    let module_count = store.modules_len();
    if module_count == 0 {
        return Vec::new();
    }

    // Phase 1: Initialize resolved exports from local exports.
    let mut resolved_exports: Vec<ResolvedExportsMap> = Vec::with_capacity(module_count);

    for i in 0..module_count {
        let idx = ModuleIdx::from_usize(i);
        let module = store.module(idx);
        let mut exports = FxHashMap::default();

        for (name, local_export) in module.named_exports() {
            exports.insert(
                name.clone(),
                ResolvedExport {
                    symbol_ref: local_export.local_symbol,
                    potentially_ambiguous: None,
                },
            );
        }

        resolved_exports.push(exports);
    }

    // Phase 2: Propagate star re-exports.
    // For each module with star exports, merge in the exports from target modules.
    // Use DFS with cycle detection.
    for i in 0..module_count {
        let idx = ModuleIdx::from_usize(i);
        let mut visited = FxHashSet::default();
        add_exports_for_star(store, idx, &mut resolved_exports, &mut visited);
    }

    // Phase 3: Match imports to resolved exports and link symbols.
    let mut errors = Vec::new();

    for i in 0..module_count {
        let idx = ModuleIdx::from_usize(i);
        let module = store.module(idx);

        // Collect imports to process (avoid borrowing store while mutating symbols).
        let imports: Vec<_> = module
            .named_imports()
            .values()
            .map(|ni| (ni.local_symbol, ni.imported_name.clone(), ni.record_idx, ni.is_type))
            .collect();

        let import_records: Vec<_> =
            module.import_records().iter().map(|r| r.resolved_module).collect();

        for (local_symbol, imported_name, record_idx, _is_type) in imports {
            // Find the target module from the import record.
            let record_idx_usize = record_idx.index();
            let target_module = if record_idx_usize < import_records.len() {
                import_records[record_idx_usize]
            } else {
                None
            };

            let Some(target_idx) = target_module else {
                continue;
            };

            // Handle namespace imports.
            if imported_name.as_str() == "*" {
                let target = store.module(target_idx);
                let ns_ref = target.namespace_object_ref();
                symbols.link(local_symbol, ns_ref);
                continue;
            }

            // Match the import against resolved exports of the target.
            let match_result = match_import(
                target_idx,
                &imported_name,
                &resolved_exports,
                &mut FxHashSet::default(),
            );

            match match_result {
                MatchImportKind::Normal { symbol_ref } => {
                    symbols.link(local_symbol, symbol_ref);
                }
                MatchImportKind::Namespace { namespace_ref } => {
                    symbols.link(local_symbol, namespace_ref);
                }
                MatchImportKind::Ambiguous { candidates } => {
                    errors.push(BindingError::AmbiguousImport {
                        module: idx,
                        import_name: imported_name.clone(),
                        candidates,
                    });
                }
                MatchImportKind::Cycle => {
                    // Circular dependency — leave the symbol unlinked.
                }
                MatchImportKind::NoMatch => {
                    errors.push(BindingError::UnresolvedImport {
                        module: idx,
                        import_name: imported_name.clone(),
                    });
                }
            }
        }
    }

    errors
}

/// Recursively add exports from star re-exports into the resolved exports map.
fn add_exports_for_star<M: ModuleStore>(
    store: &M,
    module_idx: ModuleIdx,
    resolved_exports: &mut [ResolvedExportsMap],
    visited: &mut FxHashSet<ModuleIdx>,
) {
    if !visited.insert(module_idx) {
        // Cycle detected — stop recursion.
        return;
    }

    let module = store.module(module_idx);
    let star_entries: Vec<_> =
        module.star_export_entries().iter().filter_map(|entry| entry.resolved_module).collect();

    // Also collect indirect exports that reference other modules.
    let indirect_entries: Vec<_> = module
        .indirect_export_entries()
        .iter()
        .filter_map(|entry| {
            let target = entry.resolved_module?;
            Some((entry.exported_name.clone(), entry.imported_name.clone(), target))
        })
        .collect();

    // Process indirect re-exports first: `export { x } from './foo'`
    for (exported_name, imported_name, target_idx) in indirect_entries {
        // Recursively ensure target's star exports are resolved.
        add_exports_for_star(store, target_idx, resolved_exports, visited);

        let target_exports = &resolved_exports[target_idx.index()];
        if let Some(resolved) = target_exports.get(&imported_name) {
            let resolved_clone = resolved.clone();
            let my_exports = &mut resolved_exports[module_idx.index()];
            // Don't shadow explicit local exports.
            my_exports.entry(exported_name).or_insert(resolved_clone);
        }
    }

    // Process star re-exports: `export * from './foo'`
    for target_idx in star_entries {
        // Recursively ensure target's star exports are resolved.
        add_exports_for_star(store, target_idx, resolved_exports, visited);

        // Merge target's exports into this module.
        // Skip "default" — star re-exports never include default.
        let target_exports: Vec<_> = resolved_exports[target_idx.index()]
            .iter()
            .filter(|(name, _)| name.as_str() != "default")
            .map(|(name, export)| (name.clone(), export.clone()))
            .collect();

        let my_exports = &mut resolved_exports[module_idx.index()];
        for (name, export) in target_exports {
            match my_exports.get(&name) {
                Some(existing) if existing.symbol_ref == export.symbol_ref => {
                    // Same symbol — no conflict.
                }
                Some(existing) => {
                    // Different symbol — mark as potentially ambiguous.
                    let mut candidates = existing.potentially_ambiguous.clone().unwrap_or_default();
                    candidates.push(export.symbol_ref);
                    my_exports.insert(
                        name,
                        ResolvedExport {
                            symbol_ref: existing.symbol_ref,
                            potentially_ambiguous: Some(candidates),
                        },
                    );
                }
                None => {
                    my_exports.insert(name, export);
                }
            }
        }
    }

    visited.remove(&module_idx);
}

/// Match an import name against a module's resolved exports.
fn match_import(
    target_idx: ModuleIdx,
    import_name: &str,
    resolved_exports: &[ResolvedExportsMap],
    cycle_detector: &mut FxHashSet<(ModuleIdx, CompactString)>,
) -> MatchImportKind {
    let key = (target_idx, CompactString::from(import_name));
    if !cycle_detector.insert(key) {
        return MatchImportKind::Cycle;
    }

    let target_exports = &resolved_exports[target_idx.index()];

    match target_exports.get(import_name) {
        Some(resolved) => {
            if let Some(candidates) = &resolved.potentially_ambiguous {
                MatchImportKind::Ambiguous {
                    candidates: std::iter::once(resolved.symbol_ref)
                        .chain(candidates.iter().copied())
                        .collect(),
                }
            } else {
                MatchImportKind::Normal { symbol_ref: resolved.symbol_ref }
            }
        }
        None => MatchImportKind::NoMatch,
    }
}
