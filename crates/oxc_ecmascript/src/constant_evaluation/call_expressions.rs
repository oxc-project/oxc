use oxc_ast::ast::*;

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, ValueType};
use super::call_expr;

impl<'a> ConstantEvaluation<'a> for CallExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        call_expr::try_fold_known_global_methods(&self.callee, &self.arguments, ctx)
    }
}