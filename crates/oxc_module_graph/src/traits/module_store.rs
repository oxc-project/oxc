use std::fmt::Debug;
use std::hash::Hash;

use super::ModuleInfo;

/// A collection of modules, indexed by `ModuleIdx`.
///
/// This trait abstracts over different module storage strategies.
/// Rolldown can implement this on `ModuleTable`,
/// while the default implementation uses `DefaultModuleGraph`.
///
/// Uses callback-based iteration (`for_each_*`) instead of returning
/// concrete iterators or slices, so consumers can use any internal
/// storage (e.g., `IndexVec`, `Vec<Option<..>>`, etc.).
pub trait ModuleStore {
    /// The module index type.
    type ModuleIdx: Copy + Eq + Hash + Debug;
    /// The symbol reference type.
    type SymbolRef: Copy + Eq + Hash + Debug;
    /// The module type.
    type Module: ModuleInfo<ModuleIdx = Self::ModuleIdx, SymbolRef = Self::SymbolRef>;

    /// Get a reference to a module by index.
    /// Returns `None` if the index is out of bounds or the module is not a normal module.
    fn module(&self, idx: Self::ModuleIdx) -> Option<&Self::Module>;

    /// The number of modules in the store.
    fn modules_len(&self) -> usize;

    /// Iterate over all modules with their indices.
    fn for_each_module(&self, f: &mut dyn FnMut(Self::ModuleIdx, &Self::Module));

    /// Iterate over the dependency module indices for a given module.
    fn for_each_dependency(&self, idx: Self::ModuleIdx, f: &mut dyn FnMut(Self::ModuleIdx));

    /// Iterate over static import dependency module indices for a given module.
    ///
    /// "Static" means `import` declarations (not `import()` or `require()`).
    /// This is needed by algorithms like TLA propagation that only follow
    /// synchronous import edges.
    fn for_each_static_dependency(&self, idx: Self::ModuleIdx, f: &mut dyn FnMut(Self::ModuleIdx));

    /// Get side-effects status for any module index, including external modules.
    ///
    /// Needed because `module()` only returns normal modules, but external modules
    /// can have user-defined side effects (e.g., from package.json `sideEffects` field).
    ///
    /// Returns:
    /// - `Some(Some(true))` — has side effects
    /// - `Some(Some(false))` — no side effects
    /// - `Some(None)` — no-treeshake (always keep)
    /// - `None` — module not found
    ///
    /// Default: delegates to `module().side_effects()`.
    #[expect(clippy::option_option)]
    fn any_module_side_effects(&self, idx: Self::ModuleIdx) -> Option<Option<bool>> {
        self.module(idx).map(ModuleInfo::side_effects)
    }
}
