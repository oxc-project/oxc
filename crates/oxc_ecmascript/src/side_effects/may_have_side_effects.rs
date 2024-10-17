use oxc_ast::ast::{Expression, ForStatementLeft, PropertyKey, UnaryExpression};

use super::check_for_state_change::CheckForStateChange;

/// Returns true if the node which may have side effects when executed.
/// This version default to the "safe" assumptions when the compiler object
/// is not provided (RegExp have side-effects, etc).
///
/// Ported from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
pub trait MayHaveSideEffects<'a, 'b>
where
    Self: CheckForStateChange<'a, 'b>,
{
    fn may_have_side_effects(&self) -> bool {
        self.check_for_state_change(/* check_for_new_objects */ false)
    }
}

impl<'a, 'b> MayHaveSideEffects<'a, 'b> for Expression<'a> {}
impl<'a, 'b> MayHaveSideEffects<'a, 'b> for UnaryExpression<'a> {}
impl<'a, 'b> MayHaveSideEffects<'a, 'b> for ForStatementLeft<'a> {}
impl<'a, 'b> MayHaveSideEffects<'a, 'b> for PropertyKey<'a> {}
