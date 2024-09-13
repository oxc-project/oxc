use oxc_ast::ast::*;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::{node_util::NodeUtil, tri::Tri, CompressorPass};

/// Minimize Conditions
///
/// A peephole optimization that minimizes conditional expressions according to De Morgan's laws.
/// Also rewrites conditional statements as expressions by replacing them
/// with `? :` and short-circuit binary operators.
///
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeMinimizeConditions.java>
pub struct MinimizeConditions;

impl<'a> CompressorPass<'a> for MinimizeConditions {}

impl<'a> Traverse<'a> for MinimizeConditions {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fold_expression(expr, ctx);
    }
}

impl<'a> MinimizeConditions {
    pub fn new() -> Self {
        Self
    }

    fn fold_expression(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(folded_expr) = match expr {
            Expression::ConditionalExpression(e) => self.try_fold_conditional_expression(e, ctx),
            Expression::UnaryExpression(e) if e.operator.is_not() => self.try_minimize_not(e, ctx),
            _ => None,
        } {
            *expr = folded_expr;
        };
    }

    fn try_fold_conditional_expression(
        &self,
        expr: &mut ConditionalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match ctx.get_boolean_value(&expr.test) {
            Tri::True => {
                // Bail `let o = { f() { assert.ok(this !== o); } }; (true ? o.f : false)(); (true ? o.f : false)``;`
                let parent = ctx.ancestry.parent();
                if parent.is_tagged_template_expression()
                    || matches!(parent, Ancestor::CallExpressionCallee(_))
                {
                    return None;
                }
                Some(ctx.ast.move_expression(&mut expr.consequent))
            }
            Tri::False => Some(ctx.ast.move_expression(&mut expr.alternate)),
            Tri::Unknown => None,
        }
    }

    /// Try to minimize NOT nodes such as `!(x==y)`.
    fn try_minimize_not(
        &self,
        expr: &mut UnaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        debug_assert!(expr.operator.is_not());
        if let Expression::BinaryExpression(binary_expr) = &mut expr.argument {
            if let Some(new_op) = binary_expr.operator.equality_inverse_operator() {
                binary_expr.operator = new_op;
                return Some(ctx.ast.move_expression(&mut expr.argument));
            }
        }
        None
    }
}
