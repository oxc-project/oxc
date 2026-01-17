use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::{
    GlobalContext, ToJsString,
    constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

/// Constant Folding
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>
impl<'a> PeepholeOptimizations {
    #[expect(clippy::float_cmp)]
    pub fn fold_unary_expr(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::UnaryExpression(e) = expr else { return };
        match e.operator {
            // Do not fold `void 0` back to `undefined`.
            UnaryOperator::Void if e.argument.is_number_0() => {}
            // Do not fold `true` and `false` back to `!0` and `!1`
            UnaryOperator::LogicalNot if matches!(&e.argument, Expression::NumericLiteral(lit) if lit.value == 0.0 || lit.value == 1.0) =>
                {}
            // Do not fold big int.
            UnaryOperator::UnaryNegation if e.argument.is_big_int_literal() => {}
            _ if e.may_have_side_effects(ctx) => {}
            _ => {
                if let Some(changed) = e.evaluate_value(ctx).map(|v| ctx.value_to_expr(e.span, v)) {
                    *expr = changed;
                    ctx.state.changed = true;
                }
            }
        }
    }

    pub fn fold_static_member_expr(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::StaticMemberExpression(e) = expr else { return };
        // TODO: tryFoldObjectPropAccess(n, left, name)
        if e.object.may_have_side_effects(ctx) {
            return;
        }
        if let Some(changed) = e.evaluate_value(ctx).map(|value| ctx.value_to_expr(e.span, value)) {
            *expr = changed;
            ctx.state.changed = true;
        }
    }

    pub fn fold_computed_member_expr(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::ComputedMemberExpression(e) = expr else { return };
        // TODO: tryFoldObjectPropAccess(n, left, name)
        if e.object.may_have_side_effects(ctx) || e.expression.may_have_side_effects(ctx) {
            return;
        }
        if let Some(changed) = e.evaluate_value(ctx).map(|value| ctx.value_to_expr(e.span, value)) {
            *expr = changed;
            ctx.state.changed = true;
        }
    }

    pub fn fold_logical_expr(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::LogicalExpression(e) = expr else { return };
        if let Some(changed) = match e.operator {
            LogicalOperator::And | LogicalOperator::Or => Self::try_fold_and_or(e, ctx),
            LogicalOperator::Coalesce => Self::try_fold_coalesce(e, ctx),
        } {
            *expr = changed;
            ctx.state.changed = true;
        }
    }

    pub fn fold_chain_expr(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::ChainExpression(e) = expr else { return };
        let left_expr = match &e.expression {
            match_member_expression!(ChainElement) => {
                let member_expr = e.expression.to_member_expression();
                if !member_expr.optional() {
                    return;
                }
                member_expr.object()
            }
            ChainElement::CallExpression(call_expr) => {
                if !call_expr.optional {
                    return;
                }
                &call_expr.callee
            }
            ChainElement::TSNonNullExpression(_) => return,
        };
        let ty = left_expr.value_type(ctx);
        if let Some(changed) = (ty.is_null() || ty.is_undefined())
            .then(|| ctx.value_to_expr(e.span, ConstantValue::Undefined))
        {
            *expr = changed;
            ctx.state.changed = true;
        }
    }

    /// Try to fold a AND / OR node.
    ///
    /// port from [closure-compiler](https://github.com/google/closure-compiler/blob/09094b551915a6487a980a783831cba58b5739d1/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L587)
    pub fn try_fold_and_or(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let op = logical_expr.operator;
        debug_assert!(matches!(op, LogicalOperator::And | LogicalOperator::Or));

        let left = &logical_expr.left;
        let left_val = left.evaluate_value_to_boolean(ctx);

        if let Some(lval) = left_val {
            // (TRUE || x) => TRUE (also, (3 || x) => 3)
            // (FALSE && x) => FALSE
            if if lval { op.is_or() } else { op.is_and() } {
                return Some(logical_expr.left.take_in(ctx.ast));
            } else if !left.may_have_side_effects(ctx) {
                let should_keep_indirect_access =
                    Self::should_keep_indirect_access(&logical_expr.right, ctx);
                // (true && o.f) => (0, o.f)
                if should_keep_indirect_access {
                    return Some(ctx.ast.expression_sequence(
                        logical_expr.span,
                        ctx.ast.vec_from_array([
                            ctx.ast.expression_numeric_literal(
                                logical_expr.left.span(),
                                0.0,
                                None,
                                NumberBase::Decimal,
                            ),
                            logical_expr.right.take_in(ctx.ast),
                        ]),
                    ));
                }
                // (FALSE || x) => x
                // (TRUE && x) => x
                return Some(logical_expr.right.take_in(ctx.ast));
            }
            // Left side may have side effects, but we know its boolean value.
            // e.g. true_with_sideeffects || foo() => true_with_sideeffects, foo()
            // or: false_with_sideeffects && foo() => false_with_sideeffects, foo()
            let left = logical_expr.left.take_in(ctx.ast);
            let right = logical_expr.right.take_in(ctx.ast);
            let vec = ctx.ast.vec_from_array([left, right]);
            let sequence_expr = ctx.ast.expression_sequence(logical_expr.span, vec);
            return Some(sequence_expr);
        } else if let Expression::LogicalExpression(left_child) = &mut logical_expr.left
            && left_child.operator == logical_expr.operator
        {
            let left_child_right_boolean = left_child.right.evaluate_value_to_boolean(ctx);
            let left_child_op = left_child.operator;
            if let Some(right_boolean) = left_child_right_boolean
                && !left_child.right.may_have_side_effects(ctx)
            {
                // a || false || b => a || b
                // a && true && b => a && b
                if !right_boolean && left_child_op.is_or()
                    || right_boolean && left_child_op.is_and()
                {
                    let left = left_child.left.take_in(ctx.ast);
                    let right = logical_expr.right.take_in(ctx.ast);
                    let logic_expr =
                        ctx.ast.expression_logical(logical_expr.span, left, left_child_op, right);
                    return Some(logic_expr);
                }
            }
        }
        None
    }

    /// Try to fold a nullish coalesce `foo ?? bar`.
    pub fn try_fold_coalesce(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        debug_assert_eq!(logical_expr.operator, LogicalOperator::Coalesce);
        let left = &logical_expr.left;
        let left_val = left.value_type(ctx);
        match left_val {
            ValueType::Null | ValueType::Undefined => {
                Some(if left.may_have_side_effects(ctx) {
                    // e.g. `(a(), null) ?? 1` => `(a(), null, 1)`
                    let expressions = ctx.ast.vec_from_array([
                        logical_expr.left.take_in(ctx.ast),
                        logical_expr.right.take_in(ctx.ast),
                    ]);
                    ctx.ast.expression_sequence(logical_expr.span, expressions)
                } else {
                    let should_keep_indirect_access =
                        Self::should_keep_indirect_access(&logical_expr.right, ctx);
                    // (null ?? o.f) => (0, o.f)
                    if should_keep_indirect_access {
                        return Some(ctx.ast.expression_sequence(
                            logical_expr.span,
                            ctx.ast.vec_from_array([
                                ctx.ast.expression_numeric_literal(
                                    logical_expr.left.span(),
                                    0.0,
                                    None,
                                    NumberBase::Decimal,
                                ),
                                logical_expr.right.take_in(ctx.ast),
                            ]),
                        ));
                    }
                    // nullish condition => this expression evaluates to the right side.
                    logical_expr.right.take_in(ctx.ast)
                })
            }
            ValueType::Number
            | ValueType::BigInt
            | ValueType::String
            | ValueType::Boolean
            | ValueType::Object => {
                let should_keep_indirect_access =
                    Self::should_keep_indirect_access(&logical_expr.left, ctx);
                // (o.f ?? something) => (0, o.f)
                if should_keep_indirect_access {
                    return Some(ctx.ast.expression_sequence(
                        logical_expr.span,
                        ctx.ast.vec_from_array([
                            ctx.ast.expression_numeric_literal(
                                logical_expr.right.span(),
                                0.0,
                                None,
                                NumberBase::Decimal,
                            ),
                            logical_expr.left.take_in(ctx.ast),
                        ]),
                    ));
                }
                // non-nullish condition => this expression evaluates to the left side.
                Some(logical_expr.left.take_in(ctx.ast))
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
    pub fn fold_binary_expr(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::BinaryExpression(e) = expr else { return };
        // TODO: tryReduceOperandsForOp

        // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L1136
        // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L1222
        let span = e.span;
        let changed = match e.operator {
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::ShiftRight
            | BinaryOperator::Instanceof => ctx.eval_binary(e),
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
                        // Small number multiplication.
                        || (e.operator == BinaryOperator::Multiplication
                            && left.abs() <= 255.0
                            && left.fract() == 0.0
                            && right.abs() <= 255.0
                            && right.fract() == 0.0)
                })
                .and_then(|_| ctx.eval_binary(e)),
            BinaryOperator::Division => Self::extract_numeric_values(e)
                .filter(|(_, right)| *right == 0.0 || right.is_nan() || right.is_infinite())
                .and_then(|_| ctx.eval_binary(e)),
            BinaryOperator::ShiftLeft => {
                Self::extract_numeric_values(e).and_then(|(left, right)| {
                    let result = e.evaluate_value(ctx)?.into_number()?;
                    let left_len = Self::approximate_printed_int_char_count(left);
                    let right_len = Self::approximate_printed_int_char_count(right);
                    let result_len = Self::approximate_printed_int_char_count(result);
                    (result_len <= left_len + 2 + right_len)
                        .then(|| ctx.value_to_expr(span, ConstantValue::Number(result)))
                })
            }
            BinaryOperator::ShiftRightZeroFill => {
                Self::extract_numeric_values(e).and_then(|(left, right)| {
                    let result = e.evaluate_value(ctx)?.into_number()?;
                    let left_len = Self::approximate_printed_int_char_count(left);
                    let right_len = Self::approximate_printed_int_char_count(right);
                    let result_len = Self::approximate_printed_int_char_count(result);
                    (result_len <= left_len + 3 + right_len)
                        .then(|| ctx.value_to_expr(span, ConstantValue::Number(result)))
                })
            }
            BinaryOperator::In => None,
        };
        if let Some(changed) = changed {
            *expr = changed;
            ctx.state.changed = true;
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
    fn try_fold_add(e: &mut BinaryExpression<'a>, ctx: &Ctx<'a, '_>) -> Option<Expression<'a>> {
        if !e.may_have_side_effects(ctx)
            && let Some(v) = e.evaluate_value(ctx)
        {
            return Some(ctx.value_to_expr(e.span, v));
        }
        debug_assert_eq!(e.operator, BinaryOperator::Addition);

        if let Some(expr) = Self::try_fold_add_op(&mut e.left, &mut e.right, e.span, ctx) {
            return Some(expr);
        }

        // a + 'b' + 'c' -> a + 'bc'
        if let Expression::BinaryExpression(left_binary_expr) = &mut e.left
            && left_binary_expr.right.value_type(ctx).is_string()
        {
            if let (Some(left_str), Some(right_str)) = (
                left_binary_expr.right.get_side_free_string_value(ctx),
                e.right.get_side_free_string_value(ctx),
            ) {
                let span = left_binary_expr
                    .right
                    .span()
                    .merge_within(e.right.span(), e.span)
                    .unwrap_or(SPAN);
                let value = ctx.ast.atom_from_strs_array([&left_str, &right_str]);
                let right = ctx.ast.expression_string_literal(span, value, None);
                let left = left_binary_expr.left.take_in(ctx.ast);
                return Some(ctx.ast.expression_binary(e.span, left, e.operator, right));
            }

            if let Some(new_right) =
                Self::try_fold_add_op(&mut left_binary_expr.right, &mut e.right, e.span, ctx)
            {
                let left = left_binary_expr.left.take_in(ctx.ast);
                return Some(ctx.ast.expression_binary(e.span, left, e.operator, new_right));
            }
        }

        None
    }

    fn try_fold_add_op(
        left_expr: &mut Expression<'a>,
        right_expr: &mut Expression<'a>,
        parent_span: Span,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if let Expression::TemplateLiteral(left) = left_expr {
            // "`${a}b` + `x${y}`" => "`${a}bx${y}`"
            if let Expression::TemplateLiteral(right) = right_expr {
                left.span = left.span.merge_within(right.span, parent_span).unwrap_or(SPAN);
                let left_last_quasi =
                    left.quasis.last_mut().expect("template literal must have at least one quasi");
                let right_first_quasi = right
                    .quasis
                    .first_mut()
                    .expect("template literal must have at least one quasi");
                let new_raw = left_last_quasi.value.raw.to_string() + &right_first_quasi.value.raw;
                left_last_quasi.value.raw = ctx.ast.atom(&new_raw);
                let new_cooked = if let (Some(cooked1), Some(cooked2)) =
                    (left_last_quasi.value.cooked, right_first_quasi.value.cooked)
                {
                    Some(ctx.ast.atom(&(cooked1.into_string() + cooked2.as_str())))
                } else {
                    None
                };
                left_last_quasi.value.cooked = new_cooked;
                if !right.quasis.is_empty() {
                    left_last_quasi.tail = false;
                }
                left.quasis.extend(right.quasis.drain(1..)); // first quasi is already handled
                left.expressions.extend(right.expressions.drain(..));
                return Some(left_expr.take_in(ctx.ast));
            }

            // "`${x}y` + 'z'" => "`${x}yz`"
            if let Some(right_str) = right_expr.get_side_free_string_value(ctx) {
                left.span = left.span.merge_within(right_expr.span(), parent_span).unwrap_or(SPAN);
                let last_quasi =
                    left.quasis.last_mut().expect("template literal must have at least one quasi");
                let new_raw = last_quasi.value.raw.to_string()
                    + &Self::escape_string_for_template_literal(&right_str);
                last_quasi.value.raw = ctx.ast.atom(&new_raw);
                let new_cooked = last_quasi
                    .value
                    .cooked
                    .map(|cooked| ctx.ast.atom(&(cooked.as_str().to_string() + &right_str)));
                last_quasi.value.cooked = new_cooked;
                return Some(left_expr.take_in(ctx.ast));
            }
        } else if let Expression::TemplateLiteral(right) = right_expr {
            // "'x' + `y${z}`" => "`xy${z}`"
            if let Some(left_str) = left_expr.get_side_free_string_value(ctx) {
                right.span = right.span.merge_within(left_expr.span(), parent_span).unwrap_or(SPAN);
                let first_quasi = right
                    .quasis
                    .first_mut()
                    .expect("template literal must have at least one quasi");
                let new_raw = Self::escape_string_for_template_literal(&left_str).into_owned()
                    + first_quasi.value.raw.as_str();
                first_quasi.value.raw = ctx.ast.atom(&new_raw);
                let new_cooked = first_quasi
                    .value
                    .cooked
                    .map(|cooked| ctx.ast.atom(&(left_str.into_owned() + cooked.as_str())));
                first_quasi.value.cooked = new_cooked;
                return Some(right_expr.take_in(ctx.ast));
            }
        }

        // remove useless `+ ""` (e.g. `typeof foo + ""` -> `typeof foo`)
        if Self::evaluates_to_empty_string(left_expr) && right_expr.value_type(ctx).is_string() {
            return Some(right_expr.take_in(ctx.ast));
        } else if Self::evaluates_to_empty_string(right_expr)
            && left_expr.value_type(ctx).is_string()
        {
            return Some(left_expr.take_in(ctx.ast));
        }

        None
    }

    fn evaluates_to_empty_string(e: &Expression<'a>) -> bool {
        match e {
            Expression::StringLiteral(s) => s.value.is_empty(),
            Expression::ArrayExpression(a) => a.elements.is_empty(),
            _ => false,
        }
    }

    fn try_fold_left_child_op(
        e: &mut BinaryExpression<'a>,
        ctx: &Ctx<'a, '_>,
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
            expr_to_move.take_in(ctx.ast),
            op,
            ctx.value_to_expr(
                left.right.span().merge_within(e.right.span(), e.span).unwrap_or(SPAN),
                v,
            ),
        ))
    }

    pub fn fold_call_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::CallExpression(e) = expr else { return };
        if !ctx.is_global_expr("Number", &e.callee) {
            return;
        }
        if e.arguments.len() != 1 {
            return;
        }
        let Some(arg) = e.arguments[0].as_expression() else { return };
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
                if let Some(n) = arg.evaluate_value_to_number(ctx) {
                    n
                } else {
                    *expr = ctx.ast.expression_unary(
                        e.span,
                        UnaryOperator::UnaryPlus,
                        ctx.ast.expression_string_literal(n.span, n.value, n.raw),
                    );
                    ctx.state.changed = true;
                    return;
                }
            }
            e if e.is_void_0() => f64::NAN,
            _ => return,
        });
        *expr = ctx.value_to_expr(e.span, value);
        ctx.state.changed = true;
    }

    pub fn fold_binary_typeof_comparison(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::BinaryExpression(e) = expr else { return };
        // `typeof a == typeof a` -> `true`, `typeof a != typeof a` -> `false`
        if e.operator.is_equality()
            && let (Expression::UnaryExpression(left), Expression::UnaryExpression(right)) =
                (&e.left, &e.right)
            && left.operator.is_typeof()
            && right.operator.is_typeof()
            && let (Expression::Identifier(left_ident), Expression::Identifier(right_ident)) =
                (&left.argument, &right.argument)
            && left_ident.name == right_ident.name
        {
            let b = matches!(e.operator, BinaryOperator::StrictEquality | BinaryOperator::Equality);
            *expr = ctx.ast.expression_boolean_literal(e.span, b);
            ctx.state.changed = true;
            return;
        }

        // `typeof a === 'asd` -> `false``
        // `typeof a !== 'b'` -> `true``
        if let Expression::UnaryExpression(left) = &e.left
            && left.operator.is_typeof()
            && e.operator.is_equality()
        {
            let right_ty = e.right.value_type(ctx);

            if !right_ty.is_undetermined() && right_ty != ValueType::String {
                *expr = ctx.ast.expression_boolean_literal(
                    e.span,
                    e.operator == BinaryOperator::Inequality
                        || e.operator == BinaryOperator::StrictInequality,
                );
                ctx.state.changed = true;
                return;
            }
            if let Expression::StringLiteral(string_lit) = &e.right
                && !matches!(
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
                )
            {
                *expr = ctx.ast.expression_boolean_literal(
                    e.span,
                    e.operator == BinaryOperator::Inequality
                        || e.operator == BinaryOperator::StrictInequality,
                );
                ctx.state.changed = true;
            }
        }
    }

    pub fn fold_object_exp(e: &mut ObjectExpression<'a>, ctx: &mut Ctx<'a, '_>) {
        fn should_fold_spread_element<'a>(e: &Expression<'a>, ctx: &Ctx<'a, '_>) -> bool {
            match e {
                Expression::ArrayExpression(o) if o.elements.is_empty() => true,
                Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
                e if e.is_literal() && !e.is_string_literal() => true,
                e if e.evaluate_value(ctx).is_some_and(|v| !v.is_string())
                    && !e.may_have_side_effects(ctx) =>
                {
                    true
                }
                _ => false,
            }
        }
        let (new_size, should_fold) =
            e.properties.iter().fold((0, false), |(new_size, should_fold), p| {
                let ObjectPropertyKind::SpreadProperty(spread_element) = p else {
                    return (new_size + 1, should_fold);
                };
                match &spread_element.argument {
                    Expression::ObjectExpression(o)
                        if Self::is_spread_inlineable_object_literal(o, ctx) =>
                    {
                        (new_size + o.properties.len(), true)
                    }
                    e if should_fold_spread_element(e, ctx) => (new_size, true),
                    _ => (new_size + 1, should_fold),
                }
            });
        if !should_fold {
            return;
        }

        let mut new_properties = ctx.ast.vec_with_capacity::<ObjectPropertyKind>(new_size);
        for p in e.properties.drain(..) {
            if let ObjectPropertyKind::SpreadProperty(mut spread_element) = p {
                let e = &mut spread_element.argument;
                if ctx.is_expression_undefined(e) {
                    continue;
                }
                match e {
                    Expression::ObjectExpression(o)
                        if Self::is_spread_inlineable_object_literal(o, ctx) =>
                    {
                        new_properties.extend(o.properties.drain(..).filter(|prop| {
                            match prop {
                                ObjectPropertyKind::SpreadProperty(_) => true,
                                ObjectPropertyKind::ObjectProperty(p) => {
                                    // non-computed __proto__ property sets the prototype of the object instead
                                    p.computed
                                        || p.method
                                        || !p.key.is_specific_static_name("__proto__")
                                }
                            }
                        }));
                    }
                    e if should_fold_spread_element(e, ctx) => {
                        // skip
                    }
                    _ => {
                        new_properties.push(ObjectPropertyKind::SpreadProperty(spread_element));
                    }
                }
            } else {
                new_properties.push(p);
            }
        }

        e.properties = new_properties;
        ctx.state.changed = true;
    }

    fn is_spread_inlineable_object_literal(e: &ObjectExpression<'a>, ctx: &Ctx<'a, '_>) -> bool {
        e.properties.iter().all(|p| match p {
            ObjectPropertyKind::SpreadProperty(_) => true,
            ObjectPropertyKind::ObjectProperty(p) => {
                // getters are evaluated when spreading
                matches!(p.kind, PropertyKind::Init)
                    && (
                        // non-computed __proto__ property sets the prototype of the object instead
                        p.computed
                            || p.method
                            || !p.key.is_specific_static_name("__proto__")
                            || !p.value.may_have_side_effects(ctx)
                    )
            }
        })
    }

    /// Inline constant values in template literals
    ///
    /// - `foo${1}bar${i}` => `foo1bar${i}`
    pub fn inline_template_literal(t: &mut TemplateLiteral<'a>, ctx: &mut Ctx<'a, '_>) {
        let has_expr_to_inline = t
            .expressions
            .iter()
            .any(|expr| !expr.may_have_side_effects(ctx) && expr.to_js_string(ctx).is_some());
        if !has_expr_to_inline {
            return;
        }

        let mut inline_exprs = Vec::new();
        let new_exprs =
            ctx.ast.vec_from_iter(t.expressions.drain(..).enumerate().filter_map(|(idx, expr)| {
                if expr.may_have_side_effects(ctx) {
                    Some(expr)
                } else if let Some(str) = expr.to_js_string(ctx) {
                    inline_exprs.push((idx, str));
                    None
                } else {
                    Some(expr)
                }
            }));
        t.expressions = new_exprs;

        // inline the extracted inline-able expressions into quasis
        // "current_quasis + extracted_value + next_quasis"
        for (i, (idx, str)) in inline_exprs.into_iter().enumerate() {
            let idx = idx - i;
            let next_quasi = (idx + 1 < t.quasis.len()).then(|| t.quasis.remove(idx + 1));
            let quasi = &mut t.quasis[idx];
            let new_raw = quasi.value.raw.into_string()
                + &Self::escape_string_for_template_literal(&str)
                + next_quasi.as_ref().map(|q| q.value.raw.as_str()).unwrap_or_default();
            quasi.value.raw = ctx.ast.atom(&new_raw);
            let new_cooked = if let (Some(cooked1), Some(cooked2)) =
                (quasi.value.cooked, next_quasi.as_ref().map(|q| q.value.cooked))
            {
                let v =
                    cooked1.into_string() + &str + cooked2.map(|c| c.as_str()).unwrap_or_default();
                Some(ctx.ast.atom(&v))
            } else {
                None
            };
            quasi.value.cooked = new_cooked;
            if next_quasi.is_some_and(|q| q.tail) {
                quasi.tail = true;
            }
        }

        ctx.state.changed = true;
    }
}
