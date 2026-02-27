use oxc_index::IndexVec;

use crate::traits::ModuleStore;
use crate::types::{ImportEdge, ModuleIdx};

use super::Module;

/// Default module graph — an `IndexVec` of modules.
#[derive(Debug, Default)]
pub struct DefaultModuleGraph {
    modules: IndexVec<ModuleIdx, Module>,
}

impl DefaultModuleGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a module to the graph and return its index.
    pub fn add_module(&mut self, module: Module) -> ModuleIdx {
        debug_assert_eq!(module.idx, self.modules.next_idx());
        self.modules.push(module)
    }

    /// Reserve an index for a module that will be added later.
    pub fn next_idx(&self) -> ModuleIdx {
        self.modules.next_idx()
    }
}

impl ModuleStore for DefaultModuleGraph {
    type Module = Module;

    fn module(&self, idx: ModuleIdx) -> &Module {
        &self.modules[idx]
    }

    fn module_mut(&mut self, idx: ModuleIdx) -> &mut Module {
        &mut self.modules[idx]
    }

    fn modules_len(&self) -> usize {
        self.modules.len()
    }

    fn iter_modules(&self) -> impl Iterator<Item = (ModuleIdx, &Module)> {
        self.modules.iter_enumerated()
    }

    fn dependencies(&self, idx: ModuleIdx) -> &[ImportEdge] {
        &self.modules[idx].dependencies
    }
}
