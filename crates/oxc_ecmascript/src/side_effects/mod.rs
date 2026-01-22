mod context;
mod expressions;
mod info;
mod known_globals;
mod pure_function;
mod statements;

pub use context::{MayHaveSideEffectsContext, PropertyReadSideEffects, PropertyWriteSideEffects};
pub use expressions::is_valid_regexp;
pub use info::SideEffectInfo;
pub use known_globals::{
    is_known_global_ident, is_side_effect_free_member_access,
    is_side_effect_free_nested_member_access,
};
pub use pure_function::is_pure_function;

/// Returns true if subtree changes application state.
///
/// This trait assumes the following:
/// - `.toString()`, `.valueOf()`, and `[Symbol.toPrimitive]()` are side-effect free.
///   - This is mainly to assume `ToPrimitive` is side-effect free.
///   - Note that the builtin `Array::toString` has a side-effect when a value contains a Symbol as `ToString(Symbol)` throws an error. Maybe we should revisit this assumption and remove it.
///     - For example, `"" == [Symbol()]` returns an error, but this trait returns `false`.
/// - Errors thrown when creating a String or an Array that exceeds the maximum length does not happen.
/// - TDZ errors does not happen.
///
/// Ported from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
pub trait MayHaveSideEffects<'a> {
    /// Returns `true` if the expression/statement may have side effects.
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool;

    /// Returns rich side effect information for bundler tree-shaking.
    ///
    /// This provides more granular information than [`may_have_side_effects`],
    /// including whether the expression accesses global variables or has pure annotations.
    ///
    /// The default implementation converts the boolean result to [`SideEffectInfo`].
    /// Types that need to track additional information should override this method.
    fn side_effect_info(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> SideEffectInfo {
        self.may_have_side_effects(ctx).into()
    }
}

impl<'a, T: MayHaveSideEffects<'a>> MayHaveSideEffects<'a> for Option<T> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.as_ref().is_some_and(|t| t.may_have_side_effects(ctx))
    }

    fn side_effect_info(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> SideEffectInfo {
        self.as_ref().map_or(SideEffectInfo::empty(), |t| t.side_effect_info(ctx))
    }
}
