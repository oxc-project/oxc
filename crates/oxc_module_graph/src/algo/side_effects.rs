use rustc_hash::FxHashMap;

use crate::graph::ModuleGraph;
use crate::hooks::LinkConfig;
use crate::module::{Module, SideEffects};
use crate::types::ModuleIdx;

/// Determine which modules have side effects, considering transitive dependencies.
///
/// A module has side effects if:
/// 1. Its `side_effects` is `True` or `NoTreeshake`, OR
/// 2. Its `side_effects` is `False` but any import dependency
///    (transitively) has side effects, OR
/// 3. It has `export * from '...'` edges where the hooks say the edge
///    introduces side effects (e.g., due to CJS wrapping or dynamic exports)
///
/// Returns a map from module index to the propagated side-effects value.
pub fn determine_side_effects(
    graph: &ModuleGraph,
    config: &LinkConfig,
) -> FxHashMap<ModuleIdx, bool> {
    let mut cache: FxHashMap<ModuleIdx, CacheState> = FxHashMap::default();
    let mut result: FxHashMap<ModuleIdx, bool> = FxHashMap::default();

    for (idx, _) in graph.modules.iter_enumerated() {
        let has = check_side_effects(graph, config, idx, &mut cache);
        result.insert(idx, has);
    }

    result
}

/// 3-state cache for memoized DFS.
#[derive(Clone, Copy)]
enum CacheState {
    Visiting,
    Cached(bool),
}

fn check_side_effects(
    graph: &ModuleGraph,
    config: &LinkConfig,
    idx: ModuleIdx,
    cache: &mut FxHashMap<ModuleIdx, CacheState>,
) -> bool {
    if let Some(state) = cache.get(&idx) {
        return match state {
            CacheState::Visiting => false,
            CacheState::Cached(v) => *v,
        };
    }

    let module = &graph.modules[idx];

    match module {
        Module::External(ext) => {
            let has = match ext.side_effects {
                SideEffects::True | SideEffects::NoTreeshake => true,
                SideEffects::False => false,
            };
            cache.insert(idx, CacheState::Cached(has));
            has
        }
        Module::Normal(m) => {
            match m.side_effects {
                SideEffects::True | SideEffects::NoTreeshake => {
                    cache.insert(idx, CacheState::Cached(true));
                    true
                }
                SideEffects::False => {
                    cache.insert(idx, CacheState::Visiting);

                    // Check all import dependencies.
                    let import_deps: Vec<ModuleIdx> =
                        m.import_records.iter().filter_map(|rec| rec.resolved_module).collect();

                    for dep in &import_deps {
                        if check_side_effects(graph, config, *dep, cache) {
                            cache.insert(idx, CacheState::Cached(true));
                            return true;
                        }
                    }

                    // Check star export edges.
                    let star_targets: Vec<ModuleIdx> = m.star_export_modules().collect();

                    for target in &star_targets {
                        // Consumer-specific logic via hooks.
                        if let Some(hooks) = &config.side_effects_hooks
                            && hooks.star_export_has_extra_side_effects(idx, *target)
                        {
                            cache.insert(idx, CacheState::Cached(true));
                            return true;
                        }
                    }

                    cache.insert(idx, CacheState::Cached(false));
                    false
                }
            }
        }
    }
}
