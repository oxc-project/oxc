use oxc_ast::ast::*;

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, ValueType};

impl<'a> ConstantEvaluation<'a> for LogicalExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.operator {
            LogicalOperator::And => match self.left.evaluate_value_to_boolean(ctx) {
                Some(true) => self.right.evaluate_value(ctx),
                Some(false) => self.left.evaluate_value(ctx),
                None => {
                    // ToBoolean(a && false) -> false
                    if target_ty == Some(ValueType::Boolean)
                        && self.right.evaluate_value_to_boolean(ctx) == Some(false)
                    {
                        Some(ConstantValue::Boolean(false))
                    } else {
                        None
                    }
                }
            },
            LogicalOperator::Or => match self.left.evaluate_value_to_boolean(ctx) {
                Some(true) => self.left.evaluate_value(ctx),
                Some(false) => self.right.evaluate_value(ctx),
                None => {
                    // ToBoolean(a || true) -> true
                    if target_ty == Some(ValueType::Boolean)
                        && self.right.evaluate_value_to_boolean(ctx) == Some(true)
                    {
                        Some(ConstantValue::Boolean(true))
                    } else {
                        None
                    }
                }
            },
            LogicalOperator::Coalesce => None,
        }
    }
}