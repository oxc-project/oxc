use oxc_ast::ast::Expression;

use crate::is_global_reference::IsGlobalReference;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyReadSideEffects {
    /// Treat all property read accesses as side effect free.
    None,
    /// Treat non-member property accesses as side effect free.
    /// Member property accesses are still considered to have side effects.
    OnlyMemberPropertyAccess,
    /// Treat all property read accesses as possible side effects.
    #[default]
    All,
}

pub trait MayHaveSideEffectsContext: IsGlobalReference {
    /// Whether to respect the pure annotations.
    ///
    /// Pure annotations are the comments that marks that a expression is pure.
    /// For example, `/* @__PURE__ */`, `/* #__NO_SIDE_EFFECTS__ */`.
    fn respect_annotations(&self) -> bool;

    /// Whether to treat this function call as pure.
    ///
    /// This function is called for normal function calls, new calls, and
    /// tagged template calls (`foo()`, `new Foo()`, ``foo`b` ``).
    fn is_pure_call(&self, callee: &Expression) -> bool;

    /// Whether property read accesses have side effects.
    fn property_read_side_effects(&self) -> PropertyReadSideEffects;
}
