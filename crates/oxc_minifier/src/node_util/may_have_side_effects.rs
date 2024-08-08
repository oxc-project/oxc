use oxc_ast::ast::*;

use super::check_for_state_change::CheckForStateChange;

/// port from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
/// Returns true if the node which may have side effects when executed.
/// This version default to the "safe" assumptions when the compiler object
/// is not provided (RegExp have side-effects, etc).
pub trait MayHaveSideEffects<'a, 'b>
where
    Self: CheckForStateChange<'a, 'b>,
{
    fn may_have_side_effects(&self) -> bool {
        self.check_for_state_change(false)
    }
}

impl<'a, 'b> MayHaveSideEffects<'a, 'b> for Expression<'a> {}
impl<'a, 'b> MayHaveSideEffects<'a, 'b> for UnaryExpression<'a> {}
