use compact_str::CompactString;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::graph::ModuleGraph;
use crate::hooks::LinkConfig;
use crate::module::{Module, NormalModule};
use crate::types::{MatchImportKind, ModuleIdx, ResolvedExport, SymbolRef};

/// A resolved export: the final symbol that an export name maps to.
pub type ResolvedExportsMap = FxHashMap<CompactString, ResolvedExport>;

/// An error from the binding phase.
#[derive(Debug, Clone)]
pub enum BindingError {
    /// An import could not be matched to any export.
    UnresolvedImport { module: ModuleIdx, import_name: CompactString },
    /// An import is ambiguous (multiple `export *` provide the same name).
    AmbiguousImport { module: ModuleIdx, import_name: CompactString },
}

/// Build resolved exports for all normal modules in the graph.
///
/// Phase 1 of the binding algorithm:
/// 1. Initialize resolved exports from each module's local exports
/// 2. Propagate star re-exports (merge, detect ambiguity, respect CJS semantics)
///
/// Local exports always shadow star re-exports without recording ambiguity.
///
/// # Panics
/// Panics if a module index in the graph is not a normal module when expected.
pub fn build_resolved_exports(graph: &ModuleGraph) -> FxHashMap<ModuleIdx, ResolvedExportsMap> {
    let module_indices: Vec<ModuleIdx> =
        graph.modules.iter_enumerated().filter_map(|(idx, m)| m.as_normal().map(|_| idx)).collect();

    if module_indices.is_empty() {
        return FxHashMap::default();
    }

    // Phase 1a: Initialize resolved exports from local exports.
    let mut resolved_exports: FxHashMap<ModuleIdx, ResolvedExportsMap> = FxHashMap::default();
    let mut local_export_names: FxHashMap<ModuleIdx, FxHashSet<CompactString>> =
        FxHashMap::default();

    for &idx in &module_indices {
        let module = graph.normal_module(idx).unwrap();
        let mut exports: ResolvedExportsMap = FxHashMap::default();
        let mut local_names: FxHashSet<CompactString> = FxHashSet::default();

        for (name, local_export) in &module.named_exports {
            local_names.insert(name.clone());
            exports.insert(
                name.clone(),
                ResolvedExport {
                    symbol_ref: local_export.local_symbol,
                    potentially_ambiguous: None,
                    came_from_cjs: false,
                },
            );
        }

        resolved_exports.insert(idx, exports);
        local_export_names.insert(idx, local_names);
    }

    // Phase 1b: Propagate star re-exports.
    for &idx in &module_indices {
        let mut visited = FxHashSet::default();
        add_exports_for_star(graph, idx, &mut resolved_exports, &mut visited, &local_export_names);
    }

    resolved_exports
}

/// Match all imports to resolved exports, following re-export chains.
///
/// This is Phase 2 with built-in CJS interop.
/// Returns binding errors (unresolved/ambiguous imports).
///
/// # Panics
/// Panics if a module index in the graph is not a normal module when expected.
pub fn match_imports(graph: &ModuleGraph, config: &mut LinkConfig) -> Vec<BindingError> {
    let (errors, _links) = match_imports_collect(graph, config);
    errors
}

