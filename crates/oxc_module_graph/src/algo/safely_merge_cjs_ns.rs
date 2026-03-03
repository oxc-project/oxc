use rustc_hash::FxHashMap;

use crate::graph::ModuleGraph;
use crate::types::{ImportKind, ImportRecordMeta, ModuleIdx, SymbolRef};

/// Info about CJS namespace refs that can be safely merged.
#[derive(Debug, Default)]
pub struct SafelyMergeCjsNsInfo {
    /// Namespace symbols from ESM imports that can be merged into one.
    pub namespace_refs: Vec<SymbolRef>,
    /// Whether __toESM interop is needed.
    pub needs_interop: bool,
}

/// Identify ESM imports of CJS modules whose namespace refs can be merged.
///
/// For each normal module, find ESM `import` records (not `export *`) that
/// resolve to a CommonJS module. Collect the namespace refs for each CJS
/// target so the consumer can link them to a single representative symbol.
pub fn determine_safely_merge_cjs_ns(
    graph: &ModuleGraph,
) -> FxHashMap<ModuleIdx, SafelyMergeCjsNsInfo> {
    let mut map: FxHashMap<ModuleIdx, SafelyMergeCjsNsInfo> = FxHashMap::default();

    for module in graph.normal_modules() {
        for rec in &module.import_records {
            // Only ESM imports, not star exports
            if rec.kind != ImportKind::Static {
                continue;
            }
            if rec.meta.contains(ImportRecordMeta::IS_EXPORT_STAR) {
                continue;
            }
            let Some(target_idx) = rec.resolved_module else { continue };
            let Some(target) = graph.normal_module(target_idx) else { continue };
            if !target.exports_kind.is_commonjs() {
                continue;
            }
            let entry = map.entry(target_idx).or_default();
            entry.namespace_refs.push(rec.namespace_ref);
            // needs_interop: true for CJS targets (consumer may refine this)
            entry.needs_interop = true;
        }
    }
    map
}
