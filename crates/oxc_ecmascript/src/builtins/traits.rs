use oxc_ast::ast::Argument;

use crate::constant_evaluation::ValueType;

pub trait BuiltinValue {
    fn value_type(&self) -> ValueType;

    fn may_have_side_effects_on_call(&self, _args: &[Argument<'_>]) -> bool {
        true
    }
}
