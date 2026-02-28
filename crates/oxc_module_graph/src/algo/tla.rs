use rustc_hash::FxHashSet;

use crate::traits::{ModuleInfo, ModuleStore};

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
pub fn compute_tla<M: ModuleStore>(store: &M) -> FxHashSet<M::ModuleIdx> {
    let mut result = FxHashSet::default();

    // Collect all module indices.
    let mut all_indices: Vec<M::ModuleIdx> = Vec::with_capacity(store.modules_len());
    store.for_each_module(&mut |idx, _| {
        all_indices.push(idx);
    });

    // Early return: count modules with TLA. If none, skip traversal.
    let tla_count = all_indices
        .iter()
        .filter(|&&idx| store.module(idx).is_some_and(ModuleInfo::has_top_level_await))
        .count();
    if tla_count == 0 {
        return result;
    }

    for idx in all_indices {
        if !result.contains(&idx) {
            let mut visiting = FxHashSet::default();
            check_tla(store, idx, &mut result, &mut visiting);
        }
    }

    result
}

/// Recursively check if a module is TLA-affected.
///
/// Uses `visiting` for cycle detection and `result` as a cache.
fn check_tla<M: ModuleStore>(
    store: &M,
    idx: M::ModuleIdx,
    result: &mut FxHashSet<M::ModuleIdx>,
    visiting: &mut FxHashSet<M::ModuleIdx>,
) -> bool {
    // Already confirmed TLA.
    if result.contains(&idx) {
        return true;
    }

    // Cycle — treat as not TLA to break recursion.
    if !visiting.insert(idx) {
        return false;
    }

    let Some(module) = store.module(idx) else {
        return false;
    };

    // Direct TLA.
    if module.has_top_level_await() {
        result.insert(idx);
        return true;
    }

    // Check static dependencies only.
    let mut static_deps = Vec::new();
    store.for_each_static_dependency(idx, &mut |dep| {
        static_deps.push(dep);
    });

    for dep in static_deps {
        if check_tla(store, dep, result, visiting) {
            result.insert(idx);
            return true;
        }
    }

    false
}
