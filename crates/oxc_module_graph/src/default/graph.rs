use oxc_index::IndexVec;

use crate::traits::ModuleStore;
use crate::types::{ModuleIdx, SymbolRef};

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
    type ModuleIdx = ModuleIdx;
    type SymbolRef = SymbolRef;
    type Module = Module;

    fn module(&self, idx: ModuleIdx) -> Option<&Module> {
        self.modules.get(idx)
    }

    fn modules_len(&self) -> usize {
        self.modules.len()
    }

    fn for_each_module(&self, f: &mut dyn FnMut(ModuleIdx, &Module)) {
        for (idx, module) in self.modules.iter_enumerated() {
            f(idx, module);
        }
    }

    fn for_each_dependency(&self, idx: ModuleIdx, f: &mut dyn FnMut(ModuleIdx)) {
        if let Some(module) = self.modules.get(idx) {
            for dep in &module.dependencies {
                f(dep.target);
            }
        }
    }
}
