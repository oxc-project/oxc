use rustc_hash::FxHashMap;

use crate::graph::ModuleGraph;
use crate::types::{ExportsKind, ImportKind, ModuleIdx, WrapKind};

/// Configuration for [`determine_module_exports_kind`].
pub struct ExportsKindConfig {
    /// Treat dynamic imports like `require()` for wrapping purposes.
    /// Set true when code splitting is disabled (e.g., IIFE output).
    pub dynamic_imports_as_require: bool,
    /// Whether CJS entry points should be wrapped.
    /// true = wrap CJS entries (ESM output format).
    /// false = don't wrap CJS entries (CJS/IIFE output format).
    pub wrap_cjs_entries: bool,
}

/// Result of [`determine_module_exports_kind`].
pub struct ExportsKindResult {
    /// Modules whose exports_kind changed from None to a concrete value.
    pub exports_kind_updates: FxHashMap<ModuleIdx, ExportsKind>,
    /// Modules that need initial wrapping.
    pub wrap_kind_updates: FxHashMap<ModuleIdx, WrapKind>,
}

/// Get the effective exports_kind for a module, checking in-flight updates first.
fn effective_exports_kind(
    graph: &ModuleGraph,
    updates: &FxHashMap<ModuleIdx, ExportsKind>,
    idx: ModuleIdx,
) -> ExportsKind {
    if let Some(&kind) = updates.get(&idx) {
        return kind;
    }
    graph.normal_module(idx).map_or(ExportsKind::None, |m| m.exports_kind)
}

/// Classify each module's export format and mark initial wrapping needs.
///
/// **Phase 1**: Scan import records. For each normal module (importer), for each
/// import record, determine exports_kind and wrap_kind based on the import kind
/// and the importee's current exports_kind.
///
/// **Phase 2**: Post-process CJS modules. Non-entry CJS modules (or CJS entries
/// when `wrap_cjs_entries` is true) get `WrapKind::Cjs`.
///
/// # Panics
///
/// Panics if a normal module index collected from the graph becomes invalid
/// (should not happen in normal usage).
pub fn determine_module_exports_kind(
    graph: &ModuleGraph,
    config: &ExportsKindConfig,
) -> ExportsKindResult {
    let mut exports_kind_updates: FxHashMap<ModuleIdx, ExportsKind> = FxHashMap::default();
    let mut wrap_kind_updates: FxHashMap<ModuleIdx, WrapKind> = FxHashMap::default();

    // Phase 1: Scan import records.
    let normal_indices: Vec<ModuleIdx> =
        graph.modules.iter_enumerated().filter_map(|(idx, m)| m.as_normal().map(|_| idx)).collect();

    for &importer_idx in &normal_indices {
        let importer = graph.normal_module(importer_idx).unwrap();
        let records: Vec<(ImportKind, Option<ModuleIdx>)> =
            importer.import_records.iter().map(|r| (r.kind, r.resolved_module)).collect();

        for (kind, resolved) in records {
            let Some(importee_idx) = resolved else { continue };
            // Skip external modules — they don't have exports_kind.
            if graph.normal_module(importee_idx).is_none() {
                continue;
            }

            match kind {
                ImportKind::Static => {
                    let importee_ek =
                        effective_exports_kind(graph, &exports_kind_updates, importee_idx);
                    if importee_ek == ExportsKind::None {
                        // Skip ESM classification if the importee has lazy exports.
                        // Modules with lazy exports keep ExportsKind::None even when
                        // statically imported, letting the consumer handle them specially.
                        let has_lazy =
                            graph.normal_module(importee_idx).is_some_and(|m| m.has_lazy_export);
                        if !has_lazy {
                            exports_kind_updates.insert(importee_idx, ExportsKind::Esm);
                        }
                    }
                    continue;
                }
                ImportKind::Require => {}
                ImportKind::Dynamic if config.dynamic_imports_as_require => {}
                ImportKind::Dynamic | ImportKind::HotAccept => continue,
            }

            // Require (or dynamic-as-require) path.
            let importee_ek = effective_exports_kind(graph, &exports_kind_updates, importee_idx);
            match importee_ek {
                ExportsKind::Esm => {
                    wrap_kind_updates.insert(importee_idx, WrapKind::Esm);
                }
                ExportsKind::CommonJs => {
                    wrap_kind_updates.insert(importee_idx, WrapKind::Cjs);
                }
                ExportsKind::None => {
                    exports_kind_updates.insert(importee_idx, ExportsKind::CommonJs);
                    wrap_kind_updates.insert(importee_idx, WrapKind::Cjs);
                }
            }
        }
    }

    // Phase 2: Post-process CJS modules.
    let entries: Vec<ModuleIdx> = graph.entries().to_vec();
    for &idx in &normal_indices {
        let ek = effective_exports_kind(graph, &exports_kind_updates, idx);
        if ek != ExportsKind::CommonJs {
            continue;
        }
        let is_entry = entries.contains(&idx);
        if !is_entry || config.wrap_cjs_entries {
            wrap_kind_updates.entry(idx).or_insert(WrapKind::Cjs);
        }
    }

    ExportsKindResult { exports_kind_updates, wrap_kind_updates }
}
