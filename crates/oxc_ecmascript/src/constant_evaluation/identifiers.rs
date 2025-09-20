use oxc_ast::ast::*;

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, ValueType};

impl<'a> ConstantEvaluation<'a> for IdentifierReference<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.name.as_str() {
            "undefined" if ctx.is_global_reference(self) => Some(ConstantValue::Undefined),
            "NaN" if ctx.is_global_reference(self) => Some(ConstantValue::Number(f64::NAN)),
            "Infinity" if ctx.is_global_reference(self) => {
                Some(ConstantValue::Number(f64::INFINITY))
            }
            _ => self
                .reference_id
                .get()
                .and_then(|reference_id| ctx.get_constant_value_for_reference_id(reference_id)),
        }
    }
}