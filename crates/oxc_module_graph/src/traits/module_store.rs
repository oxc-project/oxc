use crate::types::{ImportEdge, ModuleIdx};

use super::ModuleInfo;

/// A collection of modules, indexed by `ModuleIdx`.
///
/// This trait abstracts over different module storage strategies.
/// Rolldown can implement this on `ModuleTable`,
/// while the default implementation uses `ModuleGraph`.
pub trait ModuleStore {
    type Module: ModuleInfo;

    /// Get a reference to a module by index.
    fn module(&self, idx: ModuleIdx) -> &Self::Module;

    /// Get a mutable reference to a module by index.
    fn module_mut(&mut self, idx: ModuleIdx) -> &mut Self::Module;

    /// The number of modules in the store.
    fn modules_len(&self) -> usize;

    /// Iterate over all modules with their indices.
    fn iter_modules(&self) -> impl Iterator<Item = (ModuleIdx, &Self::Module)>;

    /// The import edges (dependencies) for a given module.
    fn dependencies(&self, idx: ModuleIdx) -> &[ImportEdge];
}
