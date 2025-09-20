use num_traits::ToPrimitive;
use oxc_ast::ast::*;

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue};

impl<'a> ConstantEvaluation<'a> for StaticMemberExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<super::ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.property.name.as_str() {
            "length" => evaluate_value_length(&self.object, ctx),
            _ => None,
        }
    }
}

impl<'a> ConstantEvaluation<'a> for ComputedMemberExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<super::ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match &self.expression {
            Expression::StringLiteral(s) if s.value == "length" => {
                evaluate_value_length(&self.object, ctx)
            }
            _ => None,
        }
    }
}

fn evaluate_value_length<'a>(
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if let Some(ConstantValue::String(s)) = object.evaluate_value(ctx) {
        Some(ConstantValue::Number(s.encode_utf16().count().to_f64().unwrap()))
    } else if let Expression::ArrayExpression(arr) = object {
        if arr.elements.iter().any(|e| matches!(e, ArrayExpressionElement::SpreadElement(_))) {
            None
        } else {
            Some(ConstantValue::Number(arr.elements.len().to_f64().unwrap()))
        }
    } else {
        None
    }
}