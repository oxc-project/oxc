use rustc_hash::FxHashMap;

use crate::graph::ModuleGraph;
use crate::types::{ExportsKind, ImportKind, ModuleIdx, SymbolRef, WrapKind};

/// Configuration for [`wrap_modules`].
pub struct WrapModulesConfig {
    /// Skip wrapping for pure ESM modules with no import records.
    /// (Rolldown's "on-demand wrapping" optimization.)
    pub on_demand_wrapping: bool,
    /// When true, enforce strict execution order: all CJS modules are wrapped,
    /// and ESM modules are wrapped unless on-demand conditions are met AND
    /// the module is not required by another module.
    pub strict_execution_order: bool,
    /// Skip wrapper symbol creation (Phase 2).
    ///
    /// When `true`, [`wrap_modules`] returns empty `wrapper_refs` and skips
    /// creating `require_*`/`init_*` symbols. Consumers like Rolldown set this
    /// when they create wrapper symbols via their own richer `SymbolRefDb`.
    pub skip_symbol_creation: bool,
}

/// Result of [`wrap_modules`].
pub struct WrapModulesResult {
    /// Finalized wrap_kind for modules (propagated + initial).
    pub wrap_kind_updates: FxHashMap<ModuleIdx, WrapKind>,
    /// Wrapper symbol refs created for each wrapped module.
    pub wrapper_refs: FxHashMap<ModuleIdx, SymbolRef>,
    /// Modules whose original_wrap_kind was set (before propagation).
    pub original_wrap_kinds: FxHashMap<ModuleIdx, WrapKind>,
    /// Modules that are imported via require() by another module.
    pub required_by_other_module: Vec<ModuleIdx>,
}

/// Propagate wrapping through the dependency graph and create wrapper symbols.
///
/// **Phase 1**: Propagate wrapping. For each module that already has a wrap_kind,
/// recursively wrap all its dependencies. CJS dependencies are always wrapped
/// even if the importer isn't wrapped.
///
/// **Phase 2**: Create wrapper symbols. For each wrapped module, create a
/// `require_{stem}` (CJS) or `init_{stem}` (ESM) symbol.
///
/// # Panics
///
/// Panics if a normal module index collected from the graph becomes invalid
/// (should not happen in normal usage).
pub fn wrap_modules(graph: &mut ModuleGraph, config: &WrapModulesConfig) -> WrapModulesResult {
    let module_count = graph.modules_len();
    let mut visited = vec![false; module_count];
    let mut wrap_updates: FxHashMap<ModuleIdx, WrapKind> = FxHashMap::default();
    let mut original_wrap_kinds: FxHashMap<ModuleIdx, WrapKind> = FxHashMap::default();
    let mut required_by_other: Vec<ModuleIdx> = Vec::new();

    // Collect current wrap_kind state for all normal modules.
    let normal_indices: Vec<ModuleIdx> =
        graph.modules.iter_enumerated().filter_map(|(idx, m)| m.as_normal().map(|_| idx)).collect();

    // When strict_execution_order is enabled, force wrapping for all modules.
    if config.strict_execution_order {
        for &idx in &normal_indices {
            // Runtime module should never be wrapped.
            if graph.runtime() == Some(idx) {
                continue;
            }
            let module = graph.normal_module(idx).unwrap();
            let current_wrap = module.wrap_kind;
            if current_wrap.is_none() {
                let new_wrap = match module.exports_kind {
                    ExportsKind::CommonJs => WrapKind::Cjs,
                    ExportsKind::Esm | ExportsKind::None => {
                        // With on-demand wrapping, skip if conditions are met.
                        if config.on_demand_wrapping
                            && !module.execution_order_sensitive
                            && module.import_records.is_empty()
                        {
                            continue;
                        }
                        WrapKind::Esm
                    }
                };
                wrap_updates.insert(idx, new_wrap);
                original_wrap_kinds.insert(idx, new_wrap);
            }
        }
    }

    // Phase 1: Propagate wrapping.
    for &idx in &normal_indices {
        let module = graph.normal_module(idx).unwrap();
        let current_wrap = wrap_updates.get(&idx).copied().unwrap_or(module.wrap_kind);

        if current_wrap.is_none() {
            // Even non-wrapped modules must wrap CJS dependencies.
            let deps: Vec<(ModuleIdx, ExportsKind, ImportKind)> = module
                .import_records
                .iter()
                .filter_map(|r| {
                    let target = r.resolved_module?;
                    let m = graph.normal_module(target)?;
                    Some((target, m.exports_kind, r.kind))
                })
                .collect();
            for (dep, ek, kind) in deps {
                if kind == ImportKind::Require && !required_by_other.contains(&dep) {
                    required_by_other.push(dep);
                }
                if ek == ExportsKind::CommonJs {
                    wrap_recursively(
                        graph,
                        dep,
                        &mut visited,
                        &mut wrap_updates,
                        &mut original_wrap_kinds,
                        &mut required_by_other,
                        config,
                    );
                }
            }
        } else {
            // This module is wrapped — recursively wrap all its dependencies.
            let deps: Vec<(ModuleIdx, ImportKind)> = module
                .import_records
                .iter()
                .filter_map(|r| Some((r.resolved_module?, r.kind)))
                .collect();
            for (dep, kind) in deps {
                if kind == ImportKind::Require && !required_by_other.contains(&dep) {
                    required_by_other.push(dep);
                }
                wrap_recursively(
                    graph,
                    dep,
                    &mut visited,
                    &mut wrap_updates,
                    &mut original_wrap_kinds,
                    &mut required_by_other,
                    config,
                );
            }
        }
    }

    // Phase 2: Create wrapper symbols (skipped when skip_symbol_creation is set).
    let mut wrapper_refs: FxHashMap<ModuleIdx, SymbolRef> = FxHashMap::default();
    if !config.skip_symbol_creation {
        // First, collect the info we need (reading &graph).
        let mut to_create: Vec<(ModuleIdx, WrapKind, String)> = Vec::new();

        for &idx in &normal_indices {
            let wrap_kind = wrap_updates
                .get(&idx)
                .copied()
                .unwrap_or_else(|| graph.normal_module(idx).unwrap().wrap_kind);
            if wrap_kind.is_none() {
                continue;
            }
            let path_stem = graph
                .normal_module(idx)
                .unwrap()
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("module")
                .to_string();
            to_create.push((idx, wrap_kind, path_stem));
        }

        // Now create symbols (&mut graph).
        for (idx, wrap_kind, stem) in to_create {
            let name = match wrap_kind {
                WrapKind::Cjs => format!("require_{stem}"),
                WrapKind::Esm => format!("init_{stem}"),
                WrapKind::None => unreachable!(),
            };
            let sym = graph.add_symbol(idx, name);
            wrapper_refs.insert(idx, sym);
        }
    }

    WrapModulesResult {
        wrap_kind_updates: wrap_updates,
        wrapper_refs,
        original_wrap_kinds,
        required_by_other_module: required_by_other,
    }
}

