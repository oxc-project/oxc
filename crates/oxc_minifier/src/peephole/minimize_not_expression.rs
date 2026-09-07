use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::DetermineValueType;
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_not(
        span: Span,
        mut expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
        boolean_context: bool,
    ) -> Expression<'a> {
        if Self::try_negate_expression(&mut expr, ctx, boolean_context) {
            if boolean_context {
                Self::minimize_expression_in_boolean_context(&mut expr, ctx);
            }
            expr
        } else {
            Expression::new_unary_expression(span, UnaryOperator::LogicalNot, expr, ctx)
        }
    }

    pub fn try_negate_expression(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
        boolean_context: bool,
    ) -> bool {
        match expr {
            // `!!true` -> `true`
            // `!!false` -> `false`
            Expression::UnaryExpression(e)
                if e.operator.is_not()
                    && (boolean_context || e.argument.value_type(ctx).is_boolean()) =>
            {
                ctx.replace_expression_with(expr, Self::unwrap_unary);
                true
            }
            // `!(a == b)` => `a != b`
            // `!(a != b)` => `a == b`
            // `!(a === b)` => `a !== b`
            // `!(a !== b)` => `a === b`
            Expression::BinaryExpression(binary_expr) if binary_expr.operator.is_equality() => {
                binary_expr.operator = binary_expr.operator.equality_inverse_operator().unwrap();
                true
            }
            // `!0` => `1`
            // `!1` => `0`
            Expression::NumericLiteral(num) if boolean_context => {
                num.value = if num.value.is_nan() || num.value == 0.0 { 1.0 } else { 0.0 };
                true
            }
            // `!(a == b || c == d)` => `a != b && c != d`
            // `!(a == b && c == d)` => `a != b || c != d`
            // De Morgan's law, only when every comparison in the `&&`/`||` chain
            // inverts its operator in place (equality operators; relational ones
            // are unsound under NaN) and inversion does not add parentheses.
            // The fold is exact and involutive: a later `minimize_not` restores
            // the original chain at no cost, so shapes that consume the `!` for
            // free (branch swaps, `!!` collapses) are unaffected.
            //
            // delta: we do not know the parent precedence, so we must conservatively
            // assume negated operands may still need ().
            // delta == 0 it does assume the parent will not drop parens.
            Expression::LogicalExpression(logical_expr)
                if Self::de_morgan_paren_delta(logical_expr, boolean_context)
                    .is_some_and(|delta| delta <= 0) =>
            {
                Self::de_morgan_invert_logical(logical_expr, ctx, boolean_context);
                true
            }
            // "!(a, b)" => "a, !b"
            Expression::SequenceExpression(sequence_expr) => {
                if let Some(last_expr) = sequence_expr.expressions.last_mut() {
                    ctx.replace_expression_with(last_expr, |old, ctx| {
                        Self::minimize_not(old.span(), old, ctx, boolean_context)
                    });
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// `MaybeSimplifyNot`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L73>
    pub fn minimize_unary(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::UnaryExpression(e) = expr else { return };
        if !e.operator.is_not() {
            return;
        }
        Self::minimize_expression_in_boolean_context(&mut e.argument, ctx);

        if Self::try_negate_expression(&mut e.argument, ctx, false) {
            ctx.replace_expression_with(expr, Self::unwrap_unary);
        }
    }

    /// Attempts to unwrap a `UnaryExpression` from an `Expression` enum variant, returning its argument.
    /// E.g. `!x` into `x`. The discarded `!` wrapper contains no references.
    pub fn unwrap_unary(old: Expression<'a>, _ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let Expression::UnaryExpression(e) = old else { unreachable!() };
        e.unbox().argument
    }

    /// Character delta from parentheses added or removed by De Morgan's law
    /// (flipping `&&` <-> `||` changes which nested operands need parens), or
    /// `None` if some operand cannot invert its operator in place.
    fn de_morgan_paren_delta(e: &LogicalExpression<'a>, boolean_context: bool) -> Option<i32> {
        if !matches!(e.operator, LogicalOperator::And | LogicalOperator::Or) {
            return None;
        }
        let mut delta = 0;
        for side in [&e.left, &e.right] {
            match side {
                Expression::BinaryExpression(b) if b.operator.is_equality() => {}
                Expression::UnaryExpression(u) if u.operator.is_not() => {
                    delta += if boolean_context { -1 } else { 1 }
                }
                Expression::LogicalExpression(child) => {
                    delta += Self::de_morgan_paren_delta(child, boolean_context)?;
                    // `&&` under `||` prints bare but its inversion (`||` under
                    // `&&`) needs parens; the reverse drops parens.
                    match (e.operator, child.operator) {
                        (LogicalOperator::Or, LogicalOperator::And) => delta += 2,
                        (LogicalOperator::And, LogicalOperator::Or) => delta -= 2,
                        _ => {}
                    }
                }
                Expression::Identifier(_)
                | Expression::ThisExpression(_)
                | Expression::NullLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::BigIntLiteral(_)
                | Expression::RegExpLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::TemplateLiteral(_)
                | Expression::ImportMeta(_)
                | Expression::Super(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::CallExpression(_)
                | Expression::ChainExpression(_)
                | Expression::ImportExpression(_)
                | Expression::NewExpression(_)
                | Expression::AwaitExpression(_) => delta += 1,
                Expression::BooleanLiteral(_) => {}
                _ => return None,
            }
        }
        Some(delta)
    }

    /// Apply De Morgan's law in place. Only called on chains approved by
    /// [`Self::de_morgan_paren_delta`].
    fn de_morgan_invert_logical(
        e: &mut LogicalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
        boolean_context: bool,
    ) {
        e.operator = if e.operator == LogicalOperator::And {
            LogicalOperator::Or
        } else {
            LogicalOperator::And
        };
        Self::de_morgan_invert(&mut e.left, ctx, boolean_context);
        Self::de_morgan_invert(&mut e.right, ctx, boolean_context);
    }

    fn de_morgan_invert(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
        boolean_context: bool,
    ) {
        if let Expression::LogicalExpression(e) = expr
            && !e.operator.is_coalesce()
        {
            Self::de_morgan_invert_logical(e, ctx, boolean_context);
            return;
        }

        if !Self::try_negate_expression(expr, ctx, boolean_context) {
            ctx.replace_expression_with(expr, |expr, ctx| {
                Expression::new_unary_expression(expr.span(), UnaryOperator::LogicalNot, expr, ctx)
            });
        }
    }
}
