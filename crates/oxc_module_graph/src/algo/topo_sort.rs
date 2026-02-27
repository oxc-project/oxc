use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::traits::ModuleStore;

/// Topological sort of module dependencies using Kahn's algorithm.
///
/// Returns `Some(ordered)` if the graph is a DAG, or `None` if cycles exist.
/// The order is a valid topological ordering: dependencies come before dependents.
pub fn topological_sort<M: ModuleStore>(
    store: &M,
    entries: &[M::ModuleIdx],
) -> Option<Vec<M::ModuleIdx>>
where
    M::ModuleIdx: Copy + Eq + Hash + Debug,
{
    if store.modules_len() == 0 {
        return Some(Vec::new());
    }

    // Collect all reachable modules from entries via BFS.
    let mut reachable = FxHashSet::default();
    let mut bfs_queue = VecDeque::new();
    for &entry in entries {
        if reachable.insert(entry) {
            bfs_queue.push_back(entry);
        }
    }
    while let Some(idx) = bfs_queue.pop_front() {
        store.for_each_dependency(idx, &mut |dep| {
            if reachable.insert(dep) {
                bfs_queue.push_back(dep);
            }
        });
    }

    // Compute in-degrees for reachable modules.
    let mut in_degree: FxHashMap<M::ModuleIdx, usize> = FxHashMap::default();
    for &idx in &reachable {
        in_degree.entry(idx).or_insert(0);
        store.for_each_dependency(idx, &mut |dep| {
            if reachable.contains(&dep) {
                *in_degree.entry(dep).or_insert(0) += 1;
            }
        });
    }

    // Initialize queue with modules that have zero in-degree.
    let mut queue = VecDeque::new();
    for &idx in &reachable {
        if in_degree[&idx] == 0 {
            queue.push_back(idx);
        }
    }

    let mut result = Vec::with_capacity(reachable.len());

    while let Some(idx) = queue.pop_front() {
        result.push(idx);
        store.for_each_dependency(idx, &mut |dep| {
            if reachable.contains(&dep)
                && let Some(d) = in_degree.get_mut(&dep)
            {
                *d -= 1;
                if *d == 0 {
                    queue.push_back(dep);
                }
            }
        });
    }

    if result.len() == reachable.len() {
        Some(result)
    } else {
        // Cycles exist — not all modules were processed.
        None
    }
}
