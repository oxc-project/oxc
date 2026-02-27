use std::fmt::Debug;
use std::hash::Hash;

use rustc_hash::FxHashSet;

use crate::traits::ModuleStore;

/// Find all cycles in the module dependency graph using DFS.
///
/// Returns a list of cycles, where each cycle is a sequence of module indices
/// forming a loop. Each cycle is reported once.
pub fn find_cycles<M: ModuleStore>(store: &M) -> Vec<Vec<M::ModuleIdx>>
where
    M::ModuleIdx: Copy + Eq + Hash + Debug,
{
    let mut cycles = Vec::new();
    let mut visited = FxHashSet::default();
    let mut on_stack = FxHashSet::default();
    let mut stack = Vec::new();

    // Collect all module indices.
    let mut all_indices: Vec<M::ModuleIdx> = Vec::new();
    store.for_each_module(&mut |idx, _| {
        all_indices.push(idx);
    });

    for idx in all_indices {
        if !visited.contains(&idx) {
            dfs(store, idx, &mut visited, &mut on_stack, &mut stack, &mut cycles);
        }
    }

    cycles
}

fn dfs<M: ModuleStore>(
    store: &M,
    node: M::ModuleIdx,
    visited: &mut FxHashSet<M::ModuleIdx>,
    on_stack: &mut FxHashSet<M::ModuleIdx>,
    stack: &mut Vec<M::ModuleIdx>,
    cycles: &mut Vec<Vec<M::ModuleIdx>>,
) where
    M::ModuleIdx: Copy + Eq + Hash + Debug,
{
    visited.insert(node);
    on_stack.insert(node);
    stack.push(node);

    // Collect dependencies to avoid borrowing store in the closure.
    let mut deps = Vec::new();
    store.for_each_dependency(node, &mut |dep| {
        deps.push(dep);
    });

    for target in deps {
        if !visited.contains(&target) {
            dfs(store, target, visited, on_stack, stack, cycles);
        } else if on_stack.contains(&target) {
            // Found a cycle: extract the cycle from the stack.
            let cycle_start = stack.iter().position(|&n| n == target).unwrap();
            let cycle: Vec<M::ModuleIdx> = stack[cycle_start..].to_vec();
            cycles.push(cycle);
        }
    }

    stack.pop();
    on_stack.remove(&node);
}
