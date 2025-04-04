use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, IsInt32OrUint32};
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn try_fold_stmt_in_boolean_context(&self, stmt: &mut Statement<'a>, ctx: Ctx<'a, '_>) {
        let expr = match stmt {
            Statement::IfStatement(s) => Some(&mut s.test),
            Statement::WhileStatement(s) => Some(&mut s.test),
            Statement::ForStatement(s) => s.test.as_mut(),
            Statement::DoWhileStatement(s) => Some(&mut s.test),
            _ => None,
        };

        if let Some(expr) = expr {
            self.try_fold_expr_in_boolean_context(expr, ctx);
        }
    }

    /// Simplify syntax when we know it's used inside a boolean context, e.g. `if (boolean_context) {}`.
    ///
    /// `SimplifyBooleanExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2059>
    pub fn try_fold_expr_in_boolean_context(
        &self,
        expr: &mut Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        match expr {
            // "!!a" => "a"
            Expression::UnaryExpression(u1) if u1.operator.is_not() => {
                if let Expression::UnaryExpression(u2) = &mut u1.argument {
                    if u2.operator.is_not() {
                        let mut e = u2.argument.take_in(ctx.ast.allocator);
                        self.try_fold_expr_in_boolean_context(&mut e, ctx);
                        *expr = e;
                        return true;
                    }
                }
            }
            Expression::BinaryExpression(e)
                if e.operator.is_equality()
                    && matches!(&e.right, Expression::NumericLiteral(lit) if lit.value == 0.0)
                    && e.left.is_int32_or_uint32(&ctx) =>
            {
                let argument = e.left.take_in(ctx.ast.allocator);
                *expr = if matches!(
                    e.operator,
                    BinaryOperator::StrictInequality | BinaryOperator::Inequality
                ) {
                    // `if ((a | b) !== 0)` -> `if (a | b);`
                    argument
                } else {
                    // `if ((a | b) === 0);", "if (!(a | b));")`
                    ctx.ast.expression_unary(e.span, UnaryOperator::LogicalNot, argument)
                };
                return true;
            }
            // "if (!!a && !!b)" => "if (a && b)"
            Expression::LogicalExpression(e) if e.operator.is_and() => {
                self.try_fold_expr_in_boolean_context(&mut e.left, ctx);
                self.try_fold_expr_in_boolean_context(&mut e.right, ctx);
                // "if (anything && truthyNoSideEffects)" => "if (anything)"
                if e.right.get_side_free_boolean_value(&ctx) == Some(true) {
                    *expr = e.left.take_in(ctx.ast.allocator);
                    return true;
                }
            }
            // "if (!!a ||!!b)" => "if (a || b)"
            Expression::LogicalExpression(e) if e.operator == LogicalOperator::Or => {
                self.try_fold_expr_in_boolean_context(&mut e.left, ctx);
                self.try_fold_expr_in_boolean_context(&mut e.right, ctx);
                // "if (anything || falsyNoSideEffects)" => "if (anything)"
                if e.right.get_side_free_boolean_value(&ctx) == Some(false) {
                    *expr = e.left.take_in(ctx.ast.allocator);
                    return true;
                }
            }
            Expression::ConditionalExpression(e) => {
                // "if (a ? !!b : !!c)" => "if (a ? b : c)"
                self.try_fold_expr_in_boolean_context(&mut e.consequent, ctx);
                self.try_fold_expr_in_boolean_context(&mut e.alternate, ctx);
                if let Some(boolean) = e.consequent.get_side_free_boolean_value(&ctx) {
                    let right = e.alternate.take_in(ctx.ast.allocator);
                    let left = e.test.take_in(ctx.ast.allocator);
                    let span = e.span;
                    let (op, left) = if boolean {
                        // "if (anything1 ? truthyNoSideEffects : anything2)" => "if (anything1 || anything2)"
                        (LogicalOperator::Or, left)
                    } else {
                        // "if (anything1 ? falsyNoSideEffects : anything2)" => "if (!anything1 && anything2)"
                        (LogicalOperator::And, self.minimize_not(left.span(), left, ctx))
                    };
                    *expr = self.join_with_left_associative_op(span, op, left, right, ctx);
                    return true;
                }
                if let Some(boolean) = e.alternate.get_side_free_boolean_value(&ctx) {
                    let left = e.test.take_in(ctx.ast.allocator);
                    let right = e.consequent.take_in(ctx.ast.allocator);
                    let span = e.span;
                    let (op, left) = if boolean {
                        // "if (anything1 ? anything2 : truthyNoSideEffects)" => "if (!anything1 || anything2)"
                        (LogicalOperator::Or, self.minimize_not(left.span(), left, ctx))
                    } else {
                        // "if (anything1 ? anything2 : falsyNoSideEffects)" => "if (anything1 && anything2)"
                        (LogicalOperator::And, left)
                    };
                    *expr = self.join_with_left_associative_op(span, op, left, right, ctx);
                    return true;
                }
            }
            Expression::SequenceExpression(seq_expr) => {
                if let Some(last) = seq_expr.expressions.last_mut() {
                    return self.try_fold_expr_in_boolean_context(last, ctx);
                }
            }
            _ => {}
        }
        false
    }
}

#[cfg(test)]
mod test {
    use crate::tester::test;

    #[test]
    fn test_try_fold_in_boolean_context() {
        test("if (!!a);", "a");
        test("while (!!a);", "for (;a;);");
        test("do; while (!!a);", "do; while (a);");
        test("for (;!!a;);", "for (;a;);");
        test("!!a ? b : c", "a ? b : c");
        test("if (!!!a);", "a");
        test("Boolean(!!a)", "a");
        test("if ((a | +b) !== 0);", "a | +b");
        test("if ((a | +b) === 0);", "a | +b");
        test("if (!!a && !!b);", "a && b");
        test("if (!!a || !!b);", "a || b");
        test("if (anything || (0, false));", "anything");
        test("if (a ? !!b : !!c);", "a ? b : c");
        test("if (anything1 ? (0, true) : anything2);", "anything1 || anything2");
        test("if (anything1 ? (0, false) : anything2);", "!anything1 && anything2");
        test("if (anything1 ? anything2 : (0, true));", "!anything1 || anything2");
        test("if (anything1 ? anything2 : (0, false));", "anything1 && anything2");
        test("if(!![]);", "");
        test("if (+a === 0) { b } else { c }", "+a == 0 ? b : c"); // should not be folded to `a ? b : c` (`+a` might be NaN)
        test("if (foo, !!bar) { let baz }", "if (foo, bar) { let baz }");
    }
}
