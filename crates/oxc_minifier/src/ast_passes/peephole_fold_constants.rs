use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, LogicalOperator},
};
use oxc_traverse::{traverse_mut_with_ctx, Ancestor, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{node_util::Ctx, CompressorPass};

/// Constant Folding
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>
pub struct PeepholeFoldConstants {
    pub(crate) changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeFoldConstants {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = false;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeFoldConstants {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx(ctx);
        if let Some(folded_expr) = match expr {
            Expression::BinaryExpression(e) => Self::try_fold_binary_expr(e, ctx)
                .or_else(|| Self::try_fold_binary_typeof_comparison(e, ctx)),
            Expression::UnaryExpression(e) => Self::try_fold_unary_expr(e, ctx),
            Expression::StaticMemberExpression(e) => Self::try_fold_static_member_expr(e, ctx),
            Expression::LogicalExpression(e) => Self::try_fold_logical_expr(e, ctx),
            Expression::ChainExpression(e) => Self::try_fold_optional_chain(e, ctx),
            Expression::CallExpression(e) => Self::try_fold_number_constructor(e, ctx),
            Expression::ObjectExpression(e) => self.fold_object_spread(e, ctx),
            _ => None,
        } {
            *expr = folded_expr;
            self.changed = true;
        };
    }
}

impl<'a, 'b> PeepholeFoldConstants {
    pub fn new() -> Self {
        Self { changed: false }
    }

    #[expect(clippy::float_cmp)]
    fn try_fold_unary_expr(e: &UnaryExpression<'a>, ctx: Ctx<'a, 'b>) -> Option<Expression<'a>> {
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
        static_member_expr: &mut StaticMemberExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        // TODO: tryFoldObjectPropAccess(n, left, name)
        ctx.eval_static_member_expression(static_member_expr)
            .map(|value| ctx.value_to_expr(static_member_expr.span, value))
    }

    fn try_fold_logical_expr(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        match logical_expr.operator {
            LogicalOperator::And | LogicalOperator::Or => Self::try_fold_and_or(logical_expr, ctx),
            LogicalOperator::Coalesce => Self::try_fold_coalesce(logical_expr, ctx),
        }
    }

    fn try_fold_optional_chain(
        chain_expr: &mut ChainExpression<'a>,
        ctx: Ctx<'a, 'b>,
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
        ctx: Ctx<'a, 'b>,
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
            } else if !left.may_have_side_effects() {
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
                    if !left_child.right.may_have_side_effects() {
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
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        debug_assert_eq!(logical_expr.operator, LogicalOperator::Coalesce);
        let left = &logical_expr.left;
        let left_val = ValueType::from(left);
        match left_val {
            ValueType::Null | ValueType::Undefined => {
                Some(if left.may_have_side_effects() {
                    // e.g. `(a(), null) ?? 1` => `(a(), null, 1)`
                    let expressions = ctx.ast.vec_from_array([
                        ctx.ast.move_expression(&mut logical_expr.left),
                        ctx.ast.move_expression(&mut logical_expr.right),
                    ]);
                    ctx.ast.expression_sequence(SPAN, expressions)
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
        ctx: Ctx<'a, 'b>,
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
    fn try_fold_add(e: &mut BinaryExpression<'a>, ctx: Ctx<'a, 'b>) -> Option<Expression<'a>> {
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

    fn try_fold_comparison(e: &BinaryExpression<'a>, ctx: Ctx<'a, 'b>) -> Option<Expression<'a>> {
        let left = &e.left;
        let right = &e.right;
        let op = e.operator;
        if left.may_have_side_effects() || right.may_have_side_effects() {
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
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        ctx: Ctx<'a, 'b>,
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
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        ctx: Ctx<'a, 'b>,
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
        ctx: Ctx<'a, 'b>,
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
        Some(ctx.value_to_expr(
            e.span,
            ConstantValue::Number(match &e.arguments[0] {
                // `Number(undefined)` -> `NaN`
                Argument::Identifier(ident) if ctx.is_identifier_undefined(ident) => f64::NAN,
                // `Number(null)` -> `0`
                Argument::NullLiteral(_) => 0.0,
                // `Number(true)` -> `1` `Number(false)` -> `0`
                Argument::BooleanLiteral(b) => f64::from(b.value),
                // `Number(100)` -> `100`
                Argument::NumericLiteral(n) => n.value,
                // `Number("a")` -> `+"a"` -> `NaN`
                // `Number("1")` -> `+"1"` -> `1`
                Argument::StringLiteral(n) => {
                    let argument =
                        ctx.ast.expression_string_literal(n.span, n.value.clone(), n.raw.clone());
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
                _ => return None,
            }),
        ))
    }

    // `typeof a === typeof b` -> `typeof a == typeof b`, `typeof a != typeof b` -> `typeof a != typeof b`,
    // `typeof a == typeof a` -> `true`, `typeof a != typeof a` -> `false`
    fn try_fold_binary_typeof_comparison(
        bin_expr: &mut BinaryExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
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

        None
    }

    fn fold_object_spread(
        &mut self,
        e: &mut ObjectExpression<'a>,
        ctx: Ctx<'a, 'b>,
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
            self.changed = true;
        }
        None
    }
}

/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    static MAX_SAFE_FLOAT: f64 = 9_007_199_254_740_991_f64;
    static NEG_MAX_SAFE_FLOAT: f64 = -9_007_199_254_740_991_f64;

    static MAX_SAFE_INT: i64 = 9_007_199_254_740_991_i64;
    static NEG_MAX_SAFE_INT: i64 = -9_007_199_254_740_991_i64;

    use crate::tester;

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeFoldConstants::new();
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    #[test]
    fn test_comparison() {
        test("(1, 2) !== 2", "false");
    }

    #[test]
    fn undefined_comparison1() {
        test("undefined == undefined", "true");
        test("undefined == null", "true");
        test("undefined == void 0", "true");

        test("undefined == 0", "false");
        test("undefined == 1", "false");
        test("undefined == 'hi'", "false");
        test("undefined == true", "false");
        test("undefined == false", "false");

        test("undefined === undefined", "true");
        test("undefined === null", "false");
        test("undefined === void 0", "true");

        test_same("undefined == this");
        test_same("undefined == x");

        test("undefined != undefined", "false");
        test("undefined != null", "false");
        test("undefined != void 0", "false");

        test("undefined != 0", "true");
        test("undefined != 1", "true");
        test("undefined != 'hi'", "true");
        test("undefined != true", "true");
        test("undefined != false", "true");

        test("undefined !== undefined", "false");
        test("undefined !== void 0", "false");
        test("undefined !== null", "true");

        test_same("undefined != this");
        test_same("undefined != x");

        test("undefined < undefined", "false");
        test("undefined > undefined", "false");
        test("undefined >= undefined", "false");
        test("undefined <= undefined", "false");

        test("0 < undefined", "false");
        test("true > undefined", "false");
        test("'hi' >= undefined", "false");
        test("null <= undefined", "false");

        test("undefined < 0", "false");
        test("undefined > true", "false");
        test("undefined >= 'hi'", "false");
        test("undefined <= null", "false");

        test("null == undefined", "true");
        test("0 == undefined", "false");
        test("1 == undefined", "false");
        test("'hi' == undefined", "false");
        test("true == undefined", "false");
        test("false == undefined", "false");
        test("null === undefined", "false");
        test("void 0 === undefined", "true");

        test("undefined == NaN", "false");
        test("NaN == undefined", "false");
        test("undefined == Infinity", "false");
        test("Infinity == undefined", "false");
        test("undefined == -Infinity", "false");
        test("-Infinity == undefined", "false");
        test("({}) == undefined", "false");
        test("undefined == ({})", "false");
        test("([]) == undefined", "false");
        test("undefined == ([])", "false");
        test("(/a/g) == undefined", "false");
        test("undefined == (/a/g)", "false");
        test("(function(){}) == undefined", "false");
        test("undefined == (function(){})", "false");

        test("undefined != NaN", "true");
        test("NaN != undefined", "true");
        test("undefined != Infinity", "true");
        test("Infinity != undefined", "true");
        test("undefined != -Infinity", "true");
        test("-Infinity != undefined", "true");
        test("({}) != undefined", "true");
        test("undefined != ({})", "true");
        test("([]) != undefined", "true");
        test("undefined != ([])", "true");
        test("(/a/g) != undefined", "true");
        test("undefined != (/a/g)", "true");
        test("(function(){}) != undefined", "true");
        test("undefined != (function(){})", "true");

        test_same("this == undefined");
        test_same("x == undefined");
    }

    #[test]
    fn test_undefined_comparison2() {
        test("\"123\" !== void 0", "true");
        test("\"123\" === void 0", "false");

        test("void 0 !== \"123\"", "true");
        test("void 0 === \"123\"", "false");
    }

    #[test]
    fn test_undefined_comparison3() {
        test("\"123\" !== undefined", "true");
        test("\"123\" === undefined", "false");

        test("undefined !== \"123\"", "true");
        test("undefined === \"123\"", "false");
    }

    #[test]
    fn test_null_comparison1() {
        test("null == undefined", "true");
        test("null == null", "true");
        test("null == void 0", "true");

        test("null == 0", "false");
        test("null == 1", "false");
        // test("null == 0n", "false");
        // test("null == 1n", "false");
        test("null == 'hi'", "false");
        test("null == true", "false");
        test("null == false", "false");

        test("null === undefined", "false");
        test("null === null", "true");
        test("null === void 0", "false");
        test_same("x===null");

        test_same("this==null");
        test_same("x==null");

        test("null != undefined", "false");
        test("null != null", "false");
        test("null != void 0", "false");

        test("null != 0", "true");
        test("null != 1", "true");
        // test("null != 0n", "true");
        // test("null != 1n", "true");
        test("null != 'hi'", "true");
        test("null != true", "true");
        test("null != false", "true");

        test("null !== undefined", "true");
        test("null !== void 0", "true");
        test("null !== null", "false");

        test_same("this!=null");
        test_same("x!=null");

        test("null < null", "false");
        test("null > null", "false");
        test("null >= null", "true");
        test("null <= null", "true");

        test("0 < null", "false");
        test("0 > null", "false");
        test("0 >= null", "true");
        // test("0n < null", "false");
        // test("0n > null", "false");
        // test("0n >= null", "true");
        test("true > null", "true");
        test("'hi' < null", "false");
        test("'hi' >= null", "false");
        test("null <= null", "true");

        test("null < 0", "false");
        // test("null < 0n", "false");
        test("null > true", "false");
        test("null < 'hi'", "false");
        test("null >= 'hi'", "false");
        test("null <= null", "true");

        test("null == null", "true");
        test("0 == null", "false");
        test("1 == null", "false");
        test("'hi' == null", "false");
        test("true == null", "false");
        test("false == null", "false");
        test("null === null", "true");
        test("void 0 === null", "false");

        test("null == NaN", "false");
        test("NaN == null", "false");
        test("null == Infinity", "false");
        test("Infinity == null", "false");
        test("null == -Infinity", "false");
        test("-Infinity == null", "false");
        test("({}) == null", "false");
        test("null == ({})", "false");
        test("([]) == null", "false");
        test("null == ([])", "false");
        test("(/a/g) == null", "false");
        test("null == (/a/g)", "false");
        test("(function(){}) == null", "false");
        test("null == (function(){})", "false");

        test("null != NaN", "true");
        test("NaN != null", "true");
        test("null != Infinity", "true");
        test("Infinity != null", "true");
        test("null != -Infinity", "true");
        test("-Infinity != null", "true");
        test("({}) != null", "true");
        test("null != ({})", "true");
        test("([]) != null", "true");
        test("null != ([])", "true");
        test("(/a/g) != null", "true");
        test("null != (/a/g)", "true");
        test("(function(){}) != null", "true");
        test("null != (function(){})", "true");

        test_same("({a:f()})==null");
        test_same("[f()]==null");

        test_same("this==null");
        test_same("x==null");
    }

    #[test]
    fn test_boolean_boolean_comparison() {
        test_same("!x==!y");
        test_same("!x<!y");
        test_same("!x!==!y");

        test_same("!x==!x"); // foldable
        test_same("!x<!x"); // foldable
        test_same("!x!==!x"); // foldable
    }

    #[test]
    fn test_boolean_number_comparison() {
        test_same("!x==+y");
        test_same("!x<=+y");
        test_same("!x !== +y");
    }

    #[test]
    fn test_number_boolean_comparison() {
        test_same("+x==!y");
        test_same("+x<=!y");
        test_same("+x === !y");
    }

    #[test]
    fn test_boolean_string_comparison() {
        test_same("!x==''+y");
        test_same("!x<=''+y");
        test_same("!x !== '' + y");
    }

    #[test]
    fn test_string_boolean_comparison() {
        test_same("''+x==!y");
        test_same("''+x<=!y");
        test_same("'' + x === !y");
    }

    #[test]
    fn test_string_string_comparison() {
        test("'a' < 'b'", "true");
        test("'a' <= 'b'", "true");
        test("'a' > 'b'", "false");
        test("'a' >= 'b'", "false");
        test("+'a' < +'b'", "false");
        test_same("typeof a < 'a'");
        test_same("'a' >= typeof a");
        test_same("typeof a < typeof a");
        test_same("typeof a >= typeof a");
        test("typeof 3 > typeof 4", "false");
        test("typeof function() {} < typeof function() {}", "false");
        test("'a' == 'a'", "true");
        test("'b' != 'a'", "true");
        test_same("typeof a != 'number'");
        test("'a' === 'a'", "true");
        test("'b' !== 'a'", "true");
        test_same("'' + x <= '' + y");
        test_same("'' + x != '' + y");
        test_same("'' + x === '' + y");

        test_same("'' + x <= '' + x"); // potentially foldable
        test_same("'' + x != '' + x"); // potentially foldable
        test_same("'' + x === '' + x"); // potentially foldable

        test(r#"if ("string" !== "\u000Bstr\u000Bing\u000B") {}"#, "if (false) {}\n");
    }

    #[test]
    fn test_number_string_comparison() {
        test("1 < '2'", "true");
        test("2 > '1'", "true");
        test("123 > '34'", "true");
        test("NaN >= 'NaN'", "false");
        test("1 == '2'", "false");
        test("1 != '1'", "false");
        test("NaN == 'NaN'", "false");
        test("1 === '1'", "false");
        test("1 !== '1'", "true");
        test_same("+x>''+y");
        test_same("+x==''+y");
        test_same("+x !== '' + y");
    }

    #[test]
    fn test_string_number_comparison() {
        test("'1' < 2", "true");
        test("'2' > 1", "true");
        test("'123' > 34", "true");
        test("'NaN' < NaN", "false");
        test("'1' == 2", "false");
        test("'1' != 1", "false");
        test("'NaN' == NaN", "false");
        test("'1' === 1", "false");
        test("'1' !== 1", "true");
        test_same("''+x<+y");
        test_same("''+x==+y");
        test_same("'' + x === +y");
    }

    #[test]
    fn test_nan_comparison() {
        test("NaN < 1", "false");
        test("NaN <= 1", "false");
        test("NaN > 1", "false");
        test("NaN >= 1", "false");
        // test("NaN < 1n", "false");
        // test("NaN <= 1n", "false");
        // test("NaN > 1n", "false");
        // test("NaN >= 1n", "false");

        test("NaN < NaN", "false");
        test("NaN >= NaN", "false");
        test("NaN == NaN", "false");
        test("NaN === NaN", "false");

        test("NaN < null", "false");
        test("null >= NaN", "false");
        test("NaN == null", "false");
        test("null != NaN", "true");
        test("null === NaN", "false");

        test("NaN < undefined", "false");
        test("undefined >= NaN", "false");
        test("NaN == undefined", "false");
        test("undefined != NaN", "true");
        test("undefined === NaN", "false");

        test_same("NaN<x");
        test_same("x>=NaN");
        test_same("NaN==x");
        test_same("x!=NaN");
        test_same("NaN === x");
        test_same("x !== NaN");
        test_same("NaN==foo()");
    }

    #[test]
    fn js_typeof() {
        test("x = typeof 1", "x = \"number\"");
        test("x = typeof 'foo'", "x = \"string\"");
        test("x = typeof true", "x = \"boolean\"");
        test("x = typeof false", "x = \"boolean\"");
        test("x = typeof null", "x = \"object\"");
        test("x = typeof undefined", "x = \"undefined\"");
        test("x = typeof void 0", "x = \"undefined\"");
        test("x = typeof []", "x = \"object\"");
        test("x = typeof [1]", "x = \"object\"");
        test("x = typeof [1,[]]", "x = \"object\"");
        test("x = typeof {}", "x = \"object\"");
        test("x = typeof function() {}", "x = 'function'");

        test_same("x = typeof[1,[foo()]]");
        test_same("x = typeof{bathwater:baby()}");
    }

    #[test]
    fn test_fold_unary() {
        test_same("!foo()");
        test_same("~foo()");
        test_same("-foo()");

        test("a=!true", "a=false");
        test("a=!10", "a=false");
        test("a=!false", "a=true");
        test_same("a=!foo()");

        test("a=-0", "a=-0");
        test("a=-(0)", "a=-0");
        test_same("a=-Infinity");
        test("a=-NaN", "a=NaN");
        test_same("a=-foo()");
        test("-undefined", "NaN");
        test("-null", "-0");
        test("-NaN", "NaN");

        test("a=+true", "a=1");
        test("a=+10", "a=10");
        test("a=+false", "a=0");
        test_same("a=+foo()");
        test_same("a=+f");
        // test("a=+(f?true:false)", "a=+(f?1:0)");
        test("a=+0", "a=0");
        test("a=+Infinity", "a=Infinity");
        test("a=+NaN", "a=NaN");
        test("a=+-7", "a=-7");
        test("a=+.5", "a=.5");

        test("a=~~0", "a=0");
        test("a=~~10", "a=10");
        test("a=~-7", "a=6");
        test_same("a=~~foo()");
        test("a=~0xffffffff", "a=0");
        test("a=~~0xffffffff", "a=-1");
        // test_same("a=~.5");
    }

    #[test]
    fn test_fold_unary_big_int() {
        test("-(1n)", "-1n");
        test("- -1n", "1n");
        test("!1n", "false");
        test("~0n", "-1n");

        test("~-1n", "0n");
        test("~~1n", "1n");

        test("~0x3n", "-4n");
        test("~0b11n", "-4n");
    }

    #[test]
    fn test_unary_ops_string_compare() {
        test_same("a = -1");
        test("a = ~0", "a = -1");
        test("a = ~1", "a = -2");
        test("a = ~101", "a = -102");

        test("a = ~1.1", "a = -2");
        test("a = ~0x3", "a = -4"); // Hexadecimal number
        test("a = ~9", "a = -10"); // Despite `-10` is longer than `~9`, the compiler still folds it.
        test_same("a = ~b");
        test("a = ~NaN", "a = -1");
        test("a = ~-Infinity", "a = -1");
        test("x = ~2147483658.0", "x = 2147483637");
        test("x = ~-2147483658", "x = -2147483639");
    }

    #[test]
    fn test_fold_logical_op() {
        test("x = true && x", "x = x");
        test("x = [foo()] && x", "x = ([foo()],x)");

        test("x = false && x", "x = false");
        test("x = true || x", "x = true");
        test("x = false || x", "x = x");
        test("x = 0 && x", "x = 0");
        test("x = 3 || x", "x = 3");
        test("x = 0n && x", "x = 0n");
        test("x = 3n || x", "x = 3n");
        test("x = false || 0", "x = 0");

        // unfoldable, because the right-side may be the result
        test("a = x && true", "a=x && true");
        test("a = x && false", "a=x && false");
        test("a = x || 3", "a=x || 3");
        test("a = x || false", "a=x || false");
        test("a = b ? c : x || false", "a=b ? c:x || false");
        test("a = b ? x || false : c", "a=b ? x || false:c");
        test("a = b ? c : x && true", "a=b ? c:x && true");
        test("a = b ? x && true : c", "a=b ? x && true:c");

        // folded, but not here.
        test_same("a = x || false ? b : c");
        test_same("a = x && true ? b : c");

        test("x = foo() || true || bar()", "x = foo() || true");
        test("x = foo() || true && bar()", "x = foo() || bar()");
        test("x = foo() || false && bar()", "x = foo() || false");
        test("x = foo() && false && bar()", "x = foo() && false");
        test("x = foo() && false || bar()", "x = (foo() && false,bar())");
        test("x = foo() || false || bar()", "x = foo() || bar()");
        test("x = foo() && true && bar()", "x = foo() && bar()");
        test("x = foo() || true || bar()", "x = foo() || true");
        test("x = foo() && false && bar()", "x = foo() && false");
        test("x = foo() && 0 && bar()", "x = foo() && 0");
        test("x = foo() && 1 && bar()", "x = foo() && bar()");
        test("x = foo() || 0 || bar()", "x = foo() || bar()");
        test("x = foo() || 1 || bar()", "x = foo() || 1");
        test("x = foo() && 0n && bar()", "x = foo() && 0n");
        test("x = foo() && 1n && bar()", "x = foo() && bar()");
        test("x = foo() || 0n || bar()", "x = foo() || bar()");
        test("x = foo() || 1n || bar()", "x = foo() || 1n");
        test_same("x = foo() || bar() || baz()");
        test_same("x = foo() && bar() && baz()");

        test("0 || b()", "b()");
        test("1 && b()", "b()");
        test("a() && (1 && b())", "a() && b()");
        test("(a() && 1) && b()", "a() && b()");

        test("(x || '') || y;", "x || y");
        test("false || (x || '');", "x || ''");
        test("(x && 1) && y;", "x && y");
        test("true && (x && 1);", "x && 1");

        // Really not foldable, because it would change the type of the
        // expression if foo() returns something truthy but not true.
        // Cf. FoldConstants.tryFoldAndOr().
        // An example would be if foo() is 1 (truthy) and bar() is 0 (falsey):
        // (1 && true) || 0 == true
        // 1 || 0 == 1, but true =/= 1
        test_same("x = foo() && true || bar()");
        test_same("foo() && true || bar()");
    }

    #[test]
    fn test_fold_logical_op2() {
        test("x = function(){} && x", "x=x");
        test("x = true && function(){}", "x=function(){}");
        test("x = [(function(){alert(x)})()] && x", "x=([function(){alert(x)}()],x)");
    }

    #[test]
    fn test_fold_nullish_coalesce() {
        // fold if left is null/undefined
        test("null ?? 1", "1");
        test("undefined ?? false", "false");
        test("(a(), null) ?? 1", "(a(), null, 1)");

        test("x = [foo()] ?? x", "x = [foo()]");

        // short circuit on all non nullish LHS
        test("x = false ?? x", "x = false");
        test("x = true ?? x", "x = true");
        test("x = 0 ?? x", "x = 0");
        test("x = 3 ?? x", "x = 3");

        // unfoldable, because the right-side may be the result
        test_same("a = x ?? true");
        test_same("a = x ?? false");
        test_same("a = x ?? 3");
        test_same("a = b ? c : x ?? false");
        test_same("a = b ? x ?? false : c");

        // folded, but not here.
        test_same("a = x ?? false ? b : c");
        test_same("a = x ?? true ? b : c");

        test_same("x = foo() ?? true ?? bar()");
        test("x = foo() ?? (true && bar())", "x = foo() ?? bar()");
        test_same("x = (foo() || false) ?? bar()");

        test("a() ?? (1 ?? b())", "a() ?? 1");
        test("(a() ?? 1) ?? b()", "a() ?? 1 ?? b()");
    }

    #[test]
    fn test_fold_void() {
        test_same("void 0");
        test("void 1", "void 0");
        test_same("void x");
        test_same("void x()");
    }

    #[test]
    fn test_fold_opt_chain() {
        // can't fold when optional part may execute
        test_same("a = x?.y");
        test_same("a = x?.()");

        // fold args of optional call
        test("x = foo() ?. (true && bar())", "x = foo() ?.(bar())");
        test("a() ?. (1 ?? b())", "a() ?. (1)");

        // test("({a})?.a.b.c.d()?.x.y.z", "a.b.c.d()?.x.y.z");

        test("x = undefined?.y", "x = void 0");
        test("x = null?.y", "x = void 0");
        test("x = undefined?.[foo]", "x = void 0");
        test("x = null?.[foo]", "x = void 0");
    }

    #[test]
    fn test_fold_bitwise_op() {
        test("x = 1 & 1", "x = 1");
        test("x = 1 & 2", "x = 0");
        test("x = 3 & 1", "x = 1");
        test("x = 3 & 3", "x = 3");

        test("x = 1 | 1", "x = 1");
        test("x = 1 | 2", "x = 3");
        test("x = 3 | 1", "x = 3");
        test("x = 3 | 3", "x = 3");

        test("x = 1 ^ 1", "x = 0");
        test("x = 1 ^ 2", "x = 3");
        test("x = 3 ^ 1", "x = 2");
        test("x = 3 ^ 3", "x = 0");

        test("x = -1 & 0", "x = 0");
        test("x = 0 & -1", "x = 0");
        test("x = 1 & 4", "x = 0");
        test("x = 2 & 3", "x = 2");

        // make sure we fold only when we are supposed to -- not when doing so would
        // lose information or when it is performed on nonsensical arguments.
        test("x = 1 & 1.1", "x = 1");
        test("x = 1.1 & 1", "x = 1");
        test("x = 1 & 3000000000", "x = 0");
        test("x = 3000000000 & 1", "x = 0");

        // Try some cases with | as well
        test("x = 1 | 4", "x = 5");
        test("x = 1 | 3", "x = 3");
        test("x = 1 | 1.1", "x = 1");
        // test_same("x = 1 | 3e9");

        // these cases look strange because bitwise OR converts unsigned numbers to be signed
        test("x = 1 | 3000000001", "x = -1294967295");
        test("x = 4294967295 | 0", "x = -1");
    }

    #[test]
    fn test_fold_bitwise_op2() {
        test("x = y & 1 & 1", "x = y & 1");
        test("x = y & 1 & 2", "x = y & 0");
        test("x = y & 3 & 1", "x = y & 1");
        test("x = 3 & y & 1", "x = y & 1");
        test("x = y & 3 & 3", "x = y & 3");
        test("x = 3 & y & 3", "x = y & 3");

        test("x = y | 1 | 1", "x = y | 1");
        test("x = y | 1 | 2", "x = y | 3");
        test("x = y | 3 | 1", "x = y | 3");
        test("x = 3 | y | 1", "x = y | 3");
        test("x = y | 3 | 3", "x = y | 3");
        test("x = 3 | y | 3", "x = y | 3");

        test("x = y ^ 1 ^ 1", "x = y ^ 0");
        test("x = y ^ 1 ^ 2", "x = y ^ 3");
        test("x = y ^ 3 ^ 1", "x = y ^ 2");
        test("x = 3 ^ y ^ 1", "x = y ^ 2");
        test("x = y ^ 3 ^ 3", "x = y ^ 0");
        test("x = 3 ^ y ^ 3", "x = y ^ 0");

        test("x = Infinity | NaN", "x=0");
        test("x = 12 | NaN", "x=12");
    }

    #[test]
    fn test_fold_bitwise_op_additional() {
        test("x = null & 1", "x = 0");
        test_same("x = (2 ** 31 - 1) | 1");
        test_same("x = (2 ** 31) | 1");

        // https://github.com/oxc-project/oxc/issues/7944
        test_same("(x - 1) & 1");
        test_same("(y >> 3) & 7");
        test("(y & 3) & 7", "y & 3");
        test_same("(y | 3) & 7");
        test("y | 3 & 7", "y | 3");
    }

    #[test]
    fn test_fold_bitwise_not() {
        test("~undefined", "-1");
        test("~null", "-1");
        test("~false", "-1");
        test("~true", "-2");
        test("~'1'", "-2");
        test("~'-1'", "0");
        test("~{}", "-1");
    }

    #[test]
    fn test_fold_bit_shifts() {
        test("x = 1 << 0", "x=1");
        test("x = -1 << 0", "x=-1");
        test("x = 1 << 1", "x=2");
        test("x = 3 << 1", "x=6");
        test("x = 1 << 8", "x=256");

        test("x = 1 >> 0", "x=1");
        test("x = -1 >> 0", "x=-1");
        test("x = 1 >> 1", "x=0");
        test("x = 2 >> 1", "x=1");
        test("x = 5 >> 1", "x=2");
        test("x = 127 >> 3", "x=15");
        test("x = 3 >> 1", "x=1");
        test("x = 3 >> 2", "x=0");
        test("x = 10 >> 1", "x=5");
        test("x = 10 >> 2", "x=2");
        test("x = 10 >> 5", "x=0");

        test("x = 10 >>> 1", "x=5");
        test("x = 10 >>> 2", "x=2");
        test("x = 10 >>> 5", "x=0");
        test_same("x = -1 >>> 1");
        test_same("x = -1 >>> 0");
        test_same("x = -2 >>> 0");
        test("x = 0x90000000 >>> 28", "x=9");

        test("x = 0xffffffff << 0", "x=-1");
        test("x = 0xffffffff << 4", "x=-16");
        test("1 << 32", "1");
        test("1 << -1", "1<<-1");
        test("1 >> 32", "1");

        // Regression on #6161, ported from <https://github.com/tc39/test262/blob/05c45a4c430ab6fee3e0c7f0d47d8a30d8876a6d/test/language/expressions/unsigned-right-shift/S9.6_A2.2.js>.
        test("-2147483647 >>> 0", "2147483649");
        test("-2147483648 >>> 0", "2147483648");
        test("-2147483649 >>> 0", "2147483647");
        test("-4294967295 >>> 0", "1");
        test("-4294967296 >>> 0", "0");
        test("-4294967297 >>> 0", "4294967295");
        test("4294967295 >>> 0", "4294967295");
        test("4294967296 >>> 0", "0");
        test("4294967297 >>> 0", "1");
        test("8589934591 >>> 0", "4294967295");
        test("8589934592 >>> 0", "0");
        test("8589934593 >>> 0", "1");
    }

    #[test]
    fn test_string_add() {
        test("x = 'a' + 'bc'", "x = 'abc'");
        test("x = 'a' + 5", "x = 'a5'");
        test("x = 5 + 'a'", "x = '5a'");
        // test("x = 'a' + 5n", "x = 'a5n'");
        // test("x = 5n + 'a'", "x = '5na'");
        test("x = 'a' + ''", "x = 'a'");
        test("x = 'a' + foo()", "x = 'a'+foo()");
        test("x = foo() + 'a' + 'b'", "x = foo()+'ab'");
        test("x = (foo() + 'a') + 'b'", "x = foo()+'ab'"); // believe it!
        test("x = foo() + 'a' + 'b' + 'cd' + bar()", "x = foo()+'abcd'+bar()");
        test("x = foo() + 2 + 'b'", "x = foo()+2+\"b\""); // don't fold!

        // test("x = foo() + 'a' + 2", "x = foo()+\"a2\"");
        test("x = '' + null", "x = 'null'");
        test("x = true + '' + false", "x = 'truefalse'");
        // test("x = '' + []", "x = ''");
        // test("x = foo() + 'a' + 1 + 1", "x = foo() + 'a11'");
        test("x = 1 + 1 + 'a'", "x = '2a'");
        test("x = 1 + 1 + 'a'", "x = '2a'");
        test("x = 'a' + (1 + 1)", "x = 'a2'");
        // test("x = '_' + p1 + '_' + ('' + p2)", "x = '_' + p1 + '_' + p2");
        // test("x = 'a' + ('_' + 1 + 1)", "x = 'a_11'");
        // test("x = 'a' + ('_' + 1) + 1", "x = 'a_11'");
        // test("x = 1 + (p1 + '_') + ('' + p2)", "x = 1 + (p1 + '_') + p2");
        // test("x = 1 + p1 + '_' + ('' + p2)", "x = 1 + p1 + '_' + p2");
        // test("x = 1 + 'a' + p1", "x = '1a' + p1");
        // test("x = (p1 + (p2 + 'a')) + 'b'", "x = (p1 + (p2 + 'ab'))");
        // test("'a' + ('b' + p1) + 1", "'ab' + p1 + 1");
        // test("x = 'a' + ('b' + p1 + 'c')", "x = 'ab' + (p1 + 'c')");
        test("void 0 + ''", "'undefined'");

        test_same("x = 'a' + (4 + p1 + 'a')");
        test_same("x = p1 / 3 + 4");
        test_same("foo() + 3 + 'a' + foo()");
        test_same("x = 'a' + ('b' + p1 + p2)");
        test_same("x = 1 + ('a' + p1)");
        test_same("x = p1 + '' + p2");
        test_same("x = 'a' + (1 + p1)");
        test_same("x = (p2 + 'a') + (1 + p1)");
        test_same("x = (p2 + 'a') + (1 + p1 + p2)");
        test_same("x = (p2 + 'a') + (1 + (p1 + p2))");
    }

    #[test]
    fn test_fold_arithmetic() {
        test("1n+ +1n", "1n + +1n");
        test("1n- -1n", "1n - -1n");
        test("a- -b", "a - -b");
    }

    #[test]
    fn test_fold_arithmetic_infinity() {
        test("x=-Infinity-2", "x=-Infinity");
        test("x=Infinity-2", "x=Infinity");
        test("x=Infinity*5", "x=Infinity");
        test("x = Infinity ** 2", "x = Infinity");
        test("x = Infinity ** -2", "x = 0");

        test("x = Infinity % Infinity", "x = NaN");
        test("x = Infinity % 0", "x = NaN");
    }

    #[test]
    fn test_fold_add() {
        test("x = 10 + 20", "x = 30");
        test_same("x = y + 10 + 20");
        test("x = 1 + null", "x = 1");
        test("x = null + 1", "x = 1");
    }

    #[test]
    fn test_fold_multiply() {
        test_same("x = 2.25 * 3");
        test_same("z = x * y");
        test_same("x = y * 5");
        // test("x = null * undefined", "x = NaN");
        // test("x = null * 1", "x = 0");
        // test("x = (null - 1) * 2", "x = -2");
        // test("x = (null + 1) * 2", "x = 2");
        // test("x = y + (z * 24 * 60 * 60 * 1000)", "x = y + z * 864E5");
        test("x = y + (z & 24 & 60 & 60 & 1000)", "x = y + (z & 8)");
    }

    #[test]
    fn test_fold_division() {
        test("x = Infinity / Infinity", "x = NaN");
        test("x = Infinity / 0", "x = Infinity");
        test("x = 1 / 0", "x = Infinity");
        test("x = 0 / 0", "x = NaN");
        test_same("x = 2 / 4");
        test_same("x = y / 2 / 4");
    }

    #[test]
    fn test_fold_remainder() {
        test_same("x = 3 % 2");
        test_same("x = 3 % -2");
        test_same("x = -1 % 3");
        test("x = 1 % 0", "x = NaN");
        test("x = 0 % 0", "x = NaN");
    }

    #[test]
    fn test_fold_exponential() {
        test_same("x = 2 ** 3");
        test_same("x = 2 ** -3");
        test_same("x = 2 ** 55");
        test_same("x = 3 ** -1");
        test_same("x = (-1) ** 0.5");
        test("x = (-0) ** 3", "x = -0");
        test_same("x = null ** 0");
    }

    #[test]
    fn test_fold_shift_left() {
        test("1 << 3", "8");
        test("1.2345 << 0", "1");
        test_same("1 << 24");
    }

    #[test]
    fn test_fold_shift_right() {
        test("2147483647 >> -32.1", "2147483647");
    }

    #[test]
    fn test_fold_shift_right_zero_fill() {
        test("10 >>> 1", "5");
        test_same("-1 >>> 0");
    }

    #[test]
    fn test_fold_left() {
        test_same("(+x - 1) + 2"); // not yet
        test("(+x & 1) & 2", "+x & 0");
    }

    #[test]
    fn test_fold_array_length() {
        // Can fold
        test("x = [].length", "x = 0");
        test("x = [1,2,3].length", "x = 3");
        // test("x = [a,b].length", "x = 2");

        // Not handled yet
        test("x = [,,1].length", "x = 3");

        // Cannot fold
        test("x = [foo(), 0].length", "x = [foo(),0].length");
        test_same("x = y.length");
    }

    #[test]
    fn test_fold_string_length() {
        // Can fold basic strings.
        test("x = ''.length", "x = 0");
        test("x = '123'.length", "x = 3");

        // Test Unicode escapes are accounted for.
        test("x = '123\\u01dc'.length", "x = 4");
    }

    #[test]
    fn test_fold_instance_of() {
        // Non object types are never instances of anything.
        test("64 instanceof Object", "false");
        test("64 instanceof Number", "false");
        test("'' instanceof Object", "false");
        test("'' instanceof String", "false");
        test("true instanceof Object", "false");
        test("true instanceof Boolean", "false");
        test("!0 instanceof Object", "false");
        test("!0 instanceof Boolean", "false");
        test("false instanceof Object", "false");
        test("null instanceof Object", "false");
        test("undefined instanceof Object", "false");
        test("NaN instanceof Object", "false");
        test("Infinity instanceof Object", "false");

        // Array and object literals are known to be objects.
        test("[] instanceof Object", "true");
        test("({}) instanceof Object", "true");

        // These cases is foldable, but no handled currently.
        test_same("new Foo() instanceof Object");
        // These would require type information to fold.
        test_same("[] instanceof Foo");
        test_same("({}) instanceof Foo");

        test("(function() {}) instanceof Object", "true");

        // An unknown value should never be folded.
        test_same("x instanceof Foo");
        test_same("0 instanceof Foo");
    }

    #[test]
    fn test_fold_instance_of_additional() {
        test("(typeof {}) instanceof Object", "false");
        test("(+{}) instanceof Number", "false");
    }

    #[test]
    fn test_fold_left_child_op() {
        test("x & Infinity & 2", "x & 0");
        test_same("x - Infinity - 2"); // FIXME: want "x-Infinity"
        test_same("x - 1 + Infinity");
        test_same("x - 2 + 1");
        test_same("x - 2 + 3");
        test_same("1 + x - 2 + 1");
        test_same("1 + x - 2 + 3");
        test_same("1 + x - 2 + 3 - 1");
        test_same("f(x)-0");
        test_same("x-0-0"); // FIXME: want x - 0
        test_same("x+2-2+2");
        test_same("x+2-2+2-2");
        test_same("x-2+2");
        test_same("x-2+2-2");
        test_same("x-2+2-2+2");

        test_same("1+x-0-na_n");
        test_same("1+f(x)-0-na_n");
        test_same("1+x-0+na_n");
        test_same("1+f(x)-0+na_n");

        test_same("1+x+na_n"); // unfoldable
        test_same("x+2-2"); // unfoldable
        test_same("x+2"); // nothing to do
        test_same("x-2"); // nothing to do
    }

    #[test]
    fn test_associative_fold_constants_with_variables() {
        // mul and add should not fold
        test_same("alert(x * 12 * 20);");
        test_same("alert(12 * x * 20);");
        test_same("alert(x + 12 + 20);");
        test_same("alert(12 + x + 20);");
        test("alert(x & 12 & 20);", "alert(x & 4);");
        test("alert(12 & x & 20);", "alert(x & 4);");
    }

    #[test]
    fn test_to_number() {
        test("x = +''", "x = 0");
        test("x = +'+Infinity'", "x = Infinity");
        test("x = +'-Infinity'", "x = -Infinity");

        for op in ["", "+", "-"] {
            for s in ["inf", "infinity", "INFINITY", "InFiNiTy"] {
                test(&format!("x = +'{op}{s}'"), "x = NaN");
            }
        }
    }

    #[test]
    fn test_number_constructor() {
        test("Number(undefined)", "NaN");
        test("Number(null)", "0");
        test("Number(true)", "1");
        test("Number(false)", "0");
        test("Number('a')", "NaN");
        test("Number('1')", "1");
        test_same("var Number; Number(1)");
    }

    #[test]
    fn test_fold_typeof_addition_string() {
        test_same("typeof foo");
        test_same("typeof foo + '123'");
        test("typeof foo + ''", "typeof foo");
        test_same("typeof foo - ''");
    }

    #[test]
    fn test_fold_same_typeof() {
        test("typeof foo === typeof bar", "typeof foo == typeof bar");
        test("typeof foo !== typeof bar", "typeof foo != typeof bar");
        test("typeof foo.bar === typeof foo.bar", "typeof foo.bar == typeof foo.bar");
        test("typeof foo.bar !== typeof foo.bar", "typeof foo.bar != typeof foo.bar");
    }

    // TODO: All big ints are rare and difficult to handle.
    mod bigint {
        use super::{
            test, test_same, MAX_SAFE_FLOAT, MAX_SAFE_INT, NEG_MAX_SAFE_FLOAT, NEG_MAX_SAFE_INT,
        };

        #[test]
        fn test_fold_bitwise_op_with_big_int() {
            test("x = 1n & 1n", "x = 1n");
            test("x = 1n & 2n", "x = 0n");
            test("x = 3n & 1n", "x = 1n");
            test("x = 3n & 3n", "x = 3n");

            test("x = 1n | 1n", "x = 1n");
            test("x = 1n | 2n", "x = 3n");
            test("x = 1n | 3n", "x = 3n");
            test("x = 3n | 1n", "x = 3n");
            test("x = 3n | 3n", "x = 3n");
            test("x = 1n | 4n", "x = 5n");

            test("x = 1n ^ 1n", "x = 0n");
            test("x = 1n ^ 2n", "x = 3n");
            test("x = 3n ^ 1n", "x = 2n");
            test("x = 3n ^ 3n", "x = 0n");

            // test("x = -1n & 0n", "x = 0n");
            // test("x = 0n & -1n", "x = 0n");
            test("x = 1n & 4n", "x = 0n");
            test("x = 2n & 3n", "x = 2n");

            test("x = 1n & 3000000000n", "x = 0n");
            test("x = 3000000000n & 1n", "x = 0n");

            // bitwise OR does not affect the sign of a bigint
            test("x = 1n | 3000000001n", "x = 3000000001n");
            test("x = 4294967295n | 0n", "x = 4294967295n");

            test("x = y & 1n & 1n", "x = y & 1n");
            test("x = y & 1n & 2n", "x = y & 0n");
            test("x = y & 3n & 1n", "x = y & 1n");
            test("x = 3n & y & 1n", "x = y & 1n");
            test("x = y & 3n & 3n", "x = y & 3n");
            test("x = 3n & y & 3n", "x = y & 3n");

            test("x = y | 1n | 1n", "x = y | 1n");
            test("x = y | 1n | 2n", "x = y | 3n");
            test("x = y | 3n | 1n", "x = y | 3n");
            test("x = 3n | y | 1n", "x = y | 3n");
            test("x = y | 3n | 3n", "x = y | 3n");
            test("x = 3n | y | 3n", "x = y | 3n");

            test("x = y ^ 1n ^ 1n", "x = y ^ 0n");
            test("x = y ^ 1n ^ 2n", "x = y ^ 3n");
            test("x = y ^ 3n ^ 1n", "x = y ^ 2n");
            test("x = 3n ^ y ^ 1n", "x = y ^ 2n");
            test("x = y ^ 3n ^ 3n", "x = y ^ 0n");
            test("x = 3n ^ y ^ 3n", "x = y ^ 0n");

            // TypeError: Cannot mix BigInt and other types
            test_same("1n & 1");
            test_same("1n | 1");
            test_same("1n ^ 1");
        }

        #[test]
        #[ignore]
        fn test_bigint_number_comparison() {
            test("1n < 2", "true");
            test("1n > 2", "false");
            test("1n == 1", "true");
            test("1n == 2", "false");

            // comparing with decimals is allowed
            test("1n < 1.1", "true");
            test("1n < 1.9", "true");
            test("1n < 0.9", "false");
            test("-1n < -1.1", "false");
            test("-1n < -1.9", "false");
            test("-1n < -0.9", "true");
            test("1n > 1.1", "false");
            test("1n > 0.9", "true");
            test("-1n > -1.1", "true");
            test("-1n > -0.9", "false");

            // Don't fold unsafely large numbers because there might be floating-point error
            test(&format!("0n > {MAX_SAFE_INT}"), "false");
            test(&format!("0n < {MAX_SAFE_INT}"), "true");
            test(&format!("0n > {NEG_MAX_SAFE_INT}"), "true");
            test(&format!("0n < {NEG_MAX_SAFE_INT}"), "false");
            test(&format!("0n > {MAX_SAFE_FLOAT}"), "false");
            test(&format!("0n < {MAX_SAFE_FLOAT}"), "true");
            test(&format!("0n > {NEG_MAX_SAFE_FLOAT}"), "true");
            test(&format!("0n < {NEG_MAX_SAFE_FLOAT}"), "false");

            // comparing with Infinity is allowed
            test("1n < Infinity", "true");
            test("1n > Infinity", "false");
            test("1n < -Infinity", "false");
            test("1n > -Infinity", "true");

            // null is interpreted as 0 when comparing with bigint
            // test("1n < null", "false");
            // test("1n > null", "true");
        }

        #[test]
        #[ignore]
        fn test_bigint_string_comparison() {
            test("1n < '2'", "true");
            test("2n > '1'", "true");
            test("123n > '34'", "true");
            test("1n == '1'", "true");
            test("1n == '2'", "false");
            test("1n != '1'", "false");
            test("1n === '1'", "false");
            test("1n !== '1'", "true");
        }

        #[test]
        #[ignore]
        fn test_string_bigint_comparison() {
            test("'1' < 2n", "true");
            test("'2' > 1n", "true");
            test("'123' > 34n", "true");
            test("'1' == 1n", "true");
            test("'1' == 2n", "false");
            test("'1' != 1n", "false");
            test("'1' === 1n", "false");
            test("'1' !== 1n", "true");
        }

        #[test]
        fn test_object_bigint_comparison() {
            test_same("{ valueOf: function() { return 0n; } } != 0n");
            test_same("{ toString: function() { return '0'; } } != 0n");
        }

        #[test]
        fn test_fold_object_spread() {
            test_same("({ z, ...a })");
            let result = "({ z })";
            test("({ z, ...[] })", result);
            test("({ z, ...{} })", result);
            test("({ z, ...undefined })", result);
            test("({ z, ...void 0 })", result);
            test("({ z, ...null })", result);
            test("({ z, ...true })", result);
            test("({ z, ...1 })", result);
            test("({ z, ...1n })", result);
            test("({ z, .../asdf/ })", result);
        }
    }
}
