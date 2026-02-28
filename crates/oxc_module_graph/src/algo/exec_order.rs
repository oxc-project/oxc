use rustc_hash::{FxHashMap, FxHashSet};

use crate::graph::ModuleGraph;
use crate::hooks::LinkConfig;
use crate::module::Module;
use crate::types::{ImportKind, ModuleIdx};

/// Configuration for execution order computation.
pub struct ExecOrderConfig {
    /// If true, dynamic imports are also followed (used when code splitting is disabled).
    pub include_dynamic_imports: bool,
}

/// Result of execution order computation.
pub struct ExecOrderResult {
    /// Module indices in execution order.
    pub sorted: Vec<ModuleIdx>,
    /// Detected circular dependencies (each cycle as a list of module indices).
    pub cycles: Vec<Vec<ModuleIdx>>,
}

/// Compute execution order for the module graph using DFS post-order.
///
/// This matches the JavaScript module evaluation order defined by the spec:
/// dependencies are evaluated before their dependents, and the order of
/// imports within a module determines sibling priority.
pub fn compute_exec_order(graph: &ModuleGraph, config: &LinkConfig) -> ExecOrderResult {
    let mut result = Vec::new();
    let mut visited = FxHashSet::default();
    let mut cycles = Vec::new();

    let mut stack: Vec<StackEntry> = Vec::new();

    // Push entries in reverse so first entry is processed first (LIFO).
    for &entry in graph.entries().iter().rev() {
        stack.push(StackEntry::Enter(entry));
    }

    // Runtime module is pushed last (on top of stack) so it's processed first.
    if let Some(rt) = graph.runtime() {
        stack.push(StackEntry::Enter(rt));
    }

    let mut dfs_path: Vec<ModuleIdx> = Vec::new();
    let mut path_set: FxHashMap<ModuleIdx, usize> = FxHashMap::default();

    while let Some(item) = stack.pop() {
        match item {
            StackEntry::Enter(idx) => {
                if visited.contains(&idx) {
                    if let Some(&pos) = path_set.get(&idx) {
                        let mut cycle: Vec<ModuleIdx> = dfs_path[pos..].to_vec();
                        cycle.push(idx);
                        cycles.push(cycle);
                    }
                    continue;
                }

                if !visited.insert(idx) {
                    continue;
                }

                let pos = dfs_path.len();
                dfs_path.push(idx);
                path_set.insert(idx, pos);

                stack.push(StackEntry::Exit(idx));

                // Get dependencies based on module type.
                let mut deps = Vec::new();
                match graph.modules.get(idx) {
                    Some(Module::Normal(module)) => {
                        for rec in &module.import_records {
                            let follow = match rec.kind {
                                ImportKind::Dynamic => config.include_dynamic_imports,
                                ImportKind::Static | ImportKind::Require => true,
                                ImportKind::HotAccept => false,
                            };
                            if follow && let Some(target) = rec.resolved_module {
                                deps.push(target);
                            }
                        }
                    }
                    Some(Module::External(_)) | None => {}
                }

                for &dep in deps.iter().rev() {
                    stack.push(StackEntry::Enter(dep));
                }
            }
            StackEntry::Exit(idx) => {
                dfs_path.pop();
                path_set.remove(&idx);
                result.push(idx);
            }
        }
    }

    ExecOrderResult { sorted: result, cycles }
}

enum StackEntry {
    Enter(ModuleIdx),
    Exit(ModuleIdx),
}
