use std::fmt::Debug;
use std::hash::Hash;

use compact_str::CompactString;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::traits::{ModuleInfo, ModuleStore, SymbolGraph};
use crate::types::MatchImportKind;

/// A resolved export: the final symbol that an export name maps to.
/// Generic over the symbol reference type.
pub type ResolvedExportsMap<S> = FxHashMap<CompactString, crate::types::ResolvedExport<S>>;

/// Result of the binding phase.
pub struct BindingResult<Idx: Copy + Eq + Hash + Debug, Sym: Copy + Eq + Hash + Debug> {
    /// Resolved exports per module, keyed by module index.
    pub resolved_exports: FxHashMap<Idx, ResolvedExportsMap<Sym>>,
    /// Errors encountered during binding.
    pub errors: Vec<BindingError<Idx>>,
}

/// An error from the binding phase.
#[derive(Debug)]
pub enum BindingError<Idx: Debug> {
    /// An import could not be matched to any export.
    UnresolvedImport { module: Idx, import_name: CompactString },
    /// An import is ambiguous (multiple `export *` provide the same name).
    AmbiguousImport { module: Idx, import_name: CompactString },
}

/// Build resolved exports for all modules in the graph.
///
/// This is Phase 1 of the binding algorithm:
/// 1. Initialize resolved exports from each module's local exports
/// 2. Propagate star re-exports (merge, detect ambiguity, respect CJS semantics)
///
/// Local exports always shadow star re-exports without recording ambiguity,
/// matching the ESM specification where `export *` only applies to names
/// that are not locally exported.
///
/// Returns a map from module index to its resolved exports.
///
/// This function can be used standalone by consumers (like Rolldown) that need
/// the resolved exports but have their own import-matching logic.
pub fn build_resolved_exports<M: ModuleStore>(
    store: &M,
) -> FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>> {
    // Collect all module indices first.
    let mut module_indices: Vec<M::ModuleIdx> = Vec::with_capacity(store.modules_len());
    store.for_each_module(&mut |idx, _| {
        module_indices.push(idx);
    });

    if module_indices.is_empty() {
        return FxHashMap::default();
    }

    // Phase 1a: Initialize resolved exports from local exports.
    let mut resolved_exports: FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>> =
        FxHashMap::default();

    // Track which export names are local (not from star re-exports).
    // Local exports always shadow star re-exports without ambiguity.
    let mut local_export_names: FxHashMap<M::ModuleIdx, FxHashSet<CompactString>> =
        FxHashMap::default();

    for &idx in &module_indices {
        let Some(module) = store.module(idx) else { continue };
        let mut exports: ResolvedExportsMap<M::SymbolRef> = FxHashMap::default();
        let mut local_names: FxHashSet<CompactString> = FxHashSet::default();

        module.for_each_named_export(&mut |name, symbol_ref, came_from_cjs| {
            let key = CompactString::from(name);
            local_names.insert(key.clone());
            exports.insert(
                key,
                crate::types::ResolvedExport {
                    symbol_ref,
                    potentially_ambiguous: None,
                    came_from_cjs,
                },
            );
        });

        resolved_exports.insert(idx, exports);
        local_export_names.insert(idx, local_names);
    }

    // Phase 1b: Propagate star re-exports.
    for &idx in &module_indices {
        let mut visited = FxHashSet::default();
        add_exports_for_star(store, idx, &mut resolved_exports, &mut visited, &local_export_names);
    }

    resolved_exports
}

