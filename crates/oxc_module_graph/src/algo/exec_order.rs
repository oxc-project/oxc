use std::fmt::Debug;
use std::hash::Hash;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::traits::{ModuleInfo, ModuleStore};
use crate::types::ImportKind;

/// Configuration for execution order computation.
pub struct ExecOrderConfig {
    /// If true, dynamic imports are also followed (used when code splitting is disabled).
    pub include_dynamic_imports: bool,
}

/// Result of execution order computation.
pub struct ExecOrderResult<I: Copy + Eq + Hash + Debug> {
    /// Module indices in execution order.
    pub sorted: Vec<I>,
    /// Detected circular dependencies (each cycle as a list of module indices).
    pub cycles: Vec<Vec<I>>,
}

/// Compute execution order for the module graph using DFS post-order.
///
/// This matches the JavaScript module evaluation order defined by the spec:
/// dependencies are evaluated before their dependents, and the order of
/// imports within a module determines sibling priority.
///
/// - `entries`: entry module indices, processed in order (last entry = highest priority)
/// - `runtime`: optional runtime module index, processed first via DFS (always ends up first
///   in result since it has no dependencies)
/// - `config`: controls whether dynamic imports are followed
pub fn compute_exec_order<M: ModuleStore>(
    store: &M,
    entries: &[M::ModuleIdx],
    runtime: Option<M::ModuleIdx>,
    config: &ExecOrderConfig,
) -> ExecOrderResult<M::ModuleIdx> {
    let mut result = Vec::new();
    let mut visited = FxHashSet::default();
    let mut cycles = Vec::new();

    // Stack-based DFS to avoid deep recursion.
    // Each entry is processed; entries are pushed in reverse order so
    // the first entry is processed first.
    let mut stack: Vec<StackEntry<M::ModuleIdx>> = Vec::new();

    // Push entries in reverse so first entry is processed first (LIFO).
    for &entry in entries.iter().rev() {
        stack.push(StackEntry::Enter(entry));
    }

    // Runtime module is pushed last (on top of stack) so it's processed first.
    // It goes through the normal DFS path rather than being pre-added.
    if let Some(rt) = runtime {
        stack.push(StackEntry::Enter(rt));
    }

    // Track the DFS path for proper cycle detection.
    // `dfs_path` is the current path from root to current node.
    // `path_set` maps module index to its position in `dfs_path` for O(1) lookup.
    let mut dfs_path: Vec<M::ModuleIdx> = Vec::new();
    let mut path_set: FxHashMap<M::ModuleIdx, usize> = FxHashMap::default();

    while let Some(item) = stack.pop() {
        match item {
            StackEntry::Enter(idx) => {
                if visited.contains(&idx) {
                    // Already fully visited — check for cycle (back-edge).
                    if let Some(&pos) = path_set.get(&idx) {
                        // Back-edge detected: collect full cycle path.
                        let mut cycle: Vec<M::ModuleIdx> =
                            dfs_path[pos..].to_vec();
                        cycle.push(idx);
                        cycles.push(cycle);
                    }
                    continue;
                }

                if !visited.insert(idx) {
                    continue;
                }

                // Push onto DFS path.
                let pos = dfs_path.len();
                dfs_path.push(idx);
                path_set.insert(idx, pos);

                // Push exit marker so we know when to finalize this module.
                stack.push(StackEntry::Exit(idx));

                // Get import records and push dependencies in reverse order.
                let Some(module) = store.module(idx) else {
                    continue;
                };

                let mut deps = Vec::new();
                module.for_each_import_record(&mut |_rec_idx, resolved, kind| {
                    let follow = match kind {
                        ImportKind::Dynamic => config.include_dynamic_imports,
                        ImportKind::Static | ImportKind::Require => true,
                        ImportKind::HotAccept => false, // HMR-only, not a graph edge
                    };
                    if follow && let Some(target) = resolved {
                        deps.push(target);
                    }
                });

                // Push in reverse so first dependency is processed first.
                for &dep in deps.iter().rev() {
                    stack.push(StackEntry::Enter(dep));
                }
            }
            StackEntry::Exit(idx) => {
                // Pop from DFS path.
                dfs_path.pop();
                path_set.remove(&idx);
                result.push(idx);
            }
        }
    }

    ExecOrderResult { sorted: result, cycles }
}

enum StackEntry<I> {
    /// Module to be entered (pre-order).
    Enter(I),
    /// Module to be finalized (post-order).
    Exit(I),
}
