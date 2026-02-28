use rustc_hash::FxHashSet;

use crate::graph::ModuleGraph;
use crate::types::ModuleIdx;

/// Compute which modules are affected by top-level `await`.
///
/// A module is TLA-affected if:
/// 1. It directly contains `await` at the top level, OR
/// 2. It **statically** imports a module that is TLA-affected (transitive)
///
/// Dynamic imports (`import()`) do NOT propagate TLA status, because
/// dynamic imports are always async regardless.
///
/// Returns the set of module indices that are TLA or contain a TLA dependency.
pub fn compute_tla(graph: &ModuleGraph) -> FxHashSet<ModuleIdx> {
    let mut result = FxHashSet::default();

    // Collect normal module indices.
    let all_indices: Vec<ModuleIdx> =
        graph.modules.iter_enumerated().filter_map(|(idx, m)| m.as_normal().map(|_| idx)).collect();

    // Early return: count modules with TLA. If none, skip traversal.
    let tla_count = all_indices
        .iter()
        .filter(|&&idx| graph.normal_module(idx).is_some_and(|m| m.has_top_level_await))
        .count();
    if tla_count == 0 {
        return result;
    }

    for idx in all_indices {
        if !result.contains(&idx) {
            let mut visiting = FxHashSet::default();
            check_tla(graph, idx, &mut result, &mut visiting);
        }
    }

    result
}

/// Recursively check if a module is TLA-affected.
fn check_tla(
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

    // Direct TLA.
    if module.has_top_level_await {
        result.insert(idx);
        return true;
    }

    // Check static dependencies only.
    let static_deps: Vec<ModuleIdx> = module.static_dependencies().collect();

    for dep in static_deps {
        if check_tla(graph, dep, result, visiting) {
            result.insert(idx);
            return true;
        }
    }

    false
}
