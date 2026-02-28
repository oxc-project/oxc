use rustc_hash::FxHashSet;

use crate::graph::ModuleGraph;
use crate::types::ModuleIdx;

/// Compute which modules have dynamic exports due to `export *` chains.
///
/// A module has dynamic exports if:
/// 1. It uses CommonJS (`is_commonjs` is true), OR
/// 2. It has `export *` targeting an external module, OR
/// 3. It has `export *` from a module that itself has dynamic exports (transitive)
///
/// Returns the set of module indices that have dynamic exports.
pub fn compute_has_dynamic_exports(graph: &ModuleGraph) -> FxHashSet<ModuleIdx> {
    let mut result = FxHashSet::default();

    let all_indices: Vec<ModuleIdx> =
        graph.modules.iter_enumerated().filter_map(|(idx, m)| m.as_normal().map(|_| idx)).collect();

    for idx in all_indices {
        if !result.contains(&idx) {
            let mut visiting = FxHashSet::default();
            check_dynamic(graph, idx, &mut result, &mut visiting);
        }
    }

    result
}

/// Recursively check if a module has dynamic exports.
fn check_dynamic(
    graph: &ModuleGraph,
    idx: ModuleIdx,
    result: &mut FxHashSet<ModuleIdx>,
    visiting: &mut FxHashSet<ModuleIdx>,
) -> bool {
    if result.contains(&idx) {
        return true;
    }

    if !visiting.insert(idx) {
        return false;
    }

    let Some(module) = graph.normal_module(idx) else {
        return false;
    };

    // CJS modules always have dynamic exports.
    if module.is_commonjs {
        result.insert(idx);
        return true;
    }

    // Check star export targets.
    let star_targets: Vec<ModuleIdx> = module.star_export_modules().collect();

    for target in star_targets {
        // External module → dynamic.
        if graph.external_module(target).is_some() {
            result.insert(idx);
            return true;
        }

        // Transitive: if target has dynamic exports, so do we.
        if check_dynamic(graph, target, result, visiting) {
            result.insert(idx);
            return true;
        }
    }

    false
}
