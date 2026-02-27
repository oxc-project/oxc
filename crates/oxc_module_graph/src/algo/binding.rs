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

/// Resolve all imports to exports across the module graph.
///
/// This is the main cross-module linking algorithm:
/// 1. Build resolved exports from local exports
/// 2. Propagate star re-exports (merge, detect ambiguity)
/// 3. Match each import to target's resolved exports
/// 4. Link symbols via `SymbolGraph::link()`
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
    // Collect all module indices first.
    let mut module_indices: Vec<M::ModuleIdx> = Vec::with_capacity(store.modules_len());
    store.for_each_module(&mut |idx, _| {
        module_indices.push(idx);
    });

    if module_indices.is_empty() {
        return BindingResult { resolved_exports: FxHashMap::default(), errors: Vec::new() };
    }

    // Phase 1: Initialize resolved exports from local exports.
    let mut resolved_exports: FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>> =
        FxHashMap::default();

    for &idx in &module_indices {
        let Some(module) = store.module(idx) else { continue };
        let mut exports: ResolvedExportsMap<M::SymbolRef> = FxHashMap::default();

        module.for_each_named_export(&mut |name, symbol_ref| {
            exports.insert(
                CompactString::from(name),
                crate::types::ResolvedExport { symbol_ref, potentially_ambiguous: None },
            );
        });

        resolved_exports.insert(idx, exports);
    }

    // Phase 2: Propagate star re-exports.
    for &idx in &module_indices {
        let mut visited = FxHashSet::default();
        add_exports_for_star(store, idx, &mut resolved_exports, &mut visited);
    }

    // Phase 3: Match imports to resolved exports and link symbols.
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
fn add_exports_for_star<M: ModuleStore>(
    store: &M,
    module_idx: M::ModuleIdx,
    resolved_exports: &mut FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>>,
    visited: &mut FxHashSet<M::ModuleIdx>,
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
        add_exports_for_star(store, target_idx, resolved_exports, visited);

        if let Some(target_exports) = resolved_exports.get(&target_idx)
            && let Some(resolved) = target_exports.get(&imported_name)
        {
            let resolved_clone = resolved.clone();
            if let Some(my_exports) = resolved_exports.get_mut(&module_idx) {
                my_exports.entry(exported_name).or_insert(resolved_clone);
            }
        }
    }

    // Process star re-exports: `export * from './foo'`
    for target_idx in star_targets {
        add_exports_for_star(store, target_idx, resolved_exports, visited);

        // Merge target's exports into this module.
        // Skip "default" — star re-exports never include default.
        let target_entries: Vec<(CompactString, crate::types::ResolvedExport<M::SymbolRef>)> =
            resolved_exports
                .get(&target_idx)
                .map(|exports| {
                    exports
                        .iter()
                        .filter(|(name, _)| name.as_str() != "default")
                        .map(|(name, export)| (name.clone(), export.clone()))
                        .collect()
                })
                .unwrap_or_default();

        if let Some(my_exports) = resolved_exports.get_mut(&module_idx) {
            for (name, export) in target_entries {
                match my_exports.get(&name) {
                    Some(existing) if existing.symbol_ref == export.symbol_ref => {
                        // Same symbol — no conflict.
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
