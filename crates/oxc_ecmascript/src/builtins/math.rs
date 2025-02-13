use oxc_ast::ast::Argument;

use crate::constant_evaluation::ValueType;

use super::traits::BuiltinValue;

pub struct MathAbs;
impl BuiltinValue for MathAbs {
    fn value_type(&self) -> ValueType {
        ValueType::Object
    }

    fn may_have_side_effects_on_call(&self, args: &[Argument<'_>]) -> bool {
        // TODO: use !maybe_symbol_or_bigint_or_to_primitive_may_return_symbol_or_bigint(args[0])
        args.len() <= 1
            && args.first().is_some_and(|arg| matches!(arg, Argument::NumericLiteral(_)))
    }
}
