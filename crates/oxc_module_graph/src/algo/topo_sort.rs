use rustc_hash::FxHashSet;

use crate::traits::ModuleStore;
use crate::types::ModuleIdx;

/// Topological sort of module dependencies using Kahn's algorithm.
///
/// Returns `Some(ordered)` if the graph is a DAG, or `None` if cycles exist.
/// The order is a valid topological ordering: dependencies come before dependents.
pub fn topological_sort<M: ModuleStore>(
    store: &M,
    entries: &[ModuleIdx],
) -> Option<Vec<ModuleIdx>> {
    let module_count = store.modules_len();
    if module_count == 0 {
        return Some(Vec::new());
    }

    // Collect all reachable modules from entries via BFS.
    let mut reachable = FxHashSet::default();
    let mut bfs_queue = std::collections::VecDeque::new();
    for &entry in entries {
        if reachable.insert(entry) {
            bfs_queue.push_back(entry);
        }
    }
    while let Some(idx) = bfs_queue.pop_front() {
        for dep in store.dependencies(idx) {
            if reachable.insert(dep.target) {
                bfs_queue.push_back(dep.target);
            }
        }
    }

    // Compute in-degrees for reachable modules.
    let mut in_degree: Vec<usize> = vec![0; module_count];
    for &idx in &reachable {
        for dep in store.dependencies(idx) {
            if reachable.contains(&dep.target) {
                in_degree[dep.target.index()] += 1;
            }
        }
    }

    // Initialize queue with modules that have zero in-degree.
    let mut queue = std::collections::VecDeque::new();
    for &idx in &reachable {
        if in_degree[idx.index()] == 0 {
            queue.push_back(idx);
        }
    }

    let mut result = Vec::with_capacity(reachable.len());

    while let Some(idx) = queue.pop_front() {
        result.push(idx);
        for dep in store.dependencies(idx) {
            if reachable.contains(&dep.target) {
                in_degree[dep.target.index()] -= 1;
                if in_degree[dep.target.index()] == 0 {
                    queue.push_back(dep.target);
                }
            }
        }
    }

    if result.len() == reachable.len() {
        Some(result)
    } else {
        // Cycles exist — not all modules were processed.
        None
    }
}
