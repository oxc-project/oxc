use rustc_hash::FxHashSet;

use crate::traits::ModuleStore;
use crate::types::ModuleIdx;

/// Find all cycles in the module dependency graph using DFS.
///
/// Returns a list of cycles, where each cycle is a sequence of module indices
/// forming a loop. Each cycle is reported once.
pub fn find_cycles<M: ModuleStore>(store: &M) -> Vec<Vec<ModuleIdx>> {
    let module_count = store.modules_len();
    let mut cycles = Vec::new();
    let mut visited = FxHashSet::default();
    let mut on_stack = FxHashSet::default();
    let mut stack = Vec::new();

    for i in 0..module_count {
        let idx = ModuleIdx::from_usize(i);
        if !visited.contains(&idx) {
            dfs(store, idx, &mut visited, &mut on_stack, &mut stack, &mut cycles);
        }
    }

    cycles
}

fn dfs<M: ModuleStore>(
    store: &M,
    node: ModuleIdx,
    visited: &mut FxHashSet<ModuleIdx>,
    on_stack: &mut FxHashSet<ModuleIdx>,
    stack: &mut Vec<ModuleIdx>,
    cycles: &mut Vec<Vec<ModuleIdx>>,
) {
    visited.insert(node);
    on_stack.insert(node);
    stack.push(node);

    for dep in store.dependencies(node) {
        let target = dep.target;
        if !visited.contains(&target) {
            dfs(store, target, visited, on_stack, stack, cycles);
        } else if on_stack.contains(&target) {
            // Found a cycle: extract the cycle from the stack.
            let cycle_start = stack.iter().position(|&n| n == target).unwrap();
            let cycle: Vec<ModuleIdx> = stack[cycle_start..].to_vec();
            cycles.push(cycle);
        }
    }

    stack.pop();
    on_stack.remove(&node);
}
