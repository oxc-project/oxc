use rustc_hash::FxHashMap;

use crate::traits::{ModuleInfo, ModuleStore, SideEffectsChecker};

/// Determine which modules have side effects, considering transitive dependencies.
///
/// A module has side effects if:
/// 1. Its `side_effects()` returns `Some(true)` or `None` (no-treeshake), OR
/// 2. Its `side_effects()` returns `Some(false)` but any import dependency
///    (transitively) has side effects, OR
/// 3. It has `export * from '...'` edges where the checker says the edge
///    introduces side effects (e.g., due to CJS wrapping or dynamic exports)
///
/// Returns a map from module index to the propagated side-effects value:
/// - `true` — module has side effects
/// - `false` — module and all its dependencies are side-effect-free
pub fn determine_side_effects<M, C>(store: &M, checker: &C) -> FxHashMap<M::ModuleIdx, bool>
where
    M: ModuleStore,
    C: SideEffectsChecker<ModuleIdx = M::ModuleIdx>,
{
    let mut cache: FxHashMap<M::ModuleIdx, CacheState> = FxHashMap::default();
    let mut result: FxHashMap<M::ModuleIdx, bool> = FxHashMap::default();

    // Collect all module indices.
    let mut all_indices: Vec<M::ModuleIdx> = Vec::with_capacity(store.modules_len());
    store.for_each_module(&mut |idx, _| {
        all_indices.push(idx);
    });

    for idx in all_indices {
        let has = check_side_effects(store, checker, idx, &mut cache);
        result.insert(idx, has);
    }

    result
}

/// 3-state cache for memoized DFS.
#[derive(Clone, Copy)]
enum CacheState {
    /// Currently being visited (cycle detection).
    Visiting,
    /// Result is cached.
    Cached(bool),
}

fn check_side_effects<M, C>(
    store: &M,
    checker: &C,
    idx: M::ModuleIdx,
    cache: &mut FxHashMap<M::ModuleIdx, CacheState>,
) -> bool
where
    M: ModuleStore,
    C: SideEffectsChecker<ModuleIdx = M::ModuleIdx>,
{
    // Check cache.
    if let Some(state) = cache.get(&idx) {
        return match state {
            CacheState::Visiting => false, // Cycle — treat as no side effects.
            CacheState::Cached(v) => *v,
        };
    }

    let Some(module) = store.module(idx) else {
        // External module — query via `any_module_side_effects` which can
        // return user-defined side effects (e.g., from package.json).
        let has = matches!(
            store.any_module_side_effects(idx),
            Some(Some(true) | None)
        );
        cache.insert(idx, CacheState::Cached(has));
        return has;
    };

    match module.side_effects() {
        Some(true) | None => {
            // Has side effects or no-treeshake — always true.
            cache.insert(idx, CacheState::Cached(true));
            true
        }
        Some(false) => {
            // Potentially side-effect-free. Check dependencies.
            cache.insert(idx, CacheState::Visiting);

            // Check all import dependencies (all kinds, not just static).
            let mut import_deps = Vec::new();
            module.for_each_import_record(&mut |_rec_idx, resolved, _kind| {
                if let Some(target) = resolved {
                    import_deps.push(target);
                }
            });

            for dep in &import_deps {
                if check_side_effects(store, checker, *dep, cache) {
                    cache.insert(idx, CacheState::Cached(true));
                    return true;
                }
            }

            // Check ESM star export edges for wrap-kind side effects.
            // Only ESM `export * from '...'` — not CJS reexport patterns.
            let mut star_targets = Vec::new();
            module.for_each_esm_star_export(&mut |target| {
                star_targets.push(target);
            });

            for target in &star_targets {
                // Consumer-specific logic (wrapping, dynamic exports).
                // The target's own side effects were already checked above
                // via for_each_import_record, so only check the star-export
                // specific condition here.
                if checker.star_export_has_side_effects(idx, *target) {
                    cache.insert(idx, CacheState::Cached(true));
                    return true;
                }
            }

            cache.insert(idx, CacheState::Cached(false));
            false
        }
    }
}
