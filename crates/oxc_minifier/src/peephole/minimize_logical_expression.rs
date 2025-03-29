use oxc_ast::ast::*;
use oxc_span::{ContentEq, GetSpan};
use oxc_syntax::es_target::ESTarget;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_logical_expression(
        &self,
        e: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        Self::try_compress_is_null_or_undefined(e, ctx)
            .or_else(|| self.try_compress_logical_expression_to_assignment_expression(e, ctx))
    }

    /// Compress `foo === null || foo === undefined` into `foo == null`.
    ///
    /// `foo === null || foo === undefined` => `foo == null`
    /// `foo !== null && foo !== undefined` => `foo != null`
    ///
    /// Also supports `(a = foo.bar) === null || a === undefined` which commonly happens when
    /// optional chaining is lowered. (`(a=foo.bar)==null`)
    ///
    /// This compression assumes that `document.all` is a normal object.
    /// If that assumption does not hold, this compression is not allowed.
    /// - `document.all === null || document.all === undefined` is `false`
    /// - `document.all == null` is `true`
    fn try_compress_is_null_or_undefined(
        expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let op = expr.operator;
        let target_ops = match op {
            LogicalOperator::Or => (BinaryOperator::StrictEquality, BinaryOperator::Equality),
            LogicalOperator::And => (BinaryOperator::StrictInequality, BinaryOperator::Inequality),
            LogicalOperator::Coalesce => return None,
        };
        if let Some(new_expr) = Self::try_compress_is_null_or_undefined_for_left_and_right(
            &mut expr.left,
            &mut expr.right,
            expr.span,
            target_ops,
            ctx,
        ) {
            return Some(new_expr);
        }
        let Expression::LogicalExpression(left) = &mut expr.left else {
            return None;
        };
        if left.operator != op {
            return None;
        }
        let new_span = Span::new(left.right.span().start, expr.span.end);
        Self::try_compress_is_null_or_undefined_for_left_and_right(
            &mut left.right,
            &mut expr.right,
            new_span,
            target_ops,
            ctx,
        )
        .map(|new_expr| {
            ctx.ast.expression_logical(
                expr.span,
                ctx.ast.move_expression(&mut left.left),
                expr.operator,
                new_expr,
            )
        })
    }

    fn try_compress_is_null_or_undefined_for_left_and_right(
        left: &mut Expression<'a>,
        right: &mut Expression<'a>,
        span: Span,
        (find_op, replace_op): (BinaryOperator, BinaryOperator),
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        enum LeftPairValueResult {
            Null(Span),
            Undefined,
        }

        let (
            Expression::BinaryExpression(left_binary_expr),
            Expression::BinaryExpression(right_binary_expr),
        ) = (left, right)
        else {
            return None;
        };
        if left_binary_expr.operator != find_op || right_binary_expr.operator != find_op {
            return None;
        }

        let is_null_or_undefined = |a: &Expression| {
            if a.is_null() {
                Some(LeftPairValueResult::Null(a.span()))
            } else if ctx.is_expression_undefined(a) {
                Some(LeftPairValueResult::Undefined)
            } else {
                None
            }
        };
        let (left_value, (left_non_value_expr, left_id_name)) = {
            let left_value;
            let left_non_value;
            if let Some(v) = is_null_or_undefined(&left_binary_expr.left) {
                left_value = v;
                let left_non_value_id =
                    Self::extract_id_or_assign_to_id(&left_binary_expr.right)?.name;
                left_non_value = (&mut left_binary_expr.right, left_non_value_id);
            } else {
                left_value = is_null_or_undefined(&left_binary_expr.right)?;
                let left_non_value_id =
                    Self::extract_id_or_assign_to_id(&left_binary_expr.left)?.name;
                left_non_value = (&mut left_binary_expr.left, left_non_value_id);
            }
            (left_value, left_non_value)
        };

        let (right_value, right_id) = Self::commutative_pair(
            (&right_binary_expr.left, &right_binary_expr.right),
            |a| match left_value {
                LeftPairValueResult::Null(_) => ctx.is_expression_undefined(a).then_some(None),
                LeftPairValueResult::Undefined => a.is_null().then_some(Some(a.span())),
            },
            |b| {
                if let Expression::Identifier(id) = b { Some(id) } else { None }
            },
        )?;

        if left_id_name != right_id.name {
            return None;
        }

        let null_expr_span = match left_value {
            LeftPairValueResult::Null(span) => span,
            LeftPairValueResult::Undefined => right_value.unwrap(),
        };
        Some(ctx.ast.expression_binary(
            span,
            ctx.ast.move_expression(left_non_value_expr),
            replace_op,
            ctx.ast.expression_null_literal(null_expr_span),
        ))
    }

    /// Returns `true` if the assignment target and expression have no side effect for *evaluation* and points to the same reference.
    ///
    /// Evaluation here means `Evaluation` in the spec.
    /// <https://tc39.es/ecma262/multipage/syntax-directed-operations.html#sec-evaluation>
    ///
    /// Matches the following cases (`a` can be `this`):
    ///
    /// - `a`, `a`
    /// - `a.b`, `a.b`
    /// - `a["b"]`, `a["b"]`
    /// - `a[0]`, `a[0]`
    pub fn has_no_side_effect_for_evaluation_same_target(
        assignment_target: &AssignmentTarget,
        expr: &Expression,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        if let (
            AssignmentTarget::AssignmentTargetIdentifier(write_id_ref),
            Expression::Identifier(read_id_ref),
        ) = (assignment_target, expr)
        {
            return write_id_ref.name == read_id_ref.name;
        }
        if let Some(write_expr) = assignment_target.as_member_expression() {
            if let MemberExpression::ComputedMemberExpression(e) = write_expr {
                if !matches!(
                    e.expression,
                    Expression::StringLiteral(_) | Expression::NumericLiteral(_)
                ) {
                    return false;
                }
            }
            let has_same_object = match &write_expr.object() {
                // It should also return false when the reference might refer to a reference value created by a with statement
                // when the minifier supports with statements
                Expression::Identifier(ident) => !ctx.is_global_reference(ident),
                Expression::ThisExpression(_) => {
                    expr.as_member_expression().is_some_and(|read_expr| {
                        matches!(read_expr.object(), Expression::ThisExpression(_))
                    })
                }
                _ => false,
            };
            if !has_same_object {
                return false;
            }
            if let Some(read_expr) = expr.as_member_expression() {
                return write_expr.content_eq(read_expr);
            }
        }
        false
    }

    /// Compress `a || (a = b)` to `a ||= b`
    fn try_compress_logical_expression_to_assignment_expression(
        &self,
        expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if self.target < ESTarget::ES2020 {
            return None;
        }
        let Expression::AssignmentExpression(assignment_expr) = &mut expr.right else {
            return None;
        };
        if assignment_expr.operator != AssignmentOperator::Assign {
            return None;
        }
        let new_op = expr.operator.to_assignment_operator();
        if !Self::has_no_side_effect_for_evaluation_same_target(
            &assignment_expr.left,
            &expr.left,
            ctx,
        ) {
            return None;
        }
        assignment_expr.span = expr.span;
        assignment_expr.operator = new_op;
        Some(ctx.ast.move_expression(&mut expr.right))
    }
}
