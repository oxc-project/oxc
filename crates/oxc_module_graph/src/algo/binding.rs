use std::fmt::Debug;
use std::hash::Hash;

use compact_str::CompactString;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::traits::{ImportMatcher, ModuleInfo, ModuleStore, SymbolGraph};
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
    S: SymbolGraph<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
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
                MatchImportKind::NormalAndNamespace { namespace_ref, .. } => {
                    // In the basic binding algorithm, treat as namespace link.
                    // Consumers using `match_imports` with an `ImportMatcher`
                    // get full control over this variant.
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
                        .filter(|(name, export)| name.as_str() != "default" || export.came_from_cjs)
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

// ---------------------------------------------------------------------------
// Phase 2 with re-export chain following + ImportMatcher callbacks
// ---------------------------------------------------------------------------

/// Match all imports to resolved exports, following re-export chains.
///
/// This is the full Phase 2 algorithm that replaces the simple lookup in
/// `bind_imports_and_exports`. It:
///
/// 1. Iterates all modules' named imports
/// 2. For namespace imports: links to the target's `namespace_object_ref`
/// 3. For named imports: follows re-export chains (if a resolved symbol is
///    itself an import in its owning module, recurses)
/// 4. Calls [`ImportMatcher`] callbacks for consumer-specific behavior
///    (CJS interop, external modules, dynamic fallback)
/// 5. Links symbols via `SymbolGraph::link()`
///
/// Returns binding errors (unresolved/ambiguous imports).
#[expect(clippy::implicit_hasher)]
pub fn match_imports<M, S, C>(
    store: &M,
    symbols: &mut S,
    resolved_exports: &FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>>,
    matcher: &mut C,
) -> Vec<BindingError<M::ModuleIdx>>
where
    M: ModuleStore,
    S: SymbolGraph<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
    C: ImportMatcher<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
{
    let mut errors = Vec::new();

    // Collect all module indices first.
    let mut module_indices: Vec<M::ModuleIdx> = Vec::with_capacity(store.modules_len());
    store.for_each_module(&mut |idx, _| {
        module_indices.push(idx);
    });

    for &idx in &module_indices {
        let Some(module) = store.module(idx) else { continue };

        // Collect imports to process (avoid borrow issues with callbacks).
        let mut imports: Vec<(M::SymbolRef, CompactString, usize, bool)> = Vec::new();
        module.for_each_named_import(&mut |local_symbol, imported_name, record_idx, is_ns| {
            imports.push((local_symbol, CompactString::from(imported_name), record_idx, is_ns));
        });

        for (local_symbol, imported_name, record_idx, is_ns) in imports {
            let Some(module) = store.module(idx) else { continue };
            let target_idx = module.import_record_resolved_module(record_idx);

            let Some(target_idx) = target_idx else {
                continue;
            };

            // Handle namespace imports.
            if is_ns || imported_name.as_str() == "*" {
                // Allow consumer to override (e.g., CJS uses importer_record.namespace_ref).
                if let Some(kind) =
                    matcher.on_before_match(idx, record_idx, target_idx, &imported_name, true)
                {
                    apply_match(symbols, local_symbol, &kind);
                    matcher.on_resolved(idx, local_symbol, &kind, &[]);
                    continue;
                }

                match store.module(target_idx) {
                    Some(target) => {
                        let ns_ref = target.namespace_object_ref();
                        symbols.link(local_symbol, ns_ref);
                    }
                    None => {
                        if let Some(kind) = matcher.on_missing_module(
                            idx,
                            record_idx,
                            target_idx,
                            &imported_name,
                            true,
                        ) {
                            apply_match(symbols, local_symbol, &kind);
                        }
                    }
                }
                continue;
            }

            // Resolve with re-export chain following.
            let mut visited = FxHashSet::default();
            let mut reexport_chain = Vec::new();
            let result = resolve_import(
                store,
                symbols,
                idx,
                record_idx,
                target_idx,
                &imported_name,
                resolved_exports,
                matcher,
                &mut visited,
                &mut reexport_chain,
            );

            matcher.on_resolved(idx, local_symbol, &result, &reexport_chain);

            match &result {
                MatchImportKind::Normal { symbol_ref } => {
                    symbols.link(local_symbol, *symbol_ref);
                }
                MatchImportKind::Namespace { namespace_ref } => {
                    symbols.link(local_symbol, *namespace_ref);
                }
                // NormalAndNamespace: consumer handles via on_resolved (e.g., namespace_alias).
                // Cycle: circular dependency — leave the symbol unlinked.
                MatchImportKind::NormalAndNamespace { .. } | MatchImportKind::Cycle => {}
                MatchImportKind::Ambiguous { .. } => {
                    errors.push(BindingError::AmbiguousImport {
                        module: idx,
                        import_name: imported_name.clone(),
                    });
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

/// Apply a `MatchImportKind` result by linking the local symbol.
fn apply_match<S: SymbolGraph>(
    symbols: &mut S,
    local_symbol: S::SymbolRef,
    kind: &MatchImportKind<S::SymbolRef>,
) {
    match kind {
        MatchImportKind::Normal { symbol_ref } => {
            symbols.link(local_symbol, *symbol_ref);
        }
        MatchImportKind::Namespace { namespace_ref }
        | MatchImportKind::NormalAndNamespace { namespace_ref, .. } => {
            symbols.link(local_symbol, *namespace_ref);
        }
        MatchImportKind::Ambiguous { .. } | MatchImportKind::Cycle | MatchImportKind::NoMatch => {}
    }
}

/// Recursively resolve an import name against a target module's resolved exports,
/// following re-export chains.
///
/// `importer_idx` and `importer_record_idx` identify which import record triggered
/// this lookup. These are passed to `ImportMatcher` callbacks so consumers can look
/// up per-record data (e.g., Rolldown's `import_record.namespace_ref`).
///
/// If a resolved export symbol is itself a named import in its owning module,
/// this function recurses into that module's import target — this is the
/// "re-export chain following" that the simple `match_import` does not do.
#[expect(clippy::too_many_arguments)]
fn resolve_import<M, S, C>(
    store: &M,
    symbols: &S,
    importer_idx: M::ModuleIdx,
    importer_record_idx: usize,
    target_idx: M::ModuleIdx,
    import_name: &str,
    resolved_exports: &FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>>,
    matcher: &mut C,
    visited: &mut FxHashSet<(M::ModuleIdx, CompactString)>,
    reexport_chain: &mut Vec<M::SymbolRef>,
) -> MatchImportKind<M::SymbolRef>
where
    M: ModuleStore,
    S: SymbolGraph<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
    C: ImportMatcher<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
{
    // Cycle detection.
    let key = (target_idx, CompactString::from(import_name));
    if !visited.insert(key) {
        return MatchImportKind::Cycle;
    }

    // Check if target module exists.
    if store.module(target_idx).is_none() {
        return matcher
            .on_missing_module(importer_idx, importer_record_idx, target_idx, import_name, false)
            .unwrap_or(MatchImportKind::NoMatch);
    }

    // Allow short-circuit (CJS, external, etc.).
    if let Some(kind) =
        matcher.on_before_match(importer_idx, importer_record_idx, target_idx, import_name, false)
    {
        return kind;
    }

    // Look up in resolved exports.
    let Some(target_exports) = resolved_exports.get(&target_idx) else {
        return matcher.on_no_match(target_idx, import_name).unwrap_or(MatchImportKind::NoMatch);
    };

    match target_exports.get(import_name) {
        Some(resolved) => {
            // Handle ambiguous exports by recursively resolving each candidate.
            if let Some(candidates) = &resolved.potentially_ambiguous {
                return resolve_ambiguous(
                    store,
                    symbols,
                    resolved.symbol_ref,
                    candidates,
                    resolved_exports,
                    matcher,
                    visited,
                    reexport_chain,
                );
            }

            // Handle came_from_cjs.
            if resolved.came_from_cjs
                && let Some(kind) =
                    matcher.on_cjs_match(target_idx, import_name, resolved.symbol_ref)
            {
                return kind;
            }

            let symbol = resolved.symbol_ref;

            // Check if the resolved symbol is itself an import (re-export chain).
            let owner_idx = symbols.symbol_owner(symbol);
            if let Some(owner_module) = store.module(owner_idx)
                && let Some((re_imported_name, re_record_idx, re_is_ns)) =
                    owner_module.symbol_import_info(symbol)
            {
                // Record this symbol in the re-export chain.
                reexport_chain.push(symbol);

                if re_is_ns {
                    // The re-export resolves to a namespace import — stop here.
                    if let Some(re_target) =
                        owner_module.import_record_resolved_module(re_record_idx)
                    {
                        if let Some(ns_module) = store.module(re_target) {
                            return MatchImportKind::Namespace {
                                namespace_ref: ns_module.namespace_object_ref(),
                            };
                        }
                        // Target not in store (external) — delegate to consumer.
                        return matcher
                            .on_missing_module(
                                owner_idx,
                                re_record_idx,
                                re_target,
                                import_name,
                                true,
                            )
                            .unwrap_or(MatchImportKind::NoMatch);
                    }
                    return MatchImportKind::NoMatch;
                }

                // Recurse into the re-exported import's target.
                // The importer is now the owner module (re-exporter),
                // and the record_idx is the re-export's import record.
                if let Some(re_target) = owner_module.import_record_resolved_module(re_record_idx) {
                    // If the next target is external/missing (not in the store),
                    // stop the chain here and return the current symbol as-is.
                    // The intermediate symbol's own import from the external will
                    // be handled when match_imports processes that module's imports,
                    // setting up namespace_alias etc. correctly.
                    if store.module(re_target).is_none() {
                        return MatchImportKind::Normal { symbol_ref: symbol };
                    }
                    return resolve_import(
                        store,
                        symbols,
                        owner_idx,
                        re_record_idx,
                        re_target,
                        re_imported_name,
                        resolved_exports,
                        matcher,
                        visited,
                        reexport_chain,
                    );
                }
            }

            MatchImportKind::Normal { symbol_ref: symbol }
        }
        None => {
            // Not found — ask matcher for dynamic fallback.
            matcher.on_no_match(target_idx, import_name).unwrap_or(MatchImportKind::NoMatch)
        }
    }
}

/// Resolve potentially ambiguous exports by recursively resolving each candidate
/// and comparing results.
///
/// If all candidates resolve to the same symbol, it's not truly ambiguous.
/// If they resolve to different symbols, return `Ambiguous`.
fn resolve_ambiguous<M, S, C>(
    store: &M,
    symbols: &S,
    primary: M::SymbolRef,
    candidates: &[M::SymbolRef],
    resolved_exports: &FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>>,
    matcher: &mut C,
    visited: &mut FxHashSet<(M::ModuleIdx, CompactString)>,
    reexport_chain: &mut Vec<M::SymbolRef>,
) -> MatchImportKind<M::SymbolRef>
where
    M: ModuleStore,
    S: SymbolGraph<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
    C: ImportMatcher<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
{
    // Try to resolve the primary candidate.
    let primary_result = try_resolve_symbol(
        store,
        symbols,
        primary,
        resolved_exports,
        matcher,
        visited,
        reexport_chain,
    );

    // Try to resolve each additional candidate.
    for &candidate in candidates {
        let candidate_result = try_resolve_symbol(
            store,
            symbols,
            candidate,
            resolved_exports,
            matcher,
            visited,
            reexport_chain,
        );

        // Compare: if both resolve to the same symbol, not truly ambiguous.
        let is_same = match (&primary_result, &candidate_result) {
            (
                MatchImportKind::Normal { symbol_ref: a },
                MatchImportKind::Normal { symbol_ref: b },
            )
            | (
                MatchImportKind::Namespace { namespace_ref: a },
                MatchImportKind::Namespace { namespace_ref: b },
            ) => a == b,
            // If one is Cycle or NoMatch, prefer the resolved one.
            (MatchImportKind::Cycle | MatchImportKind::NoMatch, _)
            | (_, MatchImportKind::Cycle | MatchImportKind::NoMatch) => true,
            // Different resolved symbols — truly ambiguous.
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
///
/// If the symbol is a named import in its owning module, follow the chain.
/// Otherwise, return it as a `Normal` match.
fn try_resolve_symbol<M, S, C>(
    store: &M,
    symbols: &S,
    symbol: M::SymbolRef,
    resolved_exports: &FxHashMap<M::ModuleIdx, ResolvedExportsMap<M::SymbolRef>>,
    matcher: &mut C,
    visited: &mut FxHashSet<(M::ModuleIdx, CompactString)>,
    reexport_chain: &mut Vec<M::SymbolRef>,
) -> MatchImportKind<M::SymbolRef>
where
    M: ModuleStore,
    S: SymbolGraph<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
    C: ImportMatcher<ModuleIdx = M::ModuleIdx, SymbolRef = M::SymbolRef>,
{
    let owner_idx = symbols.symbol_owner(symbol);
    if let Some(owner_module) = store.module(owner_idx)
        && let Some((re_imported_name, re_record_idx, re_is_ns)) =
            owner_module.symbol_import_info(symbol)
        && let Some(re_target) = owner_module.import_record_resolved_module(re_record_idx)
    {
        // Namespace import (`import * as foo; export { foo }`) — resolve to namespace object.
        if re_is_ns {
            if let Some(ns_module) = store.module(re_target) {
                return MatchImportKind::Namespace {
                    namespace_ref: ns_module.namespace_object_ref(),
                };
            }
            return MatchImportKind::NoMatch;
        }
        return resolve_import(
            store,
            symbols,
            owner_idx,
            re_record_idx,
            re_target,
            re_imported_name,
            resolved_exports,
            matcher,
            visited,
            reexport_chain,
        );
    }
    MatchImportKind::Normal { symbol_ref: symbol }
}