/// Recursively propagate wrapping to a target module and its dependencies.
fn wrap_recursively(
    graph: &ModuleGraph,
    target: ModuleIdx,
    visited: &mut [bool],
    wrap_updates: &mut FxHashMap<ModuleIdx, WrapKind>,
    original_wrap_kinds: &mut FxHashMap<ModuleIdx, WrapKind>,
    required_by_other: &mut Vec<ModuleIdx>,
    config: &WrapModulesConfig,
) {
    // Already visited or runtime module — skip.
    if visited[target.index()] {
        return;
    }
    if graph.runtime() == Some(target) {
        return;
    }
    visited[target.index()] = true;

    let Some(module) = graph.normal_module(target) else {
        return; // External module — skip.
    };

    // On-demand wrapping: skip ESM/None modules with no import records
    // unless they are execution_order_sensitive or (with strict mode) required by other modules.
    if config.on_demand_wrapping && !module.exports_kind.is_commonjs() {
        let skip = if config.strict_execution_order {
            !module.execution_order_sensitive
                && module.import_records.is_empty()
                && !required_by_other.contains(&target)
        } else {
            !module.execution_order_sensitive && module.import_records.is_empty()
        };
        if skip {
            return;
        }
    }

    // If target has no wrap_kind yet, assign one based on exports_kind.
    let current_wrap = wrap_updates.get(&target).copied().unwrap_or(module.wrap_kind);

    if current_wrap.is_none() {
        let new_wrap = match module.exports_kind {
            ExportsKind::CommonJs => WrapKind::Cjs,
            ExportsKind::Esm | ExportsKind::None => WrapKind::Esm,
        };
        wrap_updates.insert(target, new_wrap);
        // Record original wrap_kind before any further propagation modifies it.
        original_wrap_kinds.entry(target).or_insert(new_wrap);
    }

    // Recurse into dependencies.
    let deps: Vec<(ModuleIdx, ImportKind)> =
        module.import_records.iter().filter_map(|r| Some((r.resolved_module?, r.kind))).collect();
    for (dep, kind) in deps {
        if kind == ImportKind::Require && !required_by_other.contains(&dep) {
            required_by_other.push(dep);
        }
        wrap_recursively(
            graph,
            dep,
            visited,
            wrap_updates,
            original_wrap_kinds,
            required_by_other,
            config,
        );
    }
}
