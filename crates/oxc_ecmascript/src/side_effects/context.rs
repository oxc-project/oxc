use crate::is_global_reference::IsGlobalReference;

pub trait MayHaveSideEffectsContext: IsGlobalReference {
    /// Whether to respect the pure annotations.
    ///
    /// Pure annotations are the comments that marks that a expression is pure.
    /// For example, `/* @__PURE__ */`, `/* #__NO_SIDE_EFFECTS__ */`.
    fn respect_annotations(&self) -> bool;
}
