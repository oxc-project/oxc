use rustc_hash::FxHashSet;

use crate::traits::{ModuleInfo, ModuleStore};

/// Compute which modules have dynamic exports due to `export *` chains.
///
/// A module has dynamic exports if:
/// 1. It uses CommonJS (`is_commonjs()` is true), OR
/// 2. It has `export *` targeting an external module (not in the store), OR
/// 3. It has `export *` from a module that itself has dynamic exports (transitive)
///
/// Returns the set of module indices that have dynamic exports.
pub fn compute_has_dynamic_exports<M: ModuleStore>(store: &M) -> FxHashSet<M::ModuleIdx> {
    let mut result = FxHashSet::default();

    // Collect all module indices.
    let mut all_indices: Vec<M::ModuleIdx> = Vec::with_capacity(store.modules_len());
    store.for_each_module(&mut |idx, _| {
        all_indices.push(idx);
    });

    for idx in all_indices {
        if !result.contains(&idx) {
            let mut visiting = FxHashSet::default();
            check_dynamic(store, idx, &mut result, &mut visiting);
        }
    }

    result
}

/// Recursively check if a module has dynamic exports.
///
/// Uses `visiting` for cycle detection to avoid infinite recursion.
/// Uses `result` as a cache of already-confirmed dynamic modules.
fn check_dynamic<M: ModuleStore>(
    store: &M,
    idx: M::ModuleIdx,
    result: &mut FxHashSet<M::ModuleIdx>,
    visiting: &mut FxHashSet<M::ModuleIdx>,
) -> bool {
    // Already confirmed dynamic.
    if result.contains(&idx) {
        return true;
    }

    // Cycle — treat as not dynamic to avoid infinite recursion.
    if !visiting.insert(idx) {
        return false;
    }

    let Some(module) = store.module(idx) else {
        // Module not in store (external) — not directly checked here,
        // but handled by the star export target check below.
        return false;
    };

    // CJS modules always have dynamic exports.
    if module.is_commonjs() {
        result.insert(idx);
        return true;
    }

    // Check star export targets.
    let mut star_targets = Vec::new();
    module.for_each_star_export(&mut |target| {
        star_targets.push(target);
    });

    for target in star_targets {
        // External module (not in store) → dynamic.
        if store.module(target).is_none() {
            result.insert(idx);
            return true;
        }

        // Transitive: if target has dynamic exports, so do we.
        if check_dynamic(store, target, result, visiting) {
            result.insert(idx);
            return true;
        }
    }

    false
}
