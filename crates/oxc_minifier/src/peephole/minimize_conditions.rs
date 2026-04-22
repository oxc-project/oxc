use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_compat::ESFeature;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_semantic::ReferenceFlags;
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

/// Minimize Conditions
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeMinimizeConditions.java>
impl<'a> PeepholeOptimizations {
    // The goal of this function is to "rotate" the AST if it's possible to use the
    // left-associative property of the operator to avoid unnecessary parentheses.
    //
    // When using this, make absolutely sure that the operator is actually
    // associative. For example, the "+" operator is not associative for
    // floating-point numbers.
    pub fn join_with_left_associative_op(
        span: Span,
        op: LogicalOperator,
        a: Expression<'a>,
        b: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // "(a, b) op c" => "a, b op c"
        if let Expression::SequenceExpression(mut sequence_expr) = a {
            if let Some(right) = sequence_expr.expressions.pop() {
                sequence_expr
                    .expressions
                    .push(Self::join_with_left_associative_op(span, op, right, b, ctx));
            }
            return Expression::SequenceExpression(sequence_expr);
        }
        let mut a = a;
        let mut b = b;
        // "a op (b op c)" => "(a op b) op c"
        // "a op (b op (c op d))" => "((a op b) op c) op d"
        loop {
            if let Expression::LogicalExpression(logical_expr) = &mut b
                && logical_expr.operator == op
            {
                let right = logical_expr.left.take_in(ctx.ast);
                a = Self::join_with_left_associative_op(span, op, a, right, ctx);
                b = logical_expr.right.take_in(ctx.ast);
                continue;
            }
            break;
        }
        // "a op b" => "a op b"
        // "(a op b) op c" => "(a op b) op c"
        let mut logic_expr = ctx.ast.expression_logical(span, a, op, b);
        Self::minimize_logical_expression(&mut logic_expr, ctx);
        logic_expr
    }

    // `typeof foo === 'number'` -> `typeof foo == 'number'`
    //  ^^^^^^^^^^ `ctx.expression_value_type(&e.left).is_string()` is `true`.
    // `a instanceof b === true` -> `a instanceof b`
    // `a instanceof b === false` -> `!(a instanceof b)`
    //  ^^^^^^^^^^^^^^ `ctx.expression_value_type(&e.left).is_boolean()` is `true`.
    // `x >> +y !== 0` -> `x >> +y`
    //  ^^^^^^^ ctx.expression_value_type(&e.left).is_number()` is `true`.
    pub fn minimize_binary(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::BinaryExpression(e) = expr else { return };
        if !e.operator.is_equality() {
            return;
        }
        let left = e.left.value_type(ctx);
        let right = e.right.value_type(ctx);
        if left.is_undetermined() || right.is_undetermined() {
            return;
        }
        if left == right {
            match e.operator {
                BinaryOperator::StrictInequality => {
                    e.operator = BinaryOperator::Inequality;
                }
                BinaryOperator::StrictEquality => {
                    e.operator = BinaryOperator::Equality;
                }
                _ => {}
            }
        }
        if !left.is_boolean() {
            return;
        }
        if e.right.may_have_side_effects(ctx) {
            return;
        }
        let Some(mut b) = e.right.evaluate_value(ctx).and_then(ConstantValue::into_boolean) else {
            return;
        };
        match e.operator {
            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                e.operator = BinaryOperator::Equality;
                b = !b;
            }
            BinaryOperator::StrictEquality => {
                e.operator = BinaryOperator::Equality;
            }
            BinaryOperator::Equality => {}
            _ => return,
        }
        *expr = if b {
            e.left.take_in(ctx.ast)
        } else {
            let argument = e.left.take_in(ctx.ast);
            ctx.ast.expression_unary(e.span, UnaryOperator::LogicalNot, argument)
        };
        ctx.state.changed = true;
    }

    /// Compress `foo == true` into `foo == 1`.
    ///
    /// - `foo == true` => `foo == 1`
    /// - `foo != false` => `foo != 0`
    ///
    /// In `IsLooselyEqual`, `true` and `false` are converted to `1` and `0` first.
    /// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-islooselyequal>
    pub fn minimize_loose_boolean(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::BinaryExpression(e) = e else { return };
        if !matches!(e.operator, BinaryOperator::Equality | BinaryOperator::Inequality) {
            return;
        }
        if let Some(ConstantValue::Boolean(left_bool)) = e.left.evaluate_value(ctx) {
            e.left = ctx.ast.expression_numeric_literal(
                e.left.span(),
                if left_bool { 1.0 } else { 0.0 },
                None,
                NumberBase::Decimal,
            );
            ctx.state.changed = true;
            return;
        }
        if let Some(ConstantValue::Boolean(right_bool)) = e.right.evaluate_value(ctx) {
            e.right = ctx.ast.expression_numeric_literal(
                e.right.span(),
                if right_bool { 1.0 } else { 0.0 },
                None,
                NumberBase::Decimal,
            );
            ctx.state.changed = true;
        }
    }

    /// Returns the identifier or the assignment target's identifier of the given expression.
    pub fn extract_id_or_assign_to_id<'b>(
        expr: &'b Expression<'a>,
    ) -> Option<&'b IdentifierReference<'a>> {
        match expr {
            Expression::Identifier(id) => Some(id),
            Expression::AssignmentExpression(assign_expr) => {
                if assign_expr.operator == AssignmentOperator::Assign
                    && let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign_expr.left
                {
                    return Some(id);
                }
                None
            }
            _ => None,
        }
    }

    /// Compress `a = a || b` to `a ||= b`
    ///
    /// This can only be done for resolved identifiers as this would avoid setting `a` when `a` is truthy.
    pub fn minimize_normal_assignment_to_combined_logical_assignment(
        expr: &mut AssignmentExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !ctx.supports_feature(ESFeature::ES2021LogicalAssignmentOperators)
            || !matches!(expr.operator, AssignmentOperator::Assign)
        {
            return;
        }

        let Expression::LogicalExpression(logical_expr) = &mut expr.right else { return };
        // NOTE: if the right hand side is an anonymous function, applying this compression will
        // set the `name` property of that function.
        // Since codes relying on the fact that function's name is undefined should be rare,
        // we do this compression even if `keep_names` is enabled.

        let (
            AssignmentTarget::AssignmentTargetIdentifier(write_id_ref),
            Expression::Identifier(read_id_ref),
        ) = (&expr.left, &logical_expr.left)
        else {
            return;
        };
        // It should also early return when the reference might refer to a reference value created by a with statement
        // when the minifier supports with statements
        if write_id_ref.name != read_id_ref.name || ctx.is_global_reference(write_id_ref) {
            return;
        }

        let reference = ctx.scoping_mut().get_reference_mut(write_id_ref.reference_id());
        reference.flags_mut().insert(ReferenceFlags::Read);

        let new_op = logical_expr.operator.to_assignment_operator();
        expr.operator = new_op;
        expr.right = logical_expr.right.take_in(ctx.ast);
        ctx.state.changed = true;
    }

    /// Compress `a = a + b` to `a += b`
    pub fn minimize_normal_assignment_to_combined_assignment(
        expr: &mut AssignmentExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !matches!(expr.operator, AssignmentOperator::Assign) {
            return;
        }
        let Expression::BinaryExpression(binary_expr) = &mut expr.right else { return };
        let Some(new_op) = binary_expr.operator.to_assignment_operator() else { return };
        if !Self::has_no_side_effect_for_evaluation_same_target(&expr.left, &binary_expr.left, ctx)
        {
            return;
        }

        Self::mark_assignment_target_as_read(&expr.left, ctx);

        expr.operator = new_op;
        expr.right = binary_expr.right.take_in(ctx.ast);
        ctx.state.changed = true;
    }

    /// Compress `a -= 1` to `--a` and `a -= -1` to `++a`
    #[expect(clippy::float_cmp)]
    pub fn minimize_assignment_to_update_expression(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(e) = expr else { return };
        if !matches!(e.operator, AssignmentOperator::Subtraction) {
            return;
        }
        let operator = if let Expression::NumericLiteral(num) = &e.right {
            if num.value == 1.0 {
                UpdateOperator::Decrement
            } else if num.value == -1.0 {
                UpdateOperator::Increment
            } else {
                return;
            }
        } else {
            return;
        };
        let Some(target) = e.left.as_simple_assignment_target_mut() else { return };
        let target = target.take_in(ctx.ast);
        *expr = ctx.ast.expression_update(e.span, operator, true, target);
        ctx.state.changed = true;
    }
}
