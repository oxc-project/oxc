use rustc_hash::FxHashSet;

use crate::graph::ModuleGraph;
use crate::module::Module;
use crate::types::ModuleIdx;

/// Find all cycles in the module dependency graph using DFS.
///
/// Returns a list of cycles, where each cycle is a sequence of module indices
/// forming a loop. Each cycle is reported once.
pub fn find_cycles(graph: &ModuleGraph) -> Vec<Vec<ModuleIdx>> {
    let mut cycles = Vec::new();
    let mut visited = FxHashSet::default();
    let mut on_stack = FxHashSet::default();
    let mut stack = Vec::new();

    for (idx, _) in graph.modules.iter_enumerated() {
        if !visited.contains(&idx) {
            dfs(graph, idx, &mut visited, &mut on_stack, &mut stack, &mut cycles);
        }
    }

    cycles
}

fn dfs(
    graph: &ModuleGraph,
    node: ModuleIdx,
    visited: &mut FxHashSet<ModuleIdx>,
    on_stack: &mut FxHashSet<ModuleIdx>,
    stack: &mut Vec<ModuleIdx>,
    cycles: &mut Vec<Vec<ModuleIdx>>,
) {
    visited.insert(node);
    on_stack.insert(node);
    stack.push(node);

    // Collect dependencies.
    let deps: Vec<ModuleIdx> = match &graph.modules[node] {
        Module::Normal(module) => {
            module.import_records.iter().filter_map(|rec| rec.resolved_module).collect()
        }
        Module::External(_) => Vec::new(),
    };

    for target in deps {
        if !visited.contains(&target) {
            dfs(graph, target, visited, on_stack, stack, cycles);
        } else if on_stack.contains(&target) {
            let cycle_start = stack.iter().position(|&n| n == target).unwrap();
            let cycle: Vec<ModuleIdx> = stack[cycle_start..].to_vec();
            cycles.push(cycle);
        }
    }

    stack.pop();
    on_stack.remove(&node);
}
