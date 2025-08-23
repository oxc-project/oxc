use oxc_ast::ast::Expression;

use crate::GlobalContext;

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

pub trait MayHaveSideEffectsContext<'a>: GlobalContext<'a> {
    /// Whether to respect the pure annotations.
    ///
    /// Pure annotations are the comments that marks that a expression is pure.
    /// For example, `/* @__PURE__ */`, `/* #__NO_SIDE_EFFECTS__ */`.
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-annotations>
    fn annotations(&self) -> bool;

    /// Whether to treat this function call as pure.
    ///
    /// This function is called for normal function calls, new calls, and
    /// tagged template calls (`foo()`, `new Foo()`, ``foo`b` ``).
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-manualpurefunctions>
    fn manual_pure_functions(&self, callee: &Expression) -> bool;

    /// Whether property read accesses have side effects.
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-propertyreadsideeffects>
    fn property_read_side_effects(&self) -> PropertyReadSideEffects;

    /// Whether accessing a global variable has side effects.
    ///
    /// Accessing a non-existing global variable will throw an error.
    /// Global variable may be a getter that has side effects.
    ///
    /// <https://rollupjs.org/configuration-options/#treeshake-unknownglobalsideeffects>
    fn unknown_global_side_effects(&self) -> bool;
}
