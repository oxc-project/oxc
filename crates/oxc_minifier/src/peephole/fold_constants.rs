use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, LogicalOperator},
};
use oxc_traverse::Ancestor;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Constant Folding
    ///
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>
    pub fn fold_constants_exit_expression(&mut self, expr: &mut Expression<'a>, ctx: Ctx<'a, '_>) {
        if let Some(folded_expr) = match expr {
            Expression::BinaryExpression(e) => Self::try_fold_binary_expr(e, ctx)
                .or_else(|| Self::try_fold_binary_typeof_comparison(e, ctx)),
            Expression::UnaryExpression(e) => Self::try_fold_unary_expr(e, ctx),
            Expression::StaticMemberExpression(e) => Self::try_fold_static_member_expr(e, ctx),
            Expression::ComputedMemberExpression(e) => Self::try_fold_computed_member_expr(e, ctx),
            Expression::LogicalExpression(e) => Self::try_fold_logical_expr(e, ctx),
            Expression::ChainExpression(e) => Self::try_fold_optional_chain(e, ctx),
            Expression::CallExpression(e) => Self::try_fold_number_constructor(e, ctx),
            Expression::ObjectExpression(e) => self.fold_object_spread(e, ctx),
            _ => None,
        } {
            *expr = folded_expr;
            self.mark_current_function_as_changed();
        };
    }

    #[expect(clippy::float_cmp)]
    fn try_fold_unary_expr(e: &UnaryExpression<'a>, ctx: Ctx<'a, '_>) -> Option<Expression<'a>> {
        match e.operator {
            // Do not fold `void 0` back to `undefined`.
            UnaryOperator::Void if e.argument.is_number_0() => None,
            // Do not fold `true` and `false` back to `!0` and `!1`
            UnaryOperator::LogicalNot if matches!(&e.argument, Expression::NumericLiteral(lit) if lit.value == 0.0 || lit.value == 1.0) => {
                None
            }
            // Do not fold big int.
            UnaryOperator::UnaryNegation if e.argument.is_big_int_literal() => None,
            _ => ctx.eval_unary_expression(e).map(|v| ctx.value_to_expr(e.span, v)),
        }
    }

    fn try_fold_static_member_expr(
        e: &mut StaticMemberExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // TODO: tryFoldObjectPropAccess(n, left, name)
        ctx.eval_static_member_expression(e).map(|value| ctx.value_to_expr(e.span, value))
    }

    fn try_fold_computed_member_expr(
        e: &mut ComputedMemberExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // TODO: tryFoldObjectPropAccess(n, left, name)
        ctx.eval_computed_member_expression(e).map(|value| ctx.value_to_expr(e.span, value))
    }

    fn try_fold_logical_expr(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        match logical_expr.operator {
            LogicalOperator::And | LogicalOperator::Or => Self::try_fold_and_or(logical_expr, ctx),
            LogicalOperator::Coalesce => Self::try_fold_coalesce(logical_expr, ctx),
        }
    }

    fn try_fold_optional_chain(
        chain_expr: &mut ChainExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let member_expr = chain_expr.expression.as_member_expression()?;
        if !member_expr.optional() {
            return None;
        }
        let object = member_expr.object();
        let ty = ValueType::from(object);
        (ty.is_null() || ty.is_undefined())
            .then(|| ctx.value_to_expr(chain_expr.span, ConstantValue::Undefined))
    }

    /// Try to fold a AND / OR node.
    ///
    /// port from [closure-compiler](https://github.com/google/closure-compiler/blob/09094b551915a6487a980a783831cba58b5739d1/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L587)
    pub fn try_fold_and_or(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let op = logical_expr.operator;
        debug_assert!(matches!(op, LogicalOperator::And | LogicalOperator::Or));

        let left = &logical_expr.left;
        let left_val = ctx.get_boolean_value(left);

        if let Some(lval) = left_val {
            // Bail `0 && (module.exports = {})` for `cjs-module-lexer`.
            if !lval {
                if let Expression::AssignmentExpression(assign_expr) = &logical_expr.right {
                    if let Some(member_expr) = assign_expr.left.as_member_expression() {
                        if member_expr.is_specific_member_access("module", "exports") {
                            return None;
                        }
                    }
                }
            }

            // (TRUE || x) => TRUE (also, (3 || x) => 3)
            // (FALSE && x) => FALSE
            if if lval { op == LogicalOperator::Or } else { op == LogicalOperator::And } {
                return Some(ctx.ast.move_expression(&mut logical_expr.left));
            } else if !ctx.expression_may_have_side_effects(left) {
                let parent = ctx.ancestry.parent();
                // Bail `let o = { f() { assert.ok(this !== o); } }; (true && o.f)(); (true && o.f)``;`
                if parent.is_tagged_template_expression()
                    || matches!(parent, Ancestor::CallExpressionCallee(_))
                {
                    return None;
                }
                // (FALSE || x) => x
                // (TRUE && x) => x
                return Some(ctx.ast.move_expression(&mut logical_expr.right));
            }
            // Left side may have side effects, but we know its boolean value.
            // e.g. true_with_sideeffects || foo() => true_with_sideeffects, foo()
            // or: false_with_sideeffects && foo() => false_with_sideeffects, foo()
            let left = ctx.ast.move_expression(&mut logical_expr.left);
            let right = ctx.ast.move_expression(&mut logical_expr.right);
            let vec = ctx.ast.vec_from_array([left, right]);
            let sequence_expr = ctx.ast.expression_sequence(logical_expr.span, vec);
            return Some(sequence_expr);
        } else if let Expression::LogicalExpression(left_child) = &mut logical_expr.left {
            if left_child.operator == logical_expr.operator {
                let left_child_right_boolean = ctx.get_boolean_value(&left_child.right);
                let left_child_op = left_child.operator;
                if let Some(right_boolean) = left_child_right_boolean {
                    if !ctx.expression_may_have_side_effects(&left_child.right) {
                        // a || false || b => a || b
                        // a && true && b => a && b
                        if !right_boolean && left_child_op == LogicalOperator::Or
                            || right_boolean && left_child_op == LogicalOperator::And
                        {
                            let left = ctx.ast.move_expression(&mut left_child.left);
                            let right = ctx.ast.move_expression(&mut logical_expr.right);
                            let logic_expr = ctx.ast.expression_logical(
                                logical_expr.span,
                                left,
                                left_child_op,
                                right,
                            );
                            return Some(logic_expr);
                        }
                    }
                }
            }
        }
        None
    }

    /// Try to fold a nullish coalesce `foo ?? bar`.
    pub fn try_fold_coalesce(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        debug_assert_eq!(logical_expr.operator, LogicalOperator::Coalesce);
        let left = &logical_expr.left;
        let left_val = ValueType::from(left);
        match left_val {
            ValueType::Null | ValueType::Undefined => {
                Some(if ctx.expression_may_have_side_effects(left) {
                    // e.g. `(a(), null) ?? 1` => `(a(), null, 1)`
                    let expressions = ctx.ast.vec_from_array([
                        ctx.ast.move_expression(&mut logical_expr.left),
                        ctx.ast.move_expression(&mut logical_expr.right),
                    ]);
                    ctx.ast.expression_sequence(logical_expr.span, expressions)
                } else {
                    // nullish condition => this expression evaluates to the right side.
                    ctx.ast.move_expression(&mut logical_expr.right)
                })
            }
            ValueType::Number
            | ValueType::BigInt
            | ValueType::String
            | ValueType::Boolean
            | ValueType::Object => {
                // non-nullish condition => this expression evaluates to the left side.
                Some(ctx.ast.move_expression(&mut logical_expr.left))
            }
            ValueType::Undetermined => None,
        }
    }

    fn extract_numeric_values(e: &BinaryExpression<'a>) -> Option<(f64, f64)> {
        if let (Expression::NumericLiteral(left), Expression::NumericLiteral(right)) =
            (&e.left, &e.right)
        {
            return Some((left.value, right.value));
        }
        None
    }

    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn try_fold_binary_expr(
        e: &mut BinaryExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // TODO: tryReduceOperandsForOp

        // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L1136
        // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L1222
        let span = e.span;
        match e.operator {
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterEqualThan => Self::try_fold_comparison(e, ctx),
            BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOR | BinaryOperator::BitwiseXOR => {
                ctx.eval_binary(e).or_else(|| Self::try_fold_left_child_op(e, ctx))
            }
            BinaryOperator::Addition => Self::try_fold_add(e, ctx),
            BinaryOperator::Subtraction => {
                // Subtraction of small-ish integers can definitely be folded without issues
                Self::extract_numeric_values(e)
                    .filter(|(left, right)| {
                        left.is_nan()
                            || left.is_finite()
                            || right.is_nan()
                            || right.is_finite()
                            || (left.fract() == 0.0
                                && right.fract() == 0.0
                                && (left.abs() as usize) <= 0xFFFF_FFFF
                                && (right.abs() as usize) <= 0xFFFF_FFFF)
                    })
                    .and_then(|_| ctx.eval_binary(e))
            }
            BinaryOperator::Multiplication
            | BinaryOperator::Exponential
            | BinaryOperator::Remainder => Self::extract_numeric_values(e)
                .filter(|(left, right)| {
                    *left == 0.0
                        || left.is_nan()
                        || left.is_infinite()
                        || *right == 0.0
                        || right.is_nan()
                        || right.is_infinite()
                })
                .and_then(|_| ctx.eval_binary(e)),
            BinaryOperator::Division => Self::extract_numeric_values(e)
                .filter(|(_, right)| *right == 0.0 || right.is_nan() || right.is_infinite())
                .and_then(|_| ctx.eval_binary(e)),
            BinaryOperator::ShiftLeft => {
                if let Some((left, right)) = Self::extract_numeric_values(e) {
                    let result = ctx.eval_binary_expression(e)?.into_number()?;
                    let left_len = Self::approximate_printed_int_char_count(left);
                    let right_len = Self::approximate_printed_int_char_count(right);
                    let result_len = Self::approximate_printed_int_char_count(result);
                    if result_len <= left_len + 2 + right_len {
                        return Some(ctx.value_to_expr(span, ConstantValue::Number(result)));
                    }
                }
                None
            }
            BinaryOperator::ShiftRightZeroFill => {
                if let Some((left, right)) = Self::extract_numeric_values(e) {
                    let result = ctx.eval_binary_expression(e)?.into_number()?;
                    let left_len = Self::approximate_printed_int_char_count(left);
                    let right_len = Self::approximate_printed_int_char_count(right);
                    let result_len = Self::approximate_printed_int_char_count(result);
                    if result_len <= left_len + 3 + right_len {
                        return Some(ctx.value_to_expr(span, ConstantValue::Number(result)));
                    }
                }
                None
            }
            BinaryOperator::ShiftRight | BinaryOperator::Instanceof => ctx.eval_binary(e),
            BinaryOperator::In => None,
        }
    }

    // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L1128
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    #[must_use]
    fn approximate_printed_int_char_count(value: f64) -> usize {
        let mut count = if value.is_infinite() {
            "Infinity".len()
        } else if value.is_nan() {
            "NaN".len()
        } else {
            1 + 0.max(value.abs().log10().floor() as usize)
        };
        if value.is_sign_negative() {
            count += 1;
        }
        count
    }

    // Simplified version of `tryFoldAdd` from closure compiler.
    fn try_fold_add(e: &mut BinaryExpression<'a>, ctx: Ctx<'a, '_>) -> Option<Expression<'a>> {
        if let Some(v) = ctx.eval_binary_expression(e) {
            return Some(ctx.value_to_expr(e.span, v));
        }
        debug_assert_eq!(e.operator, BinaryOperator::Addition);
        // a + 'b' + 'c' -> a + 'bc'
        if let Expression::BinaryExpression(left_binary_expr) = &mut e.left {
            if let Expression::StringLiteral(left_str) = &left_binary_expr.right {
                if let Expression::StringLiteral(right_str) = &e.right {
                    let span = Span::new(left_str.span.start, right_str.span.end);
                    let value = left_str.value.to_string() + right_str.value.as_str();
                    let right = ctx.ast.expression_string_literal(span, value, None);
                    let left = ctx.ast.move_expression(&mut left_binary_expr.left);
                    return Some(ctx.ast.expression_binary(e.span, left, e.operator, right));
                }
            }
        }

        // typeof foo + ""
        if let Expression::UnaryExpression(left) = &e.left {
            if left.operator.is_typeof() {
                if let Expression::StringLiteral(right) = &e.right {
                    if right.value.is_empty() {
                        return Some(ctx.ast.move_expression(&mut e.left));
                    }
                }
            }
        }

        None
    }

    fn try_fold_left_child_op(
        e: &mut BinaryExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let op = e.operator;
        debug_assert!(matches!(
            op,
            BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOR | BinaryOperator::BitwiseXOR
        ));

        let Expression::BinaryExpression(left) = &mut e.left else {
            return None;
        };
        if left.operator != op {
            return None;
        }

        let (v, expr_to_move);
        if let Some(result) = ctx.eval_binary_operation(op, &left.left, &e.right) {
            (v, expr_to_move) = (result, &mut left.right);
        } else if let Some(result) = ctx.eval_binary_operation(op, &left.right, &e.right) {
            (v, expr_to_move) = (result, &mut left.left);
        } else {
            return None;
        }

        Some(ctx.ast.expression_binary(
            e.span,
            ctx.ast.move_expression(expr_to_move),
            op,
            ctx.value_to_expr(Span::new(left.right.span().start, e.right.span().end), v),
        ))
    }

    fn try_fold_comparison(e: &BinaryExpression<'a>, ctx: Ctx<'a, '_>) -> Option<Expression<'a>> {
        let left = &e.left;
        let right = &e.right;
        let op = e.operator;
        if ctx.expression_may_have_side_effects(left) || ctx.expression_may_have_side_effects(right)
        {
            return None;
        }
        let value = match op {
            BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterEqualThan => {
                return ctx.eval_binary_expression(e).map(|v| ctx.value_to_expr(e.span, v));
            }
            BinaryOperator::Equality => Self::try_abstract_equality_comparison(left, right, ctx),
            BinaryOperator::Inequality => {
                Self::try_abstract_equality_comparison(left, right, ctx).map(|b| !b)
            }
            BinaryOperator::StrictEquality => {
                Self::try_strict_equality_comparison(left, right, ctx)
            }
            BinaryOperator::StrictInequality => {
                Self::try_strict_equality_comparison(left, right, ctx).map(|b| !b)
            }
            _ => None,
        };
        let value = match value {
            Some(true) => true,
            Some(false) => false,
            None => return None,
        };
        Some(ctx.ast.expression_boolean_literal(e.span, value))
    }

    /// <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    fn try_abstract_equality_comparison(
        left_expr: &Expression<'a>,
        right_expr: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<bool> {
        let left = ValueType::from(left_expr);
        let right = ValueType::from(right_expr);
        if left != ValueType::Undetermined && right != ValueType::Undetermined {
            if left == right {
                return Self::try_strict_equality_comparison(left_expr, right_expr, ctx);
            }
            if matches!(
                (left, right),
                (ValueType::Null, ValueType::Undefined) | (ValueType::Undefined, ValueType::Null)
            ) {
                return Some(true);
            }

            if matches!((left, right), (ValueType::Number, ValueType::String))
                || matches!(right, ValueType::Boolean)
            {
                let right_number = ctx.get_side_free_number_value(right_expr);

                if let Some(num) = right_number {
                    let number_literal_expr = ctx.ast.expression_numeric_literal(
                        right_expr.span(),
                        num,
                        None,
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return Self::try_abstract_equality_comparison(
                        left_expr,
                        &number_literal_expr,
                        ctx,
                    );
                }

                return None;
            }

            if matches!((left, right), (ValueType::String, ValueType::Number))
                || matches!(left, ValueType::Boolean)
            {
                let left_number = ctx.get_side_free_number_value(left_expr);

                if let Some(num) = left_number {
                    let number_literal_expr = ctx.ast.expression_numeric_literal(
                        left_expr.span(),
                        num,
                        None,
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return Self::try_abstract_equality_comparison(
                        &number_literal_expr,
                        right_expr,
                        ctx,
                    );
                }

                return None;
            }

            if matches!(left, ValueType::BigInt) || matches!(right, ValueType::BigInt) {
                let left_bigint = ctx.get_side_free_bigint_value(left_expr);
                let right_bigint = ctx.get_side_free_bigint_value(right_expr);

                if let (Some(l_big), Some(r_big)) = (left_bigint, right_bigint) {
                    return Some(l_big.eq(&r_big));
                }
            }

            if matches!(left, ValueType::String | ValueType::Number | ValueType::BigInt)
                && matches!(right, ValueType::Object)
            {
                return None;
            }

            if matches!(left, ValueType::Object)
                && matches!(right, ValueType::String | ValueType::Number | ValueType::BigInt)
            {
                return None;
            }

            return Some(false);
        }
        None
    }

    /// <https://tc39.es/ecma262/#sec-strict-equality-comparison>
    #[expect(clippy::float_cmp)]
    fn try_strict_equality_comparison(
        left_expr: &Expression<'a>,
        right_expr: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<bool> {
        let left = ValueType::from(left_expr);
        let right = ValueType::from(right_expr);
        if !left.is_undetermined() && !right.is_undetermined() {
            // Strict equality can only be true for values of the same type.
            if left != right {
                return Some(false);
            }
            return match left {
                ValueType::Number => {
                    let lnum = ctx.get_side_free_number_value(left_expr)?;
                    let rnum = ctx.get_side_free_number_value(right_expr)?;
                    if lnum.is_nan() || rnum.is_nan() {
                        return Some(false);
                    }
                    Some(lnum == rnum)
                }
                ValueType::String => {
                    let left = ctx.get_side_free_string_value(left_expr)?;
                    let right = ctx.get_side_free_string_value(right_expr)?;
                    Some(left == right)
                }
                ValueType::Undefined | ValueType::Null => Some(true),
                ValueType::Boolean if right.is_boolean() => {
                    let left = ctx.get_boolean_value(left_expr)?;
                    let right = ctx.get_boolean_value(right_expr)?;
                    Some(left == right)
                }
                ValueType::BigInt => {
                    let left = ctx.get_side_free_bigint_value(left_expr)?;
                    let right = ctx.get_side_free_bigint_value(right_expr)?;
                    Some(left == right)
                }
                ValueType::Object | ValueType::Boolean | ValueType::Undetermined => None,
            };
        }

        // Then, try to evaluate based on the value of the expression.
        // There's only one special case:
        // Any strict equality comparison against NaN returns false.
        if left_expr.is_nan() || right_expr.is_nan() {
            return Some(false);
        }
        None
    }

    fn try_fold_number_constructor(
        e: &CallExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let Expression::Identifier(ident) = &e.callee else { return None };
        if ident.name != "Number" {
            return None;
        }
        if !ctx.is_global_reference(ident) {
            return None;
        }
        if e.arguments.len() != 1 {
            return None;
        }
        let arg = e.arguments[0].as_expression()?;
        let value = ConstantValue::Number(match arg {
            // `Number(undefined)` -> `NaN`
            Expression::Identifier(ident) if ctx.is_identifier_undefined(ident) => f64::NAN,
            // `Number(null)` -> `0`
            Expression::NullLiteral(_) => 0.0,
            // `Number(true)` -> `1` `Number(false)` -> `0`
            Expression::BooleanLiteral(b) => f64::from(b.value),
            // `Number(100)` -> `100`
            Expression::NumericLiteral(n) => n.value,
            // `Number("a")` -> `+"a"` -> `NaN`
            // `Number("1")` -> `+"1"` -> `1`
            Expression::StringLiteral(n) => {
                let argument = ctx.ast.expression_string_literal(n.span, n.value, n.raw);
                if let Some(n) = ctx.eval_to_number(&argument) {
                    n
                } else {
                    return Some(ctx.ast.expression_unary(
                        e.span,
                        UnaryOperator::UnaryPlus,
                        argument,
                    ));
                }
            }
            e if e.is_void_0() => f64::NAN,
            _ => return None,
        });
        Some(ctx.value_to_expr(e.span, value))
    }

    fn try_fold_binary_typeof_comparison(
        bin_expr: &mut BinaryExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // `typeof a === typeof b` -> `typeof a == typeof b`, `typeof a != typeof b` -> `typeof a != typeof b`,
        // `typeof a == typeof a` -> `true`, `typeof a != typeof a` -> `false`
        if bin_expr.operator.is_equality() {
            if let (Expression::UnaryExpression(left), Expression::UnaryExpression(right)) =
                (&bin_expr.left, &bin_expr.right)
            {
                if left.operator.is_typeof() && right.operator.is_typeof() {
                    if let (
                        Expression::Identifier(left_ident),
                        Expression::Identifier(right_ident),
                    ) = (&left.argument, &right.argument)
                    {
                        if left_ident.name == right_ident.name {
                            return Some(ctx.ast.expression_boolean_literal(
                                bin_expr.span,
                                matches!(
                                    bin_expr.operator,
                                    BinaryOperator::StrictEquality | BinaryOperator::Equality
                                ),
                            ));
                        }
                    }

                    if matches!(
                        bin_expr.operator,
                        BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
                    ) {
                        return Some(ctx.ast.expression_binary(
                            bin_expr.span,
                            ctx.ast.move_expression(&mut bin_expr.left),
                            if bin_expr.operator == BinaryOperator::StrictEquality {
                                BinaryOperator::Equality
                            } else {
                                BinaryOperator::Inequality
                            },
                            ctx.ast.move_expression(&mut bin_expr.right),
                        ));
                    }
                }
            }
        }

        // `typeof a === 'asd` -> `false``
        // `typeof a !== 'b'` -> `true``
        if let Expression::UnaryExpression(left) = &bin_expr.left {
            if left.operator.is_typeof() && bin_expr.operator.is_equality() {
                let right_ty = ValueType::from(&bin_expr.right);

                if !right_ty.is_undetermined() && right_ty != ValueType::String {
                    return Some(ctx.ast.expression_boolean_literal(
                        bin_expr.span,
                        bin_expr.operator == BinaryOperator::Inequality
                            || bin_expr.operator == BinaryOperator::StrictInequality,
                    ));
                }
                if let Expression::StringLiteral(string_lit) = &bin_expr.right {
                    if !matches!(
                        string_lit.value.as_str(),
                        "string"
                            | "number"
                            | "bigint"
                            | "boolean"
                            | "symbol"
                            | "undefined"
                            | "object"
                            | "function"
                            | "unknown" // IE
                    ) {
                        return Some(ctx.ast.expression_boolean_literal(
                            bin_expr.span,
                            bin_expr.operator == BinaryOperator::Inequality
                                || bin_expr.operator == BinaryOperator::StrictInequality,
                        ));
                    }
                }
            }
        }

        None
    }

    fn fold_object_spread(
        &mut self,
        e: &mut ObjectExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let len = e.properties.len();
        e.properties.retain(|p| {
            if let ObjectPropertyKind::SpreadProperty(spread_element) = p {
                let e = &spread_element.argument;
                if e.is_literal() || ctx.is_expression_undefined(e) {
                    return false;
                }
                if let Expression::ObjectExpression(o) = e {
                    if o.properties.is_empty() {
                        return false;
                    }
                }
                if let Expression::ArrayExpression(o) = e {
                    if o.elements.is_empty() {
                        return false;
                    }
                }
                return true;
            }
            true
        });
        if e.properties.len() != len {
            self.mark_current_function_as_changed();
        }
        None
    }
}

/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>
#[cfg(test)]
mod test {
    static MAX_SAFE_FLOAT: f64 = 9_007_199_254_740_991_f64;
    static NEG_MAX_SAFE_FLOAT: f64 = -9_007_199_254_740_991_f64;

    static MAX_SAFE_INT: i64 = 9_007_199_254_740_991_i64;
    static NEG_MAX_SAFE_INT: i64 = -9_007_199_254_740_991_i64;

    use crate::tester::test;

    // wrap with a function call so it doesn't get removed.
    fn fold(source_text: &str, expected: &str) {
        let source_text = format!("NOOP({source_text})");
        let expected = format!("NOOP({expected})");
        test(&source_text, &expected);
    }

    fn fold_same(source_text: &str) {
        fold(source_text, source_text);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    #[test]
    fn test_comparison() {
        fold("(1, 2) !== 2", "!1");
        fold_same("({} <= {})");
        fold_same("({} >= {})");
        fold_same("({} > {})");
        fold_same("({} < {})");
        fold_same("([] <= [])");
        fold_same("([] >= [])");
        fold_same("([] > [])");
        fold_same("([] < [])");
    }

    #[test]
    fn undefined_comparison1() {
        fold("undefined == undefined", "!0");
        fold("undefined == null", "!0");
        fold("undefined == void 0", "!0");

        fold("undefined == 0", "!1");
        fold("undefined == 1", "!1");
        fold("undefined == 'hi'", "!1");
        fold("undefined == true", "!1");
        fold("undefined == false", "!1");

        fold("undefined === undefined", "!0");
        fold("undefined === null", "!1");
        fold("undefined === void 0", "!0");

        fold("undefined == this", "this == null");
        fold("undefined == x", "x == null");

        fold("undefined != undefined", "!1");
        fold("undefined != null", "!1");
        fold("undefined != void 0", "!1");

        fold("undefined != 0", "!0");
        fold("undefined != 1", "!0");
        fold("undefined != 'hi'", "!0");
        fold("undefined != true", "!0");
        fold("undefined != false", "!0");

        fold("undefined !== undefined", "!1");
        fold("undefined !== void 0", "!1");
        fold("undefined !== null", "!0");

        fold("undefined != this", "this != null");
        fold("undefined != x", "x != null");

        fold("undefined < undefined", "!1");
        fold("undefined > undefined", "!1");
        fold("undefined >= undefined", "!1");
        fold("undefined <= undefined", "!1");

        fold("0 < undefined", "!1");
        fold("true > undefined", "!1");
        fold("'hi' >= undefined", "!1");
        fold("null <= undefined", "!1");

        fold("undefined < 0", "!1");
        fold("undefined > true", "!1");
        fold("undefined >= 'hi'", "!1");
        fold("undefined <= null", "!1");

        fold("null == undefined", "!0");
        fold("0 == undefined", "!1");
        fold("1 == undefined", "!1");
        fold("'hi' == undefined", "!1");
        fold("true == undefined", "!1");
        fold("false == undefined", "!1");
        fold("null === undefined", "!1");
        fold("void 0 === undefined", "!0");

        fold("undefined == NaN", "!1");
        fold("NaN == undefined", "!1");
        fold("undefined == Infinity", "!1");
        fold("Infinity == undefined", "!1");
        fold("undefined == -Infinity", "!1");
        fold("-Infinity == undefined", "!1");
        fold("({}) == undefined", "!1");
        fold("undefined == ({})", "!1");
        fold("([]) == undefined", "!1");
        fold("undefined == ([])", "!1");
        fold("(/a/g) == undefined", "!1");
        fold("undefined == (/a/g)", "!1");
        fold("(function(){}) == undefined", "!1");
        fold("undefined == (function(){})", "!1");

        fold("undefined != NaN", "!0");
        fold("NaN != undefined", "!0");
        fold("undefined != Infinity", "!0");
        fold("Infinity != undefined", "!0");
        fold("undefined != -Infinity", "!0");
        fold("-Infinity != undefined", "!0");
        fold("({}) != undefined", "!0");
        fold("undefined != ({})", "!0");
        fold("([]) != undefined", "!0");
        fold("undefined != ([])", "!0");
        fold("(/a/g) != undefined", "!0");
        fold("undefined != (/a/g)", "!0");
        fold("(function(){}) != undefined", "!0");
        fold("undefined != (function(){})", "!0");

        fold("this == undefined", "this == null");
        fold("x == undefined", "x == null");
    }

    #[test]
    fn test_undefined_comparison2() {
        fold("\"123\" !== void 0", "!0");
        fold("\"123\" === void 0", "!1");

        fold("void 0 !== \"123\"", "!0");
        fold("void 0 === \"123\"", "!1");
    }

    #[test]
    fn test_undefined_comparison3() {
        fold("\"123\" !== undefined", "!0");
        fold("\"123\" === undefined", "!1");

        fold("undefined !== \"123\"", "!0");
        fold("undefined === \"123\"", "!1");
    }

    #[test]
    fn test_null_comparison1() {
        fold("null == undefined", "!0");
        fold("null == null", "!0");
        fold("null == void 0", "!0");

        fold("null == 0", "!1");
        fold("null == 1", "!1");
        // test("null == 0n", "!1");
        // test("null == 1n", "!1");
        fold("null == 'hi'", "!1");
        fold("null == true", "!1");
        fold("null == false", "!1");

        fold("null === undefined", "!1");
        fold("null === null", "!0");
        fold("null === void 0", "!1");
        fold_same("x===null");

        fold_same("this==null");
        fold_same("x==null");

        fold("null != undefined", "!1");
        fold("null != null", "!1");
        fold("null != void 0", "!1");

        fold("null != 0", "!0");
        fold("null != 1", "!0");
        // test("null != 0n", "!0");
        // test("null != 1n", "!0");
        fold("null != 'hi'", "!0");
        fold("null != true", "!0");
        fold("null != false", "!0");

        fold("null !== undefined", "!0");
        fold("null !== void 0", "!0");
        fold("null !== null", "!1");

        fold_same("this!=null");
        fold_same("x!=null");

        fold("null < null", "!1");
        fold("null > null", "!1");
        fold("null >= null", "!0");
        fold("null <= null", "!0");

        fold("0 < null", "!1");
        fold("0 > null", "!1");
        fold("0 >= null", "!0");
        // test("0n < null", "!1");
        // test("0n > null", "!1");
        // test("0n >= null", "!0");
        fold("true > null", "!0");
        fold("'hi' < null", "!1");
        fold("'hi' >= null", "!1");
        fold("null <= null", "!0");

        fold("null < 0", "!1");
        // test("null < 0n", "!1");
        fold("null > true", "!1");
        fold("null < 'hi'", "!1");
        fold("null >= 'hi'", "!1");
        fold("null <= null", "!0");

        fold("null == null", "!0");
        fold("0 == null", "!1");
        fold("1 == null", "!1");
        fold("'hi' == null", "!1");
        fold("true == null", "!1");
        fold("false == null", "!1");
        fold("null === null", "!0");
        fold("void 0 === null", "!1");

        fold("null == NaN", "!1");
        fold("NaN == null", "!1");
        fold("null == Infinity", "!1");
        fold("Infinity == null", "!1");
        fold("null == -Infinity", "!1");
        fold("-Infinity == null", "!1");
        fold("({}) == null", "!1");
        fold("null == ({})", "!1");
        fold("([]) == null", "!1");
        fold("null == ([])", "!1");
        fold("(/a/g) == null", "!1");
        fold("null == (/a/g)", "!1");
        fold("(function(){}) == null", "!1");
        fold("null == (function(){})", "!1");

        fold("null != NaN", "!0");
        fold("NaN != null", "!0");
        fold("null != Infinity", "!0");
        fold("Infinity != null", "!0");
        fold("null != -Infinity", "!0");
        fold("-Infinity != null", "!0");
        fold("({}) != null", "!0");
        fold("null != ({})", "!0");
        fold("([]) != null", "!0");
        fold("null != ([])", "!0");
        fold("(/a/g) != null", "!0");
        fold("null != (/a/g)", "!0");
        fold("(function(){}) != null", "!0");
        fold("null != (function(){})", "!0");

        fold_same("({a:f()})==null");
        fold_same("[f()]==null");

        fold_same("this==null");
        fold_same("x==null");
    }

    #[test]
    fn test_boolean_boolean_comparison() {
        fold_same("!x == !y");
        fold_same("!x < !y");
        fold("!x!==!y", "!x != !y");

        fold_same("!x == !x"); // foldable
        fold_same("!x <! x"); // foldable
        fold("!x !== !x", "!x != !x"); // foldable
    }

    #[test]
    fn test_boolean_number_comparison() {
        fold_same("!x==+y");
        fold_same("!x<=+y");
        fold_same("!x !== +y");
    }

    #[test]
    fn test_number_boolean_comparison() {
        fold_same("+x==!y");
        fold_same("+x<=!y");
        fold_same("+x === !y");
    }

    #[test]
    fn test_boolean_string_comparison() {
        fold_same("!x==''+y");
        fold_same("!x<=''+y");
        fold_same("!x !== '' + y");
    }

    #[test]
    fn test_string_boolean_comparison() {
        fold_same("''+x==!y");
        fold_same("''+x<=!y");
        fold_same("'' + x === !y");
    }

    #[test]
    fn test_number_number_comparison() {
        fold("1 > 1", "!1");
        fold("2 == 3", "!1");
        fold("3.6 === 3.6", "!0");
        fold_same("+x > +y");
        fold_same("+x == +y");
        fold("+x === +y", "+x == +y");
        fold_same("+x > +x"); // foldable to false
        fold_same("+x == +x");
        fold("+x === +x", "+x == +x");
    }

    #[test]
    fn test_string_string_comparison() {
        fold("'a' < 'b'", "!0");
        fold("'a' <= 'b'", "!0");
        fold("'a' > 'b'", "!1");
        fold("'a' >= 'b'", "!1");
        fold("+'a' < +'b'", "!1");
        fold_same("typeof a < 'a'");
        fold_same("'a' >= typeof a");
        fold_same("typeof a < typeof a");
        fold_same("typeof a >= typeof a");
        fold("typeof 3 > typeof 4", "!1");
        fold("typeof function() {} < typeof function() {}", "!1");
        fold("'a' == 'a'", "!0");
        fold("'b' != 'a'", "!0");
        fold_same("typeof a != 'number'");
        fold_same("typeof a != 'unknown'"); // IE
        fold("'a' === 'a'", "!0");
        fold("'b' !== 'a'", "!0");
        fold_same("'' + x <= '' + y");
        fold_same("'' + x != '' + y");
        fold("'' + x === '' + y", "'' + x == '' + y");

        fold_same("'' + x <= '' + x"); // potentially foldable
        fold_same("'' + x != '' + x"); // potentially foldable
        fold("'' + x === '' + x", "'' + x == '' + x"); // potentially foldable

        test(r#"if ("string" !== "\u000Bstr\u000Bing\u000B") {}"#, "");
    }

    #[test]
    fn test_number_string_comparison() {
        fold("1 < '2'", "!0");
        fold("2 > '1'", "!0");
        fold("123 > '34'", "!0");
        fold("NaN >= 'NaN'", "!1");
        fold("1 == '2'", "!1");
        fold("1 != '1'", "!1");
        fold("NaN == 'NaN'", "!1");
        fold("1 === '1'", "!1");
        fold("1 !== '1'", "!0");
        fold_same("+x>''+y");
        fold_same("+x==''+y");
        fold_same("+x !== '' + y");
    }

    #[test]
    fn test_string_number_comparison() {
        fold("'1' < 2", "!0");
        fold("'2' > 1", "!0");
        fold("'123' > 34", "!0");
        fold("'NaN' < NaN", "!1");
        fold("'1' == 2", "!1");
        fold("'1' != 1", "!1");
        fold("'NaN' == NaN", "!1");
        fold("'1' === 1", "!1");
        fold("'1' !== 1", "!0");
        fold_same("''+x<+y");
        fold_same("''+x==+y");
        fold_same("'' + x === +y");
    }

    #[test]
    fn test_nan_comparison() {
        fold("NaN < 1", "!1");
        fold("NaN <= 1", "!1");
        fold("NaN > 1", "!1");
        fold("NaN >= 1", "!1");
        // test("NaN < 1n", "!1");
        // test("NaN <= 1n", "!1");
        // test("NaN > 1n", "!1");
        // test("NaN >= 1n", "!1");

        fold("NaN < NaN", "!1");
        fold("NaN >= NaN", "!1");
        fold("NaN == NaN", "!1");
        fold("NaN === NaN", "!1");

        fold("NaN < null", "!1");
        fold("null >= NaN", "!1");
        fold("NaN == null", "!1");
        fold("null != NaN", "!0");
        fold("null === NaN", "!1");

        fold("NaN < undefined", "!1");
        fold("undefined >= NaN", "!1");
        fold("NaN == undefined", "!1");
        fold("undefined != NaN", "!0");
        fold("undefined === NaN", "!1");

        fold_same("NaN<x");
        fold_same("x>=NaN");
        fold("NaN==x", "x==NaN");
        fold_same("x!=NaN");
        fold("NaN === x", "x === NaN");
        fold_same("x !== NaN");
        fold("NaN==foo()", "foo()==NaN");
    }

    #[test]
    #[ignore]
    fn test_object_comparison1() {
        fold("!new Date()", "false");
        fold("!!new Date()", "true");

        fold("new Date() == null", "false");
        fold("new Date() == undefined", "false");
        fold("new Date() != null", "true");
        fold("new Date() != undefined", "true");
        fold("null == new Date()", "false");
        fold("undefined == new Date()", "false");
        fold("null != new Date()", "true");
        fold("undefined != new Date()", "true");
    }

    #[test]
    fn js_typeof() {
        fold("x = typeof 1", "x = \"number\"");
        fold("x = typeof 'foo'", "x = \"string\"");
        fold("x = typeof true", "x = \"boolean\"");
        fold("x = typeof false", "x = \"boolean\"");
        fold("x = typeof null", "x = \"object\"");
        fold("x = typeof undefined", "x = \"undefined\"");
        fold("x = typeof void 0", "x = \"undefined\"");
        fold("x = typeof []", "x = \"object\"");
        fold("x = typeof [1]", "x = \"object\"");
        fold("x = typeof [1,[]]", "x = \"object\"");
        fold("x = typeof {}", "x = \"object\"");
        fold("x = typeof function() {}", "x = 'function'");

        fold_same("x = typeof[1,[foo()]]");
        fold_same("x = typeof{bathwater:baby()}");
    }

    #[test]
    fn test_fold_unary() {
        fold_same("!foo()");
        fold_same("~foo()");
        fold_same("-foo()");

        fold("a=!true", "a=!1");
        fold("a=!10", "a=!1");
        fold("a=!false", "a=!0");
        fold_same("a=!foo()");
        fold_same("a = !!void b");

        fold("a=-0", "a=-0");
        fold("a=-(0)", "a=-0");
        fold_same("a=-Infinity");
        fold("a=-NaN", "a=NaN");
        fold_same("a=-foo()");
        fold("-undefined", "NaN");
        fold("-null", "-0");
        fold("-NaN", "NaN");

        fold("a=+true", "a=1");
        fold("a=+10", "a=10");
        fold("a=+false", "a=0");
        fold_same("a=+foo()");
        fold_same("a=+f");
        fold("a=+(f?true:false)", "a=+!!f");
        fold("a=+0", "a=0");
        fold("a=+Infinity", "a=Infinity");
        fold("a=+NaN", "a=NaN");
        fold("a=+-7", "a=-7");
        fold("a=+.5", "a=.5");

        fold("a=~~0", "a=0");
        fold("a=~~10", "a=10");
        fold("a=~-7", "a=6");
        fold_same("a=~~foo()");
        fold("a=~0xffffffff", "a=0");
        fold("a=~~0xffffffff", "a=-1");
        fold("a=~.5", "a=-1");
    }

    #[test]
    fn test_fold_unary_big_int() {
        fold("-(1n)", "-1n");
        fold("- -1n", "1n");
        fold("!1n", "!1");
        fold("~0n", "-1n");

        fold("~-1n", "0n");
        fold("~~1n", "1n");

        fold("~0x3n", "-4n");
        fold("~0b11n", "-4n");
    }

    #[test]
    fn test_unary_ops_string_compare() {
        fold_same("a = -1");
        fold("a = ~0", "a = -1");
        fold("a = ~1", "a = -2");
        fold("a = ~101", "a = -102");

        fold("a = ~1.1", "a = -2");
        fold("a = ~0x3", "a = -4"); // Hexadecimal number
        fold("a = ~9", "a = -10"); // Despite `-10` is longer than `~9`, the compiler still folds it.
        fold_same("a = ~b");
        fold("a = ~NaN", "a = -1");
        fold("a = ~-Infinity", "a = -1");
        fold("x = ~2147483658.0", "x = 2147483637");
        fold("x = ~-2147483658", "x = -2147483639");
    }

    #[test]
    fn test_fold_logical_op() {
        fold("x = true && x", "x = x");
        fold("x = [foo()] && x", "x = ([foo()],x)");

        fold("x = false && x", "x = !1");
        fold("x = true || x", "x = !0");
        fold("x = false || x", "x = x");
        fold("x = 0 && x", "x = 0");
        fold("x = 3 || x", "x = 3");
        fold("x = 0n && x", "x = 0n");
        fold("x = 3n || x", "x = 3n");
        fold("x = false || 0", "x = 0");

        // unfoldable, because the right-side may be the result
        fold("a = x && true", "a=x && !0");
        fold("a = x && false", "a=x && !1");
        fold("a = x || 3", "a=x || 3");
        fold("a = x || false", "a=x || !1");
        fold("a = b ? c : x || false", "a=b ? c : x || !1");
        fold("a = b ? x || false : c", "a=b ? x || !1 : c");
        fold("a = b ? c : x && true", "a=b ? c : x && !0");
        fold("a = b ? x && true : c", "a=b ? x && !0 : c");

        fold("a = x || false ? b : c", "a = x ? b : c");
        fold("a = x && true ? b : c", "a = x ? b : c");

        fold("x = foo() || true || bar()", "x = foo() || !0");
        fold("x = foo() || true && bar()", "x = foo() || bar()");
        fold("x = foo() || false && bar()", "x = foo() || !1");
        fold("x = foo() && false && bar()", "x = foo() && !1");
        fold("x = foo() && false || bar()", "x = (foo() && !1, bar())");
        fold("x = foo() || false || bar()", "x = foo() || bar()");
        fold("x = foo() && true && bar()", "x = foo() && bar()");
        fold("x = foo() || true || bar()", "x = foo() || !0");
        fold("x = foo() && false && bar()", "x = foo() && !1");
        fold("x = foo() && 0 && bar()", "x = foo() && 0");
        fold("x = foo() && 1 && bar()", "x = foo() && bar()");
        fold("x = foo() || 0 || bar()", "x = foo() || bar()");
        fold("x = foo() || 1 || bar()", "x = foo() || 1");
        fold("x = foo() && 0n && bar()", "x = foo() && 0n");
        fold("x = foo() && 1n && bar()", "x = foo() && bar()");
        fold("x = foo() || 0n || bar()", "x = foo() || bar()");
        fold("x = foo() || 1n || bar()", "x = foo() || 1n");
        fold_same("x = foo() || bar() || baz()");
        fold_same("x = foo() && bar() && baz()");

        fold("0 || b()", "b()");
        fold("1 && b()", "b()");
        fold("a() && (1 && b())", "a() && b()");
        fold("(a() && 1) && b()", "a() && b()");

        fold("(x || '') || y", "x || y");
        fold("false || (x || '')", "x || ''");
        fold("(x && 1) && y", "x && y");
        fold("true && (x && 1)", "x && 1");

        // Really not foldable, because it would change the type of the
        // expression if foo() returns something truthy but not true.
        // Cf. FoldConstants.tryFoldAndOr().
        // An example would be if foo() is 1 (truthy) and bar() is 0 (falsey):
        // (1 && true) || 0 == true
        // 1 || 0 == 1, but true =/= 1
        fold("x = foo() && true || bar()", "x = foo() && !0 || bar()");
        fold("foo() && true || bar()", "foo() && !0 || bar()");
    }

    #[test]
    fn test_fold_logical_op2() {
        fold("x = function(){} && x", "x=x");
        fold("x = true && function(){}", "x=function(){}");
        fold("x = [(function(){alert(x)})()] && x", "x=([function(){alert(x)}()],x)");
    }

    #[test]
    fn test_fold_nullish_coalesce() {
        // fold if left is null/undefined
        fold("null ?? 1", "1");
        fold("undefined ?? false", "!1");
        fold("(a(), null) ?? 1", "(a(), null, 1)");

        fold("x = [foo()] ?? x", "x = [foo()]");

        // short circuit on all non nullish LHS
        fold("x = false ?? x", "x = !1");
        fold("x = true ?? x", "x = !0");
        fold("x = 0 ?? x", "x = 0");
        fold("x = 3 ?? x", "x = 3");

        // unfoldable, because the right-side may be the result
        fold("a = x ?? true", "a = x ?? !0");
        fold("a = x ?? false", "a = x ?? !1");
        fold_same("a = x ?? 3");
        fold("a = b ? c : x ?? false", "a = b ? c : x ?? !1");
        fold("a = b ? x ?? false : c", "a = b ? x ?? !1 : c");

        // folded, but not here.
        fold("a = x ?? false ? b : c", "a = x ?? !1 ? b : c");
        fold("a = x ?? true ? b : c", "a = x ?? !0 ? b : c");

        fold("x = foo() ?? true ?? bar()", "x = foo() ?? !0 ?? bar()");
        fold("x = foo() ?? (true && bar())", "x = foo() ?? bar()");
        fold("x = (foo() || false) ?? bar()", "x = (foo() || !1) ?? bar()");

        fold("a() ?? (1 ?? b())", "a() ?? 1");
        fold("(a() ?? 1) ?? b()", "a() ?? 1 ?? b()");
    }

    #[test]
    fn test_fold_void() {
        fold_same("void 0");
        fold("void 1", "void 0");
        fold_same("void x");
        fold_same("void x()");
    }

    #[test]
    fn test_fold_opt_chain() {
        // can't fold when optional part may execute
        fold_same("a = x?.y");
        fold_same("a = x?.()");

        // fold args of optional call
        fold("x = foo() ?. (true && bar())", "x = foo() ?.(bar())");
        fold("a() ?. (1 ?? b())", "a() ?. (1)");

        // test("({a})?.a.b.c.d()?.x.y.z", "a.b.c.d()?.x.y.z");

        fold("x = undefined?.y", "x = void 0");
        fold("x = null?.y", "x = void 0");
        fold("x = undefined?.[foo]", "x = void 0");
        fold("x = null?.[foo]", "x = void 0");
    }

    #[test]
    fn test_fold_bitwise_op() {
        fold("x = 1 & 1", "x = 1");
        fold("x = 1 & 2", "x = 0");
        fold("x = 3 & 1", "x = 1");
        fold("x = 3 & 3", "x = 3");

        fold("x = 1 | 1", "x = 1");
        fold("x = 1 | 2", "x = 3");
        fold("x = 3 | 1", "x = 3");
        fold("x = 3 | 3", "x = 3");

        fold("x = 1 ^ 1", "x = 0");
        fold("x = 1 ^ 2", "x = 3");
        fold("x = 3 ^ 1", "x = 2");
        fold("x = 3 ^ 3", "x = 0");

        fold("x = -1 & 0", "x = 0");
        fold("x = 0 & -1", "x = 0");
        fold("x = 1 & 4", "x = 0");
        fold("x = 2 & 3", "x = 2");

        // make sure we fold only when we are supposed to -- not when doing so would
        // lose information or when it is performed on nonsensical arguments.
        fold("x = 1 & 1.1", "x = 1");
        fold("x = 1.1 & 1", "x = 1");
        fold("x = 1 & 3000000000", "x = 0");
        fold("x = 3000000000 & 1", "x = 0");

        // Try some cases with | as well
        fold("x = 1 | 4", "x = 5");
        fold("x = 1 | 3", "x = 3");
        fold("x = 1 | 1.1", "x = 1");
        // test_same("x = 1 | 3e9");

        // these cases look strange because bitwise OR converts unsigned numbers to be signed
        fold("x = 1 | 3000000001", "x = -1294967295");
        fold("x = 4294967295 | 0", "x = -1");

        fold("x = -1 | 0", "x = -1");
    }

    #[test]
    fn test_fold_bitwise_op2() {
        fold("x = y & 1 & 1", "x = y & 1");
        fold("x = y & 1 & 2", "x = y & 0");
        fold("x = y & 3 & 1", "x = y & 1");
        fold("x = 3 & y & 1", "x = y & 1");
        fold("x = y & 3 & 3", "x = y & 3");
        fold("x = 3 & y & 3", "x = y & 3");

        fold("x = y | 1 | 1", "x = y | 1");
        fold("x = y | 1 | 2", "x = y | 3");
        fold("x = y | 3 | 1", "x = y | 3");
        fold("x = 3 | y | 1", "x = y | 3");
        fold("x = y | 3 | 3", "x = y | 3");
        fold("x = 3 | y | 3", "x = y | 3");

        fold("x = y ^ 1 ^ 1", "x = y ^ 0");
        fold("x = y ^ 1 ^ 2", "x = y ^ 3");
        fold("x = y ^ 3 ^ 1", "x = y ^ 2");
        fold("x = 3 ^ y ^ 1", "x = y ^ 2");
        fold("x = y ^ 3 ^ 3", "x = y ^ 0");
        fold("x = 3 ^ y ^ 3", "x = y ^ 0");

        fold("x = Infinity | NaN", "x=0");
        fold("x = 12 | NaN", "x=12");
    }

    #[test]
    fn test_fold_bitwise_op_additional() {
        fold("x = null & 1", "x = 0");
        fold_same("x = (2 ** 31 - 1) | 1");
        fold_same("x = (2 ** 31) | 1");

        // https://github.com/oxc-project/oxc/issues/7944
        fold_same("(x - 1) & 1");
        fold_same("(y >> 3) & 7");
        fold("(y & 3) & 7", "y & 3");
        fold_same("(y | 3) & 7");
        fold("y | 3 & 7", "y | 3");
    }

    #[test]
    fn test_fold_bitwise_not() {
        fold("~undefined", "-1");
        fold("~null", "-1");
        fold("~false", "-1");
        fold("~true", "-2");
        fold("~'1'", "-2");
        fold("~'-1'", "0");
        fold("~{}", "-1");
    }

    #[test]
    fn test_fold_bit_shifts() {
        fold("x = 1 << 0", "x=1");
        fold("x = -1 << 0", "x=-1");
        fold("x = 1 << 1", "x=2");
        fold("x = 3 << 1", "x=6");
        fold("x = 1 << 8", "x=256");

        fold("x = 1 >> 0", "x=1");
        fold("x = -1 >> 0", "x=-1");
        fold("x = 1 >> 1", "x=0");
        fold("x = 2 >> 1", "x=1");
        fold("x = 5 >> 1", "x=2");
        fold("x = 127 >> 3", "x=15");
        fold("x = 3 >> 1", "x=1");
        fold("x = 3 >> 2", "x=0");
        fold("x = 10 >> 1", "x=5");
        fold("x = 10 >> 2", "x=2");
        fold("x = 10 >> 5", "x=0");

        fold("x = 10 >>> 1", "x=5");
        fold("x = 10 >>> 2", "x=2");
        fold("x = 10 >>> 5", "x=0");
        fold_same("x = -1 >>> 1");
        fold_same("x = -1 >>> 0");
        fold_same("x = -2 >>> 0");
        fold("x = 0x90000000 >>> 28", "x=9");

        fold("x = 0xffffffff << 0", "x=-1");
        fold("x = 0xffffffff << 4", "x=-16");
        fold("1 << 32", "1");
        fold("1 << -1", "1<<-1");
        fold("1 >> 32", "1");

        // Regression on #6161, ported from <https://github.com/tc39/test262/blob/05c45a4c430ab6fee3e0c7f0d47d8a30d8876a6d/test/language/expressions/unsigned-right-shift/S9.6_A2.2.js>.
        fold("-2147483647 >>> 0", "2147483649");
        fold("-2147483648 >>> 0", "2147483648");
        fold("-2147483649 >>> 0", "2147483647");
        fold("-4294967295 >>> 0", "1");
        fold("-4294967296 >>> 0", "0");
        fold("-4294967297 >>> 0", "4294967295");
        fold("4294967295 >>> 0", "4294967295");
        fold("4294967296 >>> 0", "0");
        fold("4294967297 >>> 0", "1");
        fold("8589934591 >>> 0", "4294967295");
        fold("8589934592 >>> 0", "0");
        fold("8589934593 >>> 0", "1");

        fold("x = -1 << 1", "x = -2");
        fold("x = -1 << 8", "x = -256");
        fold("x = -1 >> 1", "x = -1");
        fold("x = -2 >> 1", "x = -1");
        fold("x = -1 >> 0", "x = -1");
    }

    #[test]
    fn test_string_add() {
        fold("x = 'a' + 'bc'", "x = 'abc'");
        fold("x = 'a' + 5", "x = 'a5'");
        fold("x = 5 + 'a'", "x = '5a'");
        fold("x = 'a' + 5n", "x = 'a5'");
        fold("x = 5n + 'a'", "x = '5a'");
        fold("x = 'a' + ''", "x = 'a'");
        fold("x = 'a' + foo()", "x = 'a'+foo()");
        fold("x = foo() + 'a' + 'b'", "x = foo()+'ab'");
        fold("x = (foo() + 'a') + 'b'", "x = foo()+'ab'"); // believe it!
        fold("x = foo() + 'a' + 'b' + 'cd' + bar()", "x = foo()+'abcd'+bar()");
        fold("x = foo() + 2 + 'b'", "x = foo()+2+\"b\""); // don't fold!

        // fold("x = foo() + 'a' + 2", "x = foo()+\"a2\"");
        fold("x = '' + null", "x = 'null'");
        fold("x = true + '' + false", "x = 'truefalse'");
        // fold("x = '' + []", "x = ''");
        // fold("x = foo() + 'a' + 1 + 1", "x = foo() + 'a11'");
        fold("x = 1 + 1 + 'a'", "x = '2a'");
        fold("x = 1 + 1 + 'a'", "x = '2a'");
        fold("x = 'a' + (1 + 1)", "x = 'a2'");
        // fold("x = '_' + p1 + '_' + ('' + p2)", "x = '_' + p1 + '_' + p2");
        // fold("x = 'a' + ('_' + 1 + 1)", "x = 'a_11'");
        // fold("x = 'a' + ('_' + 1) + 1", "x = 'a_11'");
        // fold("x = 1 + (p1 + '_') + ('' + p2)", "x = 1 + (p1 + '_') + p2");
        // fold("x = 1 + p1 + '_' + ('' + p2)", "x = 1 + p1 + '_' + p2");
        // fold("x = 1 + 'a' + p1", "x = '1a' + p1");
        // fold("x = (p1 + (p2 + 'a')) + 'b'", "x = (p1 + (p2 + 'ab'))");
        // fold("'a' + ('b' + p1) + 1", "'ab' + p1 + 1");
        // fold("x = 'a' + ('b' + p1 + 'c')", "x = 'ab' + (p1 + 'c')");
        fold("void 0 + ''", "'undefined'");

        fold_same("x = 'a' + (4 + p1 + 'a')");
        fold_same("x = p1 / 3 + 4");
        fold_same("foo() + 3 + 'a' + foo()");
        fold_same("x = 'a' + ('b' + p1 + p2)");
        fold_same("x = 1 + ('a' + p1)");
        fold_same("x = p1 + '' + p2");
        fold_same("x = 'a' + (1 + p1)");
        fold_same("x = (p2 + 'a') + (1 + p1)");
        fold_same("x = (p2 + 'a') + (1 + p1 + p2)");
        fold_same("x = (p2 + 'a') + (1 + (p1 + p2))");
    }

    #[test]
    fn test_fold_arithmetic() {
        fold("1n+ +1n", "1n + +1n");
        fold("1n- -1n", "1n - -1n");
        fold("a- -b", "a - -b");
    }

    #[test]
    fn test_fold_arithmetic_infinity() {
        fold("x=-Infinity-2", "x=-Infinity");
        fold("x=Infinity-2", "x=Infinity");
        fold("x=Infinity*5", "x=Infinity");
        fold("x = Infinity ** 2", "x = Infinity");
        fold("x = Infinity ** -2", "x = 0");

        fold("x = Infinity % Infinity", "x = NaN");
        fold("x = Infinity % 0", "x = NaN");
    }

    #[test]
    fn test_fold_add() {
        fold("x = 10 + 20", "x = 30");
        fold_same("x = y + 10 + 20");
        fold("x = 1 + null", "x = 1");
        fold("x = null + 1", "x = 1");
    }

    #[test]
    fn test_fold_sub() {
        fold("x = 10 - 20", "x = -10");
    }

    #[test]
    fn test_fold_multiply() {
        fold_same("x = 2.25 * 3");
        fold_same("z = x * y");
        fold_same("x = y * 5");
        // test("x = null * undefined", "x = NaN");
        // test("x = null * 1", "x = 0");
        // test("x = (null - 1) * 2", "x = -2");
        // test("x = (null + 1) * 2", "x = 2");
        // test("x = y + (z * 24 * 60 * 60 * 1000)", "x = y + z * 864E5");
        fold("x = y + (z & 24 & 60 & 60 & 1000)", "x = y + (z & 8)");
    }

    #[test]
    fn test_fold_division() {
        fold("x = Infinity / Infinity", "x = NaN");
        fold("x = Infinity / 0", "x = Infinity");
        fold("x = 1 / 0", "x = Infinity");
        fold("x = 0 / 0", "x = NaN");
        fold_same("x = 2 / 4");
        fold_same("x = y / 2 / 4");
    }

    #[test]
    fn test_fold_remainder() {
        fold_same("x = 3 % 2");
        fold_same("x = 3 % -2");
        fold_same("x = -1 % 3");
        fold("x = 1 % 0", "x = NaN");
        fold("x = 0 % 0", "x = NaN");
    }

    #[test]
    fn test_fold_exponential() {
        fold_same("x = 2 ** 3");
        fold_same("x = 2 ** -3");
        fold_same("x = 2 ** 55");
        fold_same("x = 3 ** -1");
        fold_same("x = (-1) ** 0.5");
        fold("x = (-0) ** 3", "x = -0");
        fold_same("x = null ** 0");
    }

    #[test]
    fn test_fold_shift_left() {
        fold("1 << 3", "8");
        fold("1.2345 << 0", "1");
        fold_same("1 << 24");
    }

    #[test]
    fn test_fold_shift_right() {
        fold("2147483647 >> -32.1", "2147483647");
    }

    #[test]
    fn test_fold_shift_right_zero_fill() {
        fold("10 >>> 1", "5");
        fold_same("-1 >>> 0");
    }

    #[test]
    fn test_fold_left() {
        fold_same("(+x - 1) + 2"); // not yet
        fold("(+x & 1) & 2", "+x & 0");
    }

    #[test]
    fn test_fold_array_length() {
        // Can fold
        fold("x = [].length", "x = 0");
        fold("x = [1,2,3].length", "x = 3");
        // test("x = [a,b].length", "x = 2");
        fold("x = 'abc'['length']", "x = 3");

        // Not handled yet
        fold("x = [,,1].length", "x = 3");

        // Cannot fold
        fold("x = [foo(), 0].length", "x = [foo(),0].length");
        fold_same("x = y.length");
    }

    #[test]
    fn test_fold_string_length() {
        // Can fold basic strings.
        fold("x = ''.length", "x = 0");
        fold("x = '123'.length", "x = 3");

        // Test Unicode escapes are accounted for.
        fold("x = '123\\u01dc'.length", "x = 4");
    }

    #[test]
    fn test_fold_instance_of() {
        // Non object types are never instances of anything.
        fold("64 instanceof Object", "!1");
        fold("64 instanceof Number", "!1");
        fold("'' instanceof Object", "!1");
        fold("'' instanceof String", "!1");
        fold("true instanceof Object", "!1");
        fold("true instanceof Boolean", "!1");
        fold("!0 instanceof Object", "!1");
        fold("!0 instanceof Boolean", "!1");
        fold("false instanceof Object", "!1");
        fold("null instanceof Object", "!1");
        fold("undefined instanceof Object", "!1");
        fold("NaN instanceof Object", "!1");
        fold("Infinity instanceof Object", "!1");

        // Array and object literals are known to be objects.
        fold("[] instanceof Object", "!0");
        fold("({}) instanceof Object", "!0");

        // These cases is foldable, but no handled currently.
        fold_same("new Foo() instanceof Object");
        // These would require type information to fold.
        fold_same("[] instanceof Foo");
        fold_same("({}) instanceof Foo");

        fold("(function() {}) instanceof Object", "!0");

        // An unknown value should never be folded.
        fold_same("x instanceof Foo");
        test_same("var x; foo(x instanceof Object)");
        fold_same("x instanceof Object");
        fold_same("0 instanceof Foo");
    }

    #[test]
    fn test_fold_instance_of_additional() {
        fold("(typeof {}) instanceof Object", "!1");
        fold("(+{}) instanceof Number", "!1");
    }

    #[test]
    fn test_fold_left_child_op() {
        fold("x & Infinity & 2", "x & 0");
        fold_same("x - Infinity - 2"); // FIXME: want "x-Infinity"
        fold_same("x - 1 + Infinity");
        fold_same("x - 2 + 1");
        fold_same("x - 2 + 3");
        fold_same("1 + x - 2 + 1");
        fold_same("1 + x - 2 + 3");
        fold_same("1 + x - 2 + 3 - 1");
        fold_same("f(x)-0");
        fold_same("x-0-0"); // FIXME: want x - 0
        fold_same("x+2-2+2");
        fold_same("x+2-2+2-2");
        fold_same("x-2+2");
        fold_same("x-2+2-2");
        fold_same("x-2+2-2+2");

        fold_same("1+x-0-na_n");
        fold_same("1+f(x)-0-na_n");
        fold_same("1+x-0+na_n");
        fold_same("1+f(x)-0+na_n");

        fold_same("1+x+na_n"); // unfoldable
        fold_same("x+2-2"); // unfoldable
        fold_same("x+2"); // nothing to do
        fold_same("x-2"); // nothing to do
    }

    #[test]
    fn test_associative_fold_constants_with_variables() {
        // mul and add should not fold
        fold_same("alert(x * 12 * 20)");
        fold_same("alert(12 * x * 20)");
        fold_same("alert(x + 12 + 20)");
        fold_same("alert(12 + x + 20)");
        fold("alert(x & 12 & 20)", "alert(x & 4)");
        fold("alert(12 & x & 20)", "alert(x & 4)");
    }

    #[test]
    fn test_to_number() {
        fold("x = +''", "x = 0");
        fold("x = +'+Infinity'", "x = Infinity");
        fold("x = +'-Infinity'", "x = -Infinity");

        for op in ["", "+", "-"] {
            for s in ["inf", "infinity", "INFINITY", "InFiNiTy"] {
                fold(&format!("x = +'{op}{s}'"), "x = NaN");
            }
        }
    }

    #[test]
    fn test_number_constructor() {
        fold("Number(undefined)", "NaN");
        fold("Number(void 0)", "NaN");
        fold("Number(null)", "0");
        fold("Number(true)", "1");
        fold("Number(false)", "0");
        fold("Number('a')", "NaN");
        fold("Number('1')", "1");
        test_same("var Number; NOOP(Number(1))");
    }

    #[test]
    fn test_fold_typeof_addition_string() {
        fold_same("typeof foo");
        fold_same("typeof foo + '123'");
        fold("typeof foo + ''", "typeof foo");
        fold_same("typeof foo - ''");
    }

    #[test]
    fn test_fold_same_typeof() {
        fold("typeof foo === typeof bar", "typeof foo == typeof bar");
        fold("typeof foo !== typeof bar", "typeof foo != typeof bar");
        fold("typeof foo.bar === typeof foo.bar", "typeof foo.bar == typeof foo.bar");
        fold("typeof foo.bar !== typeof foo.bar", "typeof foo.bar != typeof foo.bar");
    }

    #[test]
    fn test_fold_invalid_typeof_comparison() {
        fold("typeof foo == 123", "!1");
        fold("typeof foo == '123'", "!1");
        fold("typeof foo === null", "!1");
        fold("typeof foo === undefined", "!1");
        fold("typeof foo !== 123", "!0");
        fold("typeof foo !== '123'", "!0");
        fold("typeof foo != null", "!0");
        fold("typeof foo != undefined", "!0");
        fold("typeof foo === 'string'", "typeof foo == 'string'");
        fold("typeof foo === 'number'", "typeof foo == 'number'");
    }

    #[test]
    fn test_issue_8782() {
        fold("+(void unknown())", "+void unknown()");
    }

    // TODO: All big ints are rare and difficult to handle.
    mod bigint {
        use super::{
            fold, fold_same, MAX_SAFE_FLOAT, MAX_SAFE_INT, NEG_MAX_SAFE_FLOAT, NEG_MAX_SAFE_INT,
        };

        #[test]
        fn test_fold_bitwise_op_with_big_int() {
            fold("x = 1n & 1n", "x = 1n");
            fold("x = 1n & 2n", "x = 0n");
            fold("x = 3n & 1n", "x = 1n");
            fold("x = 3n & 3n", "x = 3n");

            fold("x = 1n | 1n", "x = 1n");
            fold("x = 1n | 2n", "x = 3n");
            fold("x = 1n | 3n", "x = 3n");
            fold("x = 3n | 1n", "x = 3n");
            fold("x = 3n | 3n", "x = 3n");
            fold("x = 1n | 4n", "x = 5n");

            fold("x = 1n ^ 1n", "x = 0n");
            fold("x = 1n ^ 2n", "x = 3n");
            fold("x = 3n ^ 1n", "x = 2n");
            fold("x = 3n ^ 3n", "x = 0n");

            // test("x = -1n & 0n", "x = 0n");
            // test("x = 0n & -1n", "x = 0n");
            fold("x = 1n & 4n", "x = 0n");
            fold("x = 2n & 3n", "x = 2n");

            fold("x = 1n & 3000000000n", "x = 0n");
            fold("x = 3000000000n & 1n", "x = 0n");

            // bitwise OR does not affect the sign of a bigint
            fold("x = 1n | 3000000001n", "x = 3000000001n");
            fold("x = 4294967295n | 0n", "x = 4294967295n");

            fold("x = y & 1n & 1n", "x = y & 1n");
            fold("x = y & 1n & 2n", "x = y & 0n");
            fold("x = y & 3n & 1n", "x = y & 1n");
            fold("x = 3n & y & 1n", "x = y & 1n");
            fold("x = y & 3n & 3n", "x = y & 3n");
            fold("x = 3n & y & 3n", "x = y & 3n");

            fold("x = y | 1n | 1n", "x = y | 1n");
            fold("x = y | 1n | 2n", "x = y | 3n");
            fold("x = y | 3n | 1n", "x = y | 3n");
            fold("x = 3n | y | 1n", "x = y | 3n");
            fold("x = y | 3n | 3n", "x = y | 3n");
            fold("x = 3n | y | 3n", "x = y | 3n");

            fold("x = y ^ 1n ^ 1n", "x = y ^ 0n");
            fold("x = y ^ 1n ^ 2n", "x = y ^ 3n");
            fold("x = y ^ 3n ^ 1n", "x = y ^ 2n");
            fold("x = 3n ^ y ^ 1n", "x = y ^ 2n");
            fold("x = y ^ 3n ^ 3n", "x = y ^ 0n");
            fold("x = 3n ^ y ^ 3n", "x = y ^ 0n");

            // TypeError: Cannot mix BigInt and other types
            fold_same("1n & 1");
            fold_same("1n | 1");
            fold_same("1n ^ 1");
        }

        #[test]
        #[ignore]
        fn test_bigint_number_comparison() {
            fold("1n < 2", "!0");
            fold("1n > 2", "!1");
            fold("1n == 1", "!0");
            fold("1n == 2", "!1");

            // comparing with decimals is allowed
            fold("1n < 1.1", "!0");
            fold("1n < 1.9", "!0");
            fold("1n < 0.9", "!1");
            fold("-1n < -1.1", "!1");
            fold("-1n < -1.9", "!1");
            fold("-1n < -0.9", "!0");
            fold("1n > 1.1", "!1");
            fold("1n > 0.9", "!0");
            fold("-1n > -1.1", "!0");
            fold("-1n > -0.9", "!1");

            // Don't fold unsafely large numbers because there might be floating-point error
            fold(&format!("0n > {MAX_SAFE_INT}"), "!1");
            fold(&format!("0n < {MAX_SAFE_INT}"), "!0");
            fold(&format!("0n > {NEG_MAX_SAFE_INT}"), "!0");
            fold(&format!("0n < {NEG_MAX_SAFE_INT}"), "!1");
            fold(&format!("0n > {MAX_SAFE_FLOAT}"), "!1");
            fold(&format!("0n < {MAX_SAFE_FLOAT}"), "!0");
            fold(&format!("0n > {NEG_MAX_SAFE_FLOAT}"), "!0");
            fold(&format!("0n < {NEG_MAX_SAFE_FLOAT}"), "!1");

            // comparing with Infinity is allowed
            fold("1n < Infinity", "!0");
            fold("1n > Infinity", "!1");
            fold("1n < -Infinity", "!1");
            fold("1n > -Infinity", "!0");

            // null is interpreted as 0 when comparing with bigint
            fold("1n < null", "!1");
            fold("1n > null", "!0");
        }

        #[test]
        #[ignore]
        fn test_bigint_string_comparison() {
            fold("1n < '2'", "!0");
            fold("2n > '1'", "!0");
            fold("123n > '34'", "!0");
            fold("1n == '1'", "!0");
            fold("1n == '2'", "!1");
            fold("1n != '1'", "!1");
            fold("1n === '1'", "!1");
            fold("1n !== '1'", "!0");
        }

        #[test]
        #[ignore]
        fn test_string_bigint_comparison() {
            fold("'1' < 2n", "!0");
            fold("'2' > 1n", "!0");
            fold("'123' > 34n", "!0");
            fold("'1' == 1n", "!0");
            fold("'1' == 2n", "!1");
            fold("'1' != 1n", "!1");
            fold("'1' === 1n", "!1");
            fold("'1' !== 1n", "!0");
        }

        #[test]
        fn test_object_bigint_comparison() {
            fold_same("{ valueOf: function() { return 0n; } } != 0n");
            fold_same("{ toString: function() { return '0'; } } != 0n");
        }

        #[test]
        fn test_fold_object_spread() {
            fold_same("({ z, ...a })");
            let result = "({ z })";
            fold("({ z, ...[] })", result);
            fold("({ z, ...{} })", result);
            fold("({ z, ...undefined })", result);
            fold("({ z, ...void 0 })", result);
            fold("({ z, ...null })", result);
            fold("({ z, ...true })", result);
            fold("({ z, ...1 })", result);
            fold("({ z, ...1n })", result);
            fold("({ z, .../asdf/ })", result);
        }
    }
}
