use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, IsInt32OrUint32};
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Simplify syntax when we know it's used inside a boolean context, e.g. `if (boolean_context) {}`.
    ///
    /// `SimplifyBooleanExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2059>
    pub fn minimize_expression_in_boolean_context(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match expr {
            Expression::UnaryExpression(u1) if u1.operator.is_not() => {
                if Self::try_negate_expression(&mut u1.argument, ctx, true) {
                    Self::minimize_expression_in_boolean_context(&mut u1.argument, ctx);
                    ctx.replace_expression_with(expr, Self::unwrap_unary);
                } else {
                    Self::minimize_expression_in_boolean_context(&mut u1.argument, ctx);
                }
            }
            Expression::BinaryExpression(e)
                if e.operator.is_equality()
                    && e.right.is_number_0()
                    && e.left.is_int32_or_uint32(ctx) =>
            {
                ctx.replace_expression_with(expr, |e, ctx| {
                    let Expression::BinaryExpression(e) = e else { unreachable!() };
                    let BinaryExpression { operator: op, left, span, .. } = e.unbox();
                    if matches!(op, BinaryOperator::StrictInequality | BinaryOperator::Inequality) {
                        // `if ((a | b) !== 0)` -> `if (a | b);`
                        left
                    } else {
                        // `if ((a | b) === 0);", "if (!(a | b));")`
                        Expression::new_unary_expression(span, UnaryOperator::LogicalNot, left, ctx)
                    }
                });
            }
            // "if (!!a && !!b)" => "if (a && b)"
            Expression::LogicalExpression(e) if e.operator.is_and() => {
                Self::minimize_expression_in_boolean_context(&mut e.left, ctx);
                Self::minimize_expression_in_boolean_context(&mut e.right, ctx);
                // "if (anything && truthyNoSideEffects)" => "if (anything)"
                if e.right.get_side_free_boolean_value(ctx) == Some(true) {
                    ctx.drop_expression(&e.right);
                    ctx.replace_expression_with(expr, |e, _ctx| {
                        let Expression::LogicalExpression(e) = e else { unreachable!() };
                        e.unbox().left
                    });
                }
            }
            // "if (!!a ||!!b)" => "if (a || b)"
            Expression::LogicalExpression(e) if e.operator.is_or() => {
                Self::minimize_expression_in_boolean_context(&mut e.left, ctx);
                Self::minimize_expression_in_boolean_context(&mut e.right, ctx);
                // "if (anything || falsyNoSideEffects)" => "if (anything)"
                if e.right.get_side_free_boolean_value(ctx) == Some(false) {
                    ctx.drop_expression(&e.right);
                    ctx.replace_expression_with(expr, |e, _ctx| {
                        let Expression::LogicalExpression(e) = e else { unreachable!() };
                        e.unbox().left
                    });
                }
            }
            Expression::ConditionalExpression(e) => {
                // "if (a ? !!b : !!c)" => "if (a ? b : c)"
                Self::minimize_expression_in_boolean_context(&mut e.consequent, ctx);
                Self::minimize_expression_in_boolean_context(&mut e.alternate, ctx);
                if let Some(boolean) = e.consequent.get_side_free_boolean_value(ctx) {
                    ctx.drop_expression(&e.consequent);
                    ctx.replace_expression_with(expr, |e, ctx| {
                        let Expression::ConditionalExpression(e) = e else { unreachable!() };
                        let ConditionalExpression { test, alternate, span, .. } = e.unbox();
                        let (op, left) = if boolean {
                            // "if (anything1 ? truthyNoSideEffects : anything2)" => "if (anything1 || anything2)"
                            (LogicalOperator::Or, test)
                        } else {
                            // "if (anything1 ? falsyNoSideEffects : anything2)" => "if (!anything1 && anything2)"
                            (LogicalOperator::And, Self::minimize_not(test.span(), test, ctx, true))
                        };
                        Self::join_with_left_associative_op(span, op, left, alternate, ctx)
                    });
                    return;
                }
                if let Some(boolean) = e.alternate.get_side_free_boolean_value(ctx) {
                    ctx.drop_expression(&e.alternate);
                    ctx.replace_expression_with(expr, |e, ctx| {
                        let Expression::ConditionalExpression(e) = e else { unreachable!() };
                        let ConditionalExpression { test, consequent, span, .. } = e.unbox();
                        let (op, left) = if boolean {
                            // "if (anything1 ? anything2 : truthyNoSideEffects)" => "if (!anything1 || anything2)"
                            (LogicalOperator::Or, Self::minimize_not(test.span(), test, ctx, true))
                        } else {
                            // "if (anything1 ? anything2 : falsyNoSideEffects)" => "if (anything1 && anything2)"
                            (LogicalOperator::And, test)
                        };
                        Self::join_with_left_associative_op(span, op, left, consequent, ctx)
                    });
                }
            }
            Expression::SequenceExpression(seq_expr) => {
                if let Some(last) = seq_expr.expressions.last_mut() {
                    Self::minimize_expression_in_boolean_context(last, ctx);
                }
            }
            // A binding that is provably falsy in boolean context (a write-once
            // falsy `var` whose constant was withheld from value-context folding,
            // e.g. a bundled `var hydrating = false` flag) folds to `false` here,
            // where `undefined`-before-init is indistinguishable from the falsy
            // init. The existing `if (false)` dead-code pass removes the branch.
            Expression::Identifier(ident) => {
                let reference_id = ident.reference_id();
                let span = ident.span;
                if let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id()
                    && ctx.state.symbols.value(symbol_id).is_some_and(|sv| sv.boolean_falsy)
                {
                    let new_expr =
                        Expression::new_numeric_literal(span, 0.0, None, NumberBase::Decimal, ctx);
                    ctx.replace_expression(expr, new_expr);
                }
            }
            _ => {}
        }
    }
}
