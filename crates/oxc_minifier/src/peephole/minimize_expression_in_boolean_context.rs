use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ValueType};
use oxc_span::GetSpan;
use oxc_traverse::Ancestor;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn try_fold_stmt_in_boolean_context(stmt: &mut Statement<'a>, ctx: Ctx<'a, '_>) {
        let expr = match stmt {
            Statement::IfStatement(s) => Some(&mut s.test),
            Statement::WhileStatement(s) => Some(&mut s.test),
            Statement::ForStatement(s) => s.test.as_mut(),
            Statement::DoWhileStatement(s) => Some(&mut s.test),
            Statement::ExpressionStatement(s)
                if !matches!(
                    ctx.ancestry.ancestor(1),
                    Ancestor::ArrowFunctionExpressionBody(_)
                ) =>
            {
                Some(&mut s.expression)
            }
            _ => None,
        };

        if let Some(expr) = expr {
            Self::try_fold_expr_in_boolean_context(expr, ctx);
        }
    }

    /// Simplify syntax when we know it's used inside a boolean context, e.g. `if (boolean_context) {}`.
    ///
    /// `SimplifyBooleanExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2059>
    pub fn try_fold_expr_in_boolean_context(expr: &mut Expression<'a>, ctx: Ctx<'a, '_>) -> bool {
        match expr {
            // "!!a" => "a"
            Expression::UnaryExpression(u1) if u1.operator.is_not() => {
                if let Expression::UnaryExpression(u2) = &mut u1.argument {
                    if u2.operator.is_not() {
                        let mut e = ctx.ast.move_expression(&mut u2.argument);
                        Self::try_fold_expr_in_boolean_context(&mut e, ctx);
                        *expr = e;
                        return true;
                    }
                }
            }
            Expression::BinaryExpression(e)
                if e.operator.is_equality()
                    && matches!(&e.right, Expression::NumericLiteral(lit) if lit.value == 0.0)
                    && ValueType::from(&e.left).is_number() =>
            {
                let argument = ctx.ast.move_expression(&mut e.left);
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
            Expression::LogicalExpression(e) if e.operator == LogicalOperator::And => {
                Self::try_fold_expr_in_boolean_context(&mut e.left, ctx);
                Self::try_fold_expr_in_boolean_context(&mut e.right, ctx);
                // "if (anything && truthyNoSideEffects)" => "if (anything)"
                if ctx.get_side_free_boolean_value(&e.right) == Some(true) {
                    *expr = ctx.ast.move_expression(&mut e.left);
                    return true;
                }
            }
            // "if (!!a ||!!b)" => "if (a || b)"
            Expression::LogicalExpression(e) if e.operator == LogicalOperator::Or => {
                Self::try_fold_expr_in_boolean_context(&mut e.left, ctx);
                Self::try_fold_expr_in_boolean_context(&mut e.right, ctx);
                // "if (anything || falsyNoSideEffects)" => "if (anything)"
                if ctx.get_side_free_boolean_value(&e.right) == Some(false) {
                    *expr = ctx.ast.move_expression(&mut e.left);
                    return true;
                }
            }
            Expression::ConditionalExpression(e) => {
                // "if (a ? !!b : !!c)" => "if (a ? b : c)"
                Self::try_fold_expr_in_boolean_context(&mut e.consequent, ctx);
                Self::try_fold_expr_in_boolean_context(&mut e.alternate, ctx);
                if let Some(boolean) = ctx.get_side_free_boolean_value(&e.consequent) {
                    let right = ctx.ast.move_expression(&mut e.alternate);
                    let left = ctx.ast.move_expression(&mut e.test);
                    let span = e.span;
                    let (op, left) = if boolean {
                        // "if (anything1 ? truthyNoSideEffects : anything2)" => "if (anything1 || anything2)"
                        (LogicalOperator::Or, left)
                    } else {
                        // "if (anything1 ? falsyNoSideEffects : anything2)" => "if (!anything1 && anything2)"
                        (LogicalOperator::And, Self::minimize_not(left.span(), left, ctx))
                    };
                    *expr = Self::join_with_left_associative_op(span, op, left, right, ctx);
                    return true;
                }
                if let Some(boolean) = ctx.get_side_free_boolean_value(&e.alternate) {
                    let left = ctx.ast.move_expression(&mut e.test);
                    let right = ctx.ast.move_expression(&mut e.consequent);
                    let span = e.span;
                    let (op, left) = if boolean {
                        // "if (anything1 ? anything2 : truthyNoSideEffects)" => "if (!anything1 || anything2)"
                        (LogicalOperator::Or, Self::minimize_not(left.span(), left, ctx))
                    } else {
                        // "if (anything1 ? anything2 : falsyNoSideEffects)" => "if (anything1 && anything2)"
                        (LogicalOperator::And, left)
                    };
                    *expr = Self::join_with_left_associative_op(span, op, left, right, ctx);
                    return true;
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
        test("if (!!!a);", "!a");
        // test("Boolean(!!a)", "Boolean()");
        test("if ((a | b) !== 0);", "a | b");
        test("if ((a | b) === 0);", "!(a | b)");
        test("if (!!a && !!b);", "a && b");
        test("if (!!a || !!b);", "a || b");
        test("if (anything || (0, false));", "anything");
        test("if (a ? !!b : !!c);", "a ? b : c");
        test("if (anything1 ? (0, true) : anything2);", "anything1 || anything2");
        test("if (anything1 ? (0, false) : anything2);", "!anything1 && anything2");
        test("if (anything1 ? anything2 : (0, true));", "!anything1 || anything2");
        test("if (anything1 ? anything2 : (0, false));", "anything1 && anything2");
        test("if(!![]);", "");
    }
}
