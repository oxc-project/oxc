use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// Callback trait for consumer-specific side-effects checks.
///
/// Follows the same pattern as `ImportMatcher`: algorithms call these methods
/// at specific points to allow consumers to inject custom logic.
pub trait SideEffectsChecker {
    /// The module index type.
    type ModuleIdx: Copy + Eq + Hash + Debug;

    /// Check if a star export edge introduces side effects due to wrapping.
    ///
    /// Called for `export * from '...'` edges where the importee has
    /// no side effects itself. Returns `true` if the edge should be
    /// considered side-effectful (e.g., wrapped CJS/ESM modules,
    /// modules with dynamic exports).
    fn star_export_has_side_effects(
        &self,
        importer: Self::ModuleIdx,
        importee: Self::ModuleIdx,
    ) -> bool;
}

/// Default checker that never adds extra side effects from star exports.
pub struct DefaultSideEffectsChecker<Idx>(PhantomData<Idx>);

impl<Idx> Default for DefaultSideEffectsChecker<Idx> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Idx: Copy + Eq + Hash + Debug> SideEffectsChecker for DefaultSideEffectsChecker<Idx> {
    type ModuleIdx = Idx;

    fn star_export_has_side_effects(&self, _importer: Idx, _importee: Idx) -> bool {
        false
    }
}