/// Collect link operations from matching all imports to resolved exports.
///
/// Returns `(errors, link_pairs)` where `link_pairs` contains `(from, to)` tuples
/// that the consumer should apply to its symbol database.
///
/// Rolldown uses this to interleave its own steps (e.g., `determine_module_exports_kind`,
/// `wrap_modules`) between algorithm calls and apply links to its own `SymbolRefDb`.
///
/// # Panics
/// Panics if a module index in the graph is not a normal module when expected.
pub fn match_imports_collect(
    graph: &ModuleGraph,
    config: &mut LinkConfig,
) -> (Vec<BindingError>, Vec<(SymbolRef, SymbolRef)>) {
    let mut errors = Vec::new();
    let mut links: Vec<(SymbolRef, SymbolRef)> = Vec::new();

    let module_indices: Vec<ModuleIdx> =
        graph.modules.iter_enumerated().filter_map(|(idx, m)| m.as_normal().map(|_| idx)).collect();

    for &idx in &module_indices {
        let module = graph.normal_module(idx).unwrap();

        let imports: Vec<(SymbolRef, CompactString, usize, bool)> = module
            .named_imports
            .values()
            .map(|ni| {
                let is_ns = ni.imported_name.as_str() == "*";
                (ni.local_symbol, ni.imported_name.clone(), ni.record_idx.index(), is_ns)
            })
            .collect();

        for (local_symbol, imported_name, record_idx, is_ns) in imports {
            let module = graph.normal_module(idx).unwrap();
            let target_idx = module.import_record_resolved_module(record_idx);

            let Some(target_idx) = target_idx else {
                continue;
            };

            // Handle namespace imports.
            if is_ns || imported_name.as_str() == "*" {
                let ns_ref = graph.module(target_idx).namespace_object_ref();
                let result = MatchImportKind::Namespace { namespace_ref: ns_ref };
                if let Some(hooks) = config.import_hooks.as_deref_mut() {
                    hooks.on_resolved(idx, local_symbol, &result, &[]);
                }
                links.push((local_symbol, ns_ref));
                continue;
            }

            // Built-in CJS interop: if target is CJS, use NormalAndNamespace.
            if config.cjs_interop
                && let Some(target) = graph.normal_module(target_idx)
                && target.is_commonjs()
            {
                let ns_ref = graph.module(target_idx).namespace_object_ref();
                let result = MatchImportKind::NormalAndNamespace {
                    namespace_ref: ns_ref,
                    alias: imported_name.clone(),
                };
                if let Some(hooks) = config.import_hooks.as_deref_mut() {
                    hooks.on_resolved(idx, local_symbol, &result, &[]);
                }
                links.push((local_symbol, ns_ref));
                continue;
            }

            // Resolve with re-export chain following.
            let mut visited = FxHashSet::default();
            let mut reexport_chain = Vec::new();
            let result = resolve_import(
                graph,
                idx,
                record_idx,
                target_idx,
                &imported_name,
                config,
                &mut visited,
                &mut reexport_chain,
            );

            // Notify hooks of the resolution result.
            if let Some(hooks) = config.import_hooks.as_deref_mut() {
                hooks.on_resolved(idx, local_symbol, &result, &reexport_chain);
            }

            match &result {
                MatchImportKind::Normal { symbol_ref } => {
                    links.push((local_symbol, *symbol_ref));
                }
                MatchImportKind::Namespace { namespace_ref }
                | MatchImportKind::NormalAndNamespace { namespace_ref, .. } => {
                    links.push((local_symbol, *namespace_ref));
                }
                MatchImportKind::Ambiguous { .. } => {
                    errors.push(BindingError::AmbiguousImport {
                        module: idx,
                        import_name: imported_name.clone(),
                    });
                }
                MatchImportKind::Cycle => {}
                MatchImportKind::NoMatch => {
                    // Give hooks a chance to override the no-match result.
                    if let Some(hooks) = config.import_hooks.as_deref_mut()
                        && let Some(override_result) =
                            hooks.on_final_no_match(target_idx, &imported_name)
                    {
                        match &override_result {
                            MatchImportKind::Normal { symbol_ref } => {
                                links.push((local_symbol, *symbol_ref));
                            }
                            MatchImportKind::Namespace { namespace_ref }
                            | MatchImportKind::NormalAndNamespace { namespace_ref, .. } => {
                                links.push((local_symbol, *namespace_ref));
                            }
                            _ => {}
                        }
                        continue;
                    }
                    errors.push(BindingError::UnresolvedImport {
                        module: idx,
                        import_name: imported_name.clone(),
                    });
                }
            }
        }
    }

    (errors, links)
}