/// Resolve all imports to exports across the module graph.
///
/// This is the main cross-module linking algorithm:
/// 1. Build resolved exports from local exports (via `build_resolved_exports`)
/// 2. Match each import to target's resolved exports
/// 3. Link symbols via `SymbolGraph::link()`
///
/// Returns a `BindingResult` containing the resolved exports per module
/// and any binding errors.
pub fn bind_imports_and_exports<M, S>(
    store: &M,
    symbols: &mut S,
) -> BindingResult<M::ModuleIdx, M::SymbolRef>
where
    M: ModuleStore,
    S: SymbolGraph<SymbolRef = M::SymbolRef>,
{
    let resolved_exports = build_resolved_exports(store);

    if resolved_exports.is_empty() {
        return BindingResult { resolved_exports, errors: Vec::new() };
    }

    // Collect all module indices.
    let mut module_indices: Vec<M::ModuleIdx> = Vec::with_capacity(store.modules_len());
    store.for_each_module(&mut |idx, _| {
        module_indices.push(idx);
    });

    // Phase 2: Match imports to resolved exports and link symbols.
    let mut errors = Vec::new();

    for &idx in &module_indices {
        let Some(module) = store.module(idx) else { continue };

        // Collect imports to process (avoid borrow issues).
        let mut imports: Vec<(M::SymbolRef, CompactString, usize, bool)> = Vec::new();
        module.for_each_named_import(&mut |local_symbol, imported_name, record_idx, is_ns| {
            imports.push((local_symbol, CompactString::from(imported_name), record_idx, is_ns));
        });

        for (local_symbol, imported_name, record_idx, is_ns) in imports {
            // Find the target module from the import record.
            let Some(module) = store.module(idx) else { continue };
            let target_module = module.import_record_resolved_module(record_idx);

            let Some(target_idx) = target_module else {
                continue;
            };

            // Handle namespace imports.
            if is_ns || imported_name.as_str() == "*" {
                if let Some(target) = store.module(target_idx) {
                    let ns_ref = target.namespace_object_ref();
                    symbols.link(local_symbol, ns_ref);
                }
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
                MatchImportKind::Ambiguous { .. } => {
                    errors.push(BindingError::AmbiguousImport {
                        module: idx,
                        import_name: imported_name.clone(),
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

    BindingResult { resolved_exports, errors }
}

/// Recursively add exports from star re-exports into the resolved exports map.
///
/// Local exports (tracked in `local_export_names`) always shadow star re-exports
/// without recording ambiguity, per ESM specification. Ambiguity is only recorded
/// between different star re-export sources.
fn add_exports_for_star<M: ModuleStore>(
    store: &M,
    module_idx: M::ModuleIdx,
    resolved_exports: &mut FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>>,
    visited: &mut FxHashSet<M::ModuleIdx>,
    local_export_names: &FxHashMap<M::ModuleIdx, FxHashSet<CompactString>>,
) {
    if !visited.insert(module_idx) {
        return;
    }

    let Some(module) = store.module(module_idx) else {
        visited.remove(&module_idx);
        return;
    };

    // Collect star export targets.
    let mut star_targets: Vec<M::ModuleIdx> = Vec::new();
    module.for_each_star_export(&mut |target| {
        star_targets.push(target);
    });

    // Collect indirect exports.
    let mut indirect_entries: Vec<(CompactString, CompactString, M::ModuleIdx)> = Vec::new();
    module.for_each_indirect_export(&mut |exported_name, imported_name, target| {
        indirect_entries.push((
            CompactString::from(exported_name),
            CompactString::from(imported_name),
            target,
        ));
    });

    // Process indirect re-exports first: `export { x } from './foo'`
    for (exported_name, imported_name, target_idx) in indirect_entries {
        add_exports_for_star(store, target_idx, resolved_exports, visited, local_export_names);

        if let Some(target_exports) = resolved_exports.get(&target_idx)
            && let Some(resolved) = target_exports.get(&imported_name)
        {
            let resolved_clone = resolved.clone();
            if let Some(my_exports) = resolved_exports.get_mut(&module_idx) {
                my_exports.entry(exported_name).or_insert(resolved_clone);
            }
        }
    }

    // Get the local export names for this module (for shadowing check).
    let my_locals = local_export_names.get(&module_idx);

    // Process star re-exports: `export * from './foo'`
    for target_idx in star_targets {
        add_exports_for_star(store, target_idx, resolved_exports, visited, local_export_names);

        // Merge target's exports into this module.
        // Skip "default" unless it came from CJS — ESM star re-exports never include default,
        // but CJS "default" exports are propagated through `export *`.
        let target_entries: Vec<(CompactString, crate::types::ResolvedExport<M::SymbolRef>)> =
            resolved_exports
                .get(&target_idx)
                .map(|exports| {
                    exports
                        .iter()
                        .filter(|(name, export)| {
                            name.as_str() != "default" || export.came_from_cjs
                        })
                        .map(|(name, export)| (name.clone(), export.clone()))
                        .collect()
                })
                .unwrap_or_default();

        if let Some(my_exports) = resolved_exports.get_mut(&module_idx) {
            for (name, export) in target_entries {
                // Local exports always shadow star re-exports without ambiguity.
                if my_locals.is_some_and(|s| s.contains(&name)) {
                    continue;
                }

                match my_exports.get(&name) {
                    Some(existing) if existing.symbol_ref == export.symbol_ref => {
                        // Same symbol — no conflict.
                    }
                    Some(existing) if existing.came_from_cjs => {
                        // Existing export came from CJS — suppress ambiguity,
                        // keep the existing export as-is.
                    }
                    Some(existing) => {
                        // Different symbol — mark as potentially ambiguous.
                        let mut candidates =
                            existing.potentially_ambiguous.clone().unwrap_or_default();
                        candidates.push(export.symbol_ref);
                        my_exports.insert(
                            name,
                            crate::types::ResolvedExport {
                                symbol_ref: existing.symbol_ref,
                                potentially_ambiguous: Some(candidates),
                                came_from_cjs: existing.came_from_cjs,
                            },
                        );
                    }
                    None => {
                        my_exports.insert(name, export);
                    }
                }
            }
        }
    }

    visited.remove(&module_idx);
}

/// Match an import name against a module's resolved exports.
fn match_import<Idx: Copy + Eq + Hash + Debug, Sym: Copy + Eq + Hash + Debug>(
    target_idx: Idx,
    import_name: &str,
    resolved_exports: &FxHashMap<Idx, ResolvedExportsMap<Sym>>,
    cycle_detector: &mut FxHashSet<(Idx, CompactString)>,
) -> MatchImportKind<Sym> {
    let key = (target_idx, CompactString::from(import_name));
    if !cycle_detector.insert(key) {
        return MatchImportKind::Cycle;
    }

    let Some(target_exports) = resolved_exports.get(&target_idx) else {
        return MatchImportKind::NoMatch;
    };

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