/// Convenience function: build resolved exports then match imports.
pub fn bind_imports_and_exports(graph: &mut ModuleGraph) {
    let resolved = build_resolved_exports(graph);

    // Store resolved exports on each module.
    for (idx, exports) in resolved {
        if let Module::Normal(m) = &mut graph.modules[idx] {
            m.resolved_exports = exports;
        }
    }

    let mut config = LinkConfig::default();
    let (errors, links) = match_imports_collect(graph, &mut config);

    // Apply links.
    for (from, to) in links {
        graph.symbols.link(from, to);
    }

    graph.set_binding_errors(errors);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Recursively add exports from star re-exports into the resolved exports map.
fn add_exports_for_star(
    graph: &ModuleGraph,
    module_idx: ModuleIdx,
    resolved_exports: &mut FxHashMap<ModuleIdx, ResolvedExportsMap>,
    visited: &mut FxHashSet<ModuleIdx>,
    local_export_names: &FxHashMap<ModuleIdx, FxHashSet<CompactString>>,
) {
    if !visited.insert(module_idx) {
        return;
    }

    let Some(module) = graph.normal_module(module_idx) else {
        visited.remove(&module_idx);
        return;
    };

    // Collect star export targets.
    let star_targets: Vec<ModuleIdx> = module.star_export_modules().collect();

    // Collect indirect exports.
    let indirect_entries: Vec<(CompactString, CompactString, ModuleIdx)> = module
        .indirect_export_entries
        .iter()
        .filter_map(|entry| {
            entry
                .resolved_module
                .map(|target| (entry.exported_name.clone(), entry.imported_name.clone(), target))
        })
        .collect();

    // Process indirect re-exports first: `export { x } from './foo'`
    for (exported_name, imported_name, target_idx) in indirect_entries {
        add_exports_for_star(graph, target_idx, resolved_exports, visited, local_export_names);

        if let Some(target_exports) = resolved_exports.get(&target_idx)
            && let Some(resolved) = target_exports.get(&imported_name)
        {
            let mut resolved_clone = resolved.clone();
            if graph.normal_module(target_idx).is_some_and(NormalModule::is_commonjs)
                || resolved_clone.came_from_cjs
            {
                resolved_clone.came_from_cjs = true;
            }
            if let Some(my_exports) = resolved_exports.get_mut(&module_idx) {
                my_exports.entry(exported_name).or_insert(resolved_clone);
            }
        }
    }

    // Get the local export names for this module (for shadowing check).
    let my_locals = local_export_names.get(&module_idx);

    // Process star re-exports: `export * from './foo'`
    for target_idx in star_targets {
        add_exports_for_star(graph, target_idx, resolved_exports, visited, local_export_names);
        let target_is_commonjs =
            graph.normal_module(target_idx).is_some_and(NormalModule::is_commonjs);

        // Merge target's exports into this module.
        let target_entries: Vec<(CompactString, ResolvedExport)> = resolved_exports
            .get(&target_idx)
            .map(|exports| {
                exports
                    .iter()
                    .filter(|(name, export)| name.as_str() != "default" || export.came_from_cjs)
                    .map(|(name, export)| {
                        let mut export = export.clone();
                        if target_is_commonjs {
                            export.came_from_cjs = true;
                        }
                        (name.clone(), export)
                    })
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
                        // Existing from CJS — suppress ambiguity.
                    }
                    Some(existing) => {
                        // Different symbol — mark as potentially ambiguous.
                        let mut candidates =
                            existing.potentially_ambiguous.clone().unwrap_or_default();
                        candidates.push(export.symbol_ref);
                        my_exports.insert(
                            name,
                            ResolvedExport {
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

/// Recursively resolve an import name against a target module's resolved exports,
/// following re-export chains.
fn resolve_import(
    graph: &ModuleGraph,
    _importer_idx: ModuleIdx,
    _importer_record_idx: usize,
    target_idx: ModuleIdx,
    import_name: &str,
    config: &LinkConfig,
    visited: &mut FxHashSet<(ModuleIdx, CompactString)>,
    reexport_chain: &mut Vec<SymbolRef>,
) -> MatchImportKind {
    // Cycle detection.
    let key = (target_idx, CompactString::from(import_name));
    if !visited.insert(key) {
        return MatchImportKind::Cycle;
    }

    // Check if target module is external.
    if let Some(ext) = graph.external_module(target_idx) {
        // Built-in: external modules → NormalAndNamespace with namespace_ref.
        return MatchImportKind::NormalAndNamespace {
            namespace_ref: ext.namespace_ref,
            alias: CompactString::from(import_name),
        };
    }

    let Some(target) = graph.normal_module(target_idx) else {
        return MatchImportKind::NoMatch;
    };

    // Built-in CJS interop.
    if config.cjs_interop && target.is_commonjs() {
        return MatchImportKind::NormalAndNamespace {
            namespace_ref: target.namespace_object_ref,
            alias: CompactString::from(import_name),
        };
    }

    // Look up in resolved exports.
    if let Some(resolved) = target.resolved_exports.get(import_name) {
        // Handle ambiguous exports.
        if let Some(candidates) = &resolved.potentially_ambiguous {
            return resolve_ambiguous(
                graph,
                resolved.symbol_ref,
                candidates,
                config,
                visited,
                reexport_chain,
            );
        }

        let symbol = resolved.symbol_ref;

        // Check if the resolved symbol is itself an import (re-export chain).
        let owner_idx = symbol.owner;
        if let Some(owner_module) = graph.normal_module(owner_idx)
            && let Some((re_imported_name, re_record_idx, re_is_ns)) =
                owner_module.symbol_import_info(symbol)
        {
            // Record this symbol in the re-export chain.
            reexport_chain.push(symbol);

            if re_is_ns {
                // Resolves to a namespace import — stop here.
                if let Some(re_target) = owner_module.import_record_resolved_module(re_record_idx) {
                    return MatchImportKind::Namespace {
                        namespace_ref: graph.module(re_target).namespace_object_ref(),
                    };
                }
                return MatchImportKind::NoMatch;
            }

            // Recurse into the re-exported import's target.
            if let Some(re_target) = owner_module.import_record_resolved_module(re_record_idx) {
                // If the next target is external, stop here and return current symbol.
                if graph.external_module(re_target).is_some() {
                    return MatchImportKind::Normal { symbol_ref: symbol };
                }
                return resolve_import(
                    graph,
                    owner_idx,
                    re_record_idx,
                    re_target,
                    re_imported_name,
                    config,
                    visited,
                    reexport_chain,
                );
            }
        }

        MatchImportKind::Normal { symbol_ref: symbol }
    } else {
        // Built-in: has_dynamic_exports → NormalAndNamespace fallback.
        if target.has_dynamic_exports {
            return MatchImportKind::NormalAndNamespace {
                namespace_ref: target.namespace_object_ref,
                alias: CompactString::from(import_name),
            };
        }

        MatchImportKind::NoMatch
    }
}

/// Resolve potentially ambiguous exports.
fn resolve_ambiguous(
    graph: &ModuleGraph,
    primary: SymbolRef,
    candidates: &[SymbolRef],
    config: &LinkConfig,
    visited: &mut FxHashSet<(ModuleIdx, CompactString)>,
    reexport_chain: &mut Vec<SymbolRef>,
) -> MatchImportKind {
    let primary_result = try_resolve_symbol(graph, primary, config, visited, reexport_chain);

    for &candidate in candidates {
        let candidate_result =
            try_resolve_symbol(graph, candidate, config, visited, reexport_chain);

        let is_same = match (&primary_result, &candidate_result) {
            (
                MatchImportKind::Normal { symbol_ref: a },
                MatchImportKind::Normal { symbol_ref: b },
            )
            | (
                MatchImportKind::Namespace { namespace_ref: a },
                MatchImportKind::Namespace { namespace_ref: b },
            ) => a == b,
            (MatchImportKind::Cycle | MatchImportKind::NoMatch, _)
            | (_, MatchImportKind::Cycle | MatchImportKind::NoMatch) => true,
            _ => false,
        };
        if !is_same {
            let mut all = vec![primary];
            all.extend_from_slice(candidates);
            return MatchImportKind::Ambiguous { candidates: all };
        }
    }

    primary_result
}

/// Try to resolve a single symbol through re-export chains.
fn try_resolve_symbol(
    graph: &ModuleGraph,
    symbol: SymbolRef,
    config: &LinkConfig,
    visited: &mut FxHashSet<(ModuleIdx, CompactString)>,
    reexport_chain: &mut Vec<SymbolRef>,
) -> MatchImportKind {
    let owner_idx = symbol.owner;
    if let Some(owner_module) = graph.normal_module(owner_idx)
        && let Some((re_imported_name, re_record_idx, re_is_ns)) =
            owner_module.symbol_import_info(symbol)
        && let Some(re_target) = owner_module.import_record_resolved_module(re_record_idx)
    {
        if re_is_ns {
            return MatchImportKind::Namespace {
                namespace_ref: graph.module(re_target).namespace_object_ref(),
            };
        }
        return resolve_import(
            graph,
            owner_idx,
            re_record_idx,
            re_target,
            re_imported_name,
            config,
            visited,
            reexport_chain,
        );
    }
    MatchImportKind::Normal { symbol_ref: symbol }
}
