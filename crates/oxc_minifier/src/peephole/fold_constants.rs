use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::{
    GlobalContext, ToJsString,
    constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::TraverseCtx;

use super::PeepholeOptimizations;

/// Scan a string for the lone surrogate encoding pattern `\u{FFFD}XXXX` where
/// XXXX is in the surrogate range (d800–dfff) or the self-escape "fffd".
///
/// **Warning:** This can produce false positives if the string naturally contains
/// U+FFFD followed by 4 hex chars that happen to be in the surrogate range or "fffd".
/// Prefer [`expr_has_lone_surrogates`] when the source AST node is available, since
/// that checks the `lone_surrogates` flag directly without false positives.
///
/// This function is the fallback for paths where only the string value is available
/// (e.g. `value_to_expr` receiving a `ConstantValue::String` with no origin info).
///
/// See [`StringLiteral::lone_surrogates`] for the encoding scheme.
pub fn scan_for_lone_surrogate_encoding(s: &str) -> bool {
    let bytes = s.as_bytes();
    // 0xEF is the leading byte of U+FFFD's UTF-8 encoding (0xEF 0xBF 0xBD) and
    // is rare in typical JS strings. Skip the windowed scan entirely if absent.
    if !bytes.contains(&0xEF) {
        return false;
    }
    // Need 7 bytes: 3 for U+FFFD + 4 hex chars.
    bytes.windows(7).any(|w| w[..3] == [0xEF, 0xBF, 0xBD] && is_lone_surrogate_suffix(&w[3..]))
}

/// Check if 4 bytes are a valid lone surrogate encoding suffix:
/// either a surrogate code point (d800–dfff) or the U+FFFD self-escape (fffd).
fn is_lone_surrogate_suffix(b: &[u8]) -> bool {
    debug_assert_eq!(b.len(), 4);
    // Surrogate range d800–dfff: first char 'd', second '8'–'f', rest hex.
    (b[0] == b'd'
        && matches!(b[1], b'8'..=b'9' | b'a'..=b'f')
        && matches!(b[2], b'0'..=b'9' | b'a'..=b'f')
        && matches!(b[3], b'0'..=b'9' | b'a'..=b'f'))
        || b == b"fffd"
}

/// Check if an expression's string value may contain lone surrogates.
///
/// Based on AST node flags, so not prone to the false positives that
/// [`scan_for_lone_surrogate_encoding`] can produce. For identifiers, looks
/// up the symbol table to check the initializer's flag.
///
/// Called on source subexpressions of a string-producing fold. Must cover
/// every kind whose `to_js_string` / `evaluate_value_to_string` result can
/// contain the lone-surrogate encoding bytes:
///
/// - `StringLiteral` — flag on the node.
/// - `TemplateLiteral` — flags on quasis, recurse on expressions.
/// - `Identifier` — resolve through the symbol table.
/// - `BinaryExpression(+)` — recurse on both sides.
/// - `ArrayExpression` — recurse on element expressions (`array_join`
///   concatenates element `to_js_string` values, so a lone-surrogate element
///   ends up in `String([...])` and `` `${[...]}` `` results).
/// - `CallExpression` — conservative: recurse on callee's object (for method
///   calls) and arguments. False positives are harmless because
///   `correct_lone_surrogates_flag` only acts when the byte scan also flagged
///   the result.
///
/// Other `to_js_string` producers are safe-by-shape — their result is a
/// fixed ASCII string (`"undefined"`, `"true"`, `"false"`, `"null"`,
/// `"[object Object]"`) or a number/BigInt/regex-source representation — and
/// return `false` trivially.
///
/// `LogicalExpression`, `ConditionalExpression`, `SequenceExpression`, and
/// `Static/ComputedMemberExpression` also yield `false` here. That is safe
/// by traversal order: exit-order peephole folds any foldable
/// string-yielding child into a literal before its parent reaches this
/// helper's call sites, so they never arrive here with a string value. See
/// `test_lone_surrogate_through_non_literal_subexprs`.
///
/// If either invariant breaks — a newly string-producing kind is added to
/// `to_js_string`, or a parent is folded before its children — the byte
/// scan in `value_to_expr` would flag the result and
/// `correct_lone_surrogates_flag` would silently clear it back, producing
/// wrong codegen.
pub fn expr_has_lone_surrogates(expr: &Expression, ctx: &TraverseCtx) -> bool {
    match expr {
        Expression::StringLiteral(s) => s.lone_surrogates,
        Expression::TemplateLiteral(t) => {
            t.quasis.iter().any(|q| q.lone_surrogates)
                || t.expressions.iter().any(|e| expr_has_lone_surrogates(e, ctx))
        }
        Expression::BinaryExpression(e) if e.operator == BinaryOperator::Addition => {
            expr_has_lone_surrogates(&e.left, ctx) || expr_has_lone_surrogates(&e.right, ctx)
        }
        Expression::ArrayExpression(arr) => arr
            .elements
            .iter()
            .any(|el| el.as_expression().is_some_and(|e| expr_has_lone_surrogates(e, ctx))),
        Expression::Identifier(ident) => ident
            .reference_id
            .get()
            .and_then(|rid| ctx.scoping().get_reference(rid).symbol_id())
            .and_then(|sid| ctx.state.symbol_values.get_symbol_value(sid))
            .is_some_and(|sv| sv.lone_surrogates),
        Expression::CallExpression(call) => {
            // Conservatively true if the receiver or any argument carries
            // lone surrogates, even when the callee doesn't actually reflect
            // them into its return value (e.g. `arr.find('\uDC00')`). False
            // positives here are harmless: `correct_lone_surrogates_flag` only
            // acts when the byte scan also flagged the result.
            let object_has = match &call.callee {
                Expression::StaticMemberExpression(m) => expr_has_lone_surrogates(&m.object, ctx),
                Expression::ComputedMemberExpression(m) => expr_has_lone_surrogates(&m.object, ctx),
                _ => false,
            };
            object_has
                || call
                    .arguments
                    .iter()
                    .any(|a| a.as_expression().is_some_and(|e| expr_has_lone_surrogates(e, ctx)))
        }
        _ => false,
    }
}

/// Correct the `lone_surrogates` flag on a [`TraverseCtx::value_to_expr`] result.
///
/// `value_to_expr` uses [`scan_for_lone_surrogate_encoding`] which can yield
/// false positives when U+FFFD naturally appears before surrogate-range hex
/// chars. When the authoritative answer is available (from AST flags or the
/// symbol table), call this to override the scan's false positive.
///
/// `lone_surrogates` is a closure so the (potentially expensive)
/// [`expr_has_lone_surrogates`] AST walk only runs in the rare case where
/// the byte scan flagged the result.
pub fn correct_lone_surrogates_flag(
    result: &mut Expression,
    lone_surrogates: impl FnOnce() -> bool,
) {
    if let Expression::StringLiteral(lit) = result
        && lit.lone_surrogates
    {
        lit.lone_surrogates = lone_surrogates();
    }
}

/// Constant Folding
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>
impl<'a> PeepholeOptimizations {
    #[expect(clippy::float_cmp)]
    pub fn fold_unary_expr(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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

    pub fn fold_static_member_expr(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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

    pub fn fold_computed_member_expr(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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

    pub fn fold_logical_expr(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::LogicalExpression(e) = expr else { return };
        if let Some(changed) = match e.operator {
            LogicalOperator::And | LogicalOperator::Or => Self::try_fold_and_or(e, ctx),
            LogicalOperator::Coalesce => Self::try_fold_coalesce(e, ctx),
        } {
            *expr = changed;
            ctx.state.changed = true;
        }
    }

    pub fn fold_chain_expr(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ChainExpression(e) = expr else { return };
        let span = e.span;
        match try_fold_chain_at_element(&mut e.expression, ctx) {
            ChainFold::Unfolded { .. } => {}
            ChainFold::Flipped { has_optional } => {
                // For `(known_obj)?.foo?.bar` the inner `?.` flips, but the
                // outer `?.bar` keeps the wrapper alive.
                if !has_optional {
                    *expr = Expression::from(e.expression.take_in(ctx.ast));
                }
                ctx.state.changed = true;
            }
            ChainFold::Collapse { base, base_has_side_effects } => {
                *expr = if base_has_side_effects {
                    ctx.ast.expression_sequence(
                        span,
                        ctx.ast.vec_from_array([base, ctx.ast.void_0(span)]),
                    )
                } else {
                    ctx.value_to_expr(span, ConstantValue::Undefined)
                };
                ctx.state.changed = true;
            }
        }
    }

    /// Try to fold a AND / OR node.
    ///
    /// port from [closure-compiler](https://github.com/google/closure-compiler/blob/09094b551915a6487a980a783831cba58b5739d1/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L587)
    pub fn try_fold_and_or(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: &TraverseCtx<'a>,
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
        ctx: &TraverseCtx<'a>,
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
    pub fn fold_binary_expr(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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
            1 + value.abs().log10().floor() as usize
        };
        if value.is_sign_negative() {
            count += 1;
        }
        count
    }

    // Simplified version of `tryFoldAdd` from closure compiler.
    fn try_fold_add(e: &mut BinaryExpression<'a>, ctx: &TraverseCtx<'a>) -> Option<Expression<'a>> {
        if !e.may_have_side_effects(ctx)
            && let Some(v) = e.evaluate_value(ctx)
        {
            let mut result = ctx.value_to_expr(e.span, v);
            correct_lone_surrogates_flag(&mut result, || {
                expr_has_lone_surrogates(&e.left, ctx) || expr_has_lone_surrogates(&e.right, ctx)
            });
            return Some(result);
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
                let value = ctx.ast.str_from_strs_array([&left_str, &right_str]);
                let lone_surrogates = expr_has_lone_surrogates(&left_binary_expr.right, ctx)
                    || expr_has_lone_surrogates(&e.right, ctx);
                let right = ctx.ast.expression_string_literal_with_lone_surrogates(
                    span,
                    value,
                    None,
                    lone_surrogates,
                );
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
        ctx: &TraverseCtx<'a>,
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
                left_last_quasi.value.raw = ctx.ast.str_from_strs_array([
                    left_last_quasi.value.raw.as_str(),
                    right_first_quasi.value.raw.as_str(),
                ]);
                let new_cooked = if let (Some(cooked1), Some(cooked2)) =
                    (left_last_quasi.value.cooked, right_first_quasi.value.cooked)
                {
                    Some(ctx.ast.str_from_strs_array([cooked1.as_str(), cooked2.as_str()]))
                } else {
                    None
                };
                left_last_quasi.value.cooked = new_cooked;
                left_last_quasi.lone_surrogates =
                    left_last_quasi.lone_surrogates || right_first_quasi.lone_surrogates;
                if !right.quasis.is_empty() {
                    left_last_quasi.tail = false;
                }
                left.quasis.extend(right.quasis.drain(1..)); // first quasi is already handled
                left.expressions.extend(right.expressions.drain(..));
                return Some(left_expr.take_in(ctx.ast));
            }

            // "`${x}y` + 'z'" => "`${x}yz`"
            if let Some(right_str) = right_expr.get_side_free_string_value(ctx) {
                // Encoded lone surrogates can't go into template raw values.
                if expr_has_lone_surrogates(right_expr, ctx) {
                    return None;
                }
                left.span = left.span.merge_within(right_expr.span(), parent_span).unwrap_or(SPAN);
                let last_quasi =
                    left.quasis.last_mut().expect("template literal must have at least one quasi");
                let new_raw = last_quasi.value.raw.to_string()
                    + &Self::escape_string_for_template_literal(&right_str);
                last_quasi.value.raw = ctx.ast.str(&new_raw);
                let new_cooked = last_quasi
                    .value
                    .cooked
                    .map(|cooked| ctx.ast.str(&(cooked.as_str().to_string() + &right_str)));
                last_quasi.value.cooked = new_cooked;
                return Some(left_expr.take_in(ctx.ast));
            }
        } else if let Expression::TemplateLiteral(right) = right_expr {
            // "'x' + `y${z}`" => "`xy${z}`"
            if let Some(left_str) = left_expr.get_side_free_string_value(ctx) {
                // Encoded lone surrogates can't go into template raw values.
                if expr_has_lone_surrogates(left_expr, ctx) {
                    return None;
                }
                right.span = right.span.merge_within(left_expr.span(), parent_span).unwrap_or(SPAN);
                let first_quasi = right
                    .quasis
                    .first_mut()
                    .expect("template literal must have at least one quasi");
                let new_raw = Self::escape_string_for_template_literal(&left_str).into_owned()
                    + first_quasi.value.raw.as_str();
                first_quasi.value.raw = ctx.ast.str(&new_raw);
                let new_cooked = first_quasi
                    .value
                    .cooked
                    .map(|cooked| ctx.ast.str(&(left_str.into_owned() + cooked.as_str())));
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
        ctx: &TraverseCtx<'a>,
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

    pub fn fold_call_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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
                        ctx.ast.expression_string_literal_with_lone_surrogates(
                            n.span,
                            n.value,
                            n.raw,
                            n.lone_surrogates,
                        ),
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

    pub fn fold_binary_typeof_comparison(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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

    pub fn fold_object_exp(e: &mut ObjectExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        fn should_fold_spread_element<'a>(e: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
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

    fn is_spread_inlineable_object_literal(
        e: &ObjectExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
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
    pub fn inline_template_literal(t: &mut TemplateLiteral<'a>, ctx: &mut TraverseCtx<'a>) {
        // Pre-check: cheap skip when no expression is foldable. The main loop
        // below also rejects lone-surrogate expressions, so don't repeat that
        // check here — false positives just lead to a no-op walk in the main
        // loop, which is fine.
        let has_expr_to_inline = t
            .expressions
            .iter()
            .any(|expr| !expr.may_have_side_effects(ctx) && expr.to_js_string(ctx).is_some());
        if !has_expr_to_inline {
            return;
        }

        let mut inline_exprs = Vec::with_capacity(t.expressions.len());
        let new_exprs =
            ctx.ast.vec_from_iter(t.expressions.drain(..).enumerate().filter_map(|(idx, expr)| {
                if expr.may_have_side_effects(ctx) {
                    Some(expr)
                } else if let Some(str) = expr.to_js_string(ctx) {
                    // Encoded lone surrogates can't go into template raw values.
                    if expr_has_lone_surrogates(&expr, ctx) {
                        return Some(expr);
                    }
                    inline_exprs.push((idx, str));
                    None
                } else {
                    Some(expr)
                }
            }));
        t.expressions = new_exprs;

        // The pre-check is a fast over-approximation that doesn't reject
        // lone-surrogate expressions — if every foldable expression had lone
        // surrogates, `inline_exprs` is empty and there's nothing to do.
        // (Without this guard, `ctx.state.changed = true` below would loop the
        // compressor.)
        if inline_exprs.is_empty() {
            return;
        }

        // inline the extracted inline-able expressions into quasis
        // "current_quasis + extracted_value + next_quasis"
        for (i, (idx, str)) in inline_exprs.into_iter().enumerate() {
            let idx = idx - i;
            let next_quasi = (idx + 1 < t.quasis.len()).then(|| t.quasis.remove(idx + 1));
            let quasi = &mut t.quasis[idx];
            let escaped = Self::escape_string_for_template_literal(&str);
            let next_raw = next_quasi.as_ref().map(|q| q.value.raw.as_str()).unwrap_or_default();
            quasi.value.raw =
                ctx.ast.str_from_strs_array([quasi.value.raw.as_str(), &escaped, next_raw]);
            let new_cooked = if let (Some(cooked1), Some(cooked2)) =
                (quasi.value.cooked, next_quasi.as_ref().map(|q| q.value.cooked))
            {
                let cooked2_str = cooked2.map(|c| c.as_str()).unwrap_or_default();
                Some(ctx.ast.str_from_strs_array([cooked1.as_str(), &str, cooked2_str]))
            } else {
                None
            };
            quasi.value.cooked = new_cooked;
            if next_quasi.as_ref().is_some_and(|q| q.lone_surrogates) {
                quasi.lone_surrogates = true;
            }
            if next_quasi.is_some_and(|q| q.tail) {
                quasi.tail = true;
            }
        }

        ctx.state.changed = true;
    }
}

/// Outcome of folding an optional-chain at the deepest optional position.
///
/// `has_optional` carries the same notion in both `Unfolded` and `Flipped`:
/// whether any unresolved `?.` exists in the chain segment seen so far —
/// below the current level while still searching, above the fold point
/// after firing. It gates outer fold attempts and the wrapper unwrap.
enum ChainFold<'a> {
    Unfolded {
        has_optional: bool,
    },
    Flipped {
        has_optional: bool,
    },
    /// `base_has_side_effects` is computed here so the caller doesn't walk
    /// the base subtree a second time.
    Collapse {
        base: Expression<'a>,
        base_has_side_effects: bool,
    },
}

/// Try folding the *deepest* (= leftmost in source order) optional in a
/// chain. Recurses inward through `.object` / `.callee` first so the
/// short-circuit point is found before any outer access.
///
/// Nested `ChainExpression`s (e.g. `(a?.b)?.c`) are not descended into —
/// the compressor's separate flattening pass merges them into a single
/// chain on a later iteration, after which this fold fires.
fn try_fold_chain_at_element<'a>(
    elem: &mut ChainElement<'a>,
    ctx: &TraverseCtx<'a>,
) -> ChainFold<'a> {
    match elem {
        ChainElement::CallExpression(c) => try_fold_call_expression(c, ctx),
        match_member_expression!(ChainElement) => {
            try_fold_member_expression(elem.to_member_expression_mut(), ctx)
        }
        ChainElement::TSNonNullExpression(t) => try_fold_chain_at_expr(&mut t.expression, ctx),
    }
}

fn try_fold_chain_at_expr<'a>(expr: &mut Expression<'a>, ctx: &TraverseCtx<'a>) -> ChainFold<'a> {
    match expr.get_inner_expression_mut() {
        Expression::CallExpression(c) => try_fold_call_expression(c, ctx),
        match_member_expression!(Expression) => {
            try_fold_member_expression(expr.to_member_expression_mut(), ctx)
        }
        _ => ChainFold::Unfolded { has_optional: false },
    }
}

fn try_fold_call_expression<'a>(
    call: &mut CallExpression<'a>,
    ctx: &TraverseCtx<'a>,
) -> ChainFold<'a> {
    match try_fold_chain_at_expr(&mut call.callee, ctx) {
        ChainFold::Flipped { has_optional } => {
            ChainFold::Flipped { has_optional: has_optional || call.optional }
        }
        collapse @ ChainFold::Collapse { .. } => collapse,
        ChainFold::Unfolded { has_optional } => {
            try_fold_at_optional(&mut call.optional, &mut call.callee, has_optional, ctx)
                .unwrap_or(ChainFold::Unfolded { has_optional: has_optional || call.optional })
        }
    }
}

fn try_fold_member_expression<'a>(
    member: &mut MemberExpression<'a>,
    ctx: &TraverseCtx<'a>,
) -> ChainFold<'a> {
    let (optional_mut, object) = member_expression_optional_and_object_mut(member);
    let optional = *optional_mut;
    match try_fold_chain_at_expr(object, ctx) {
        ChainFold::Flipped { has_optional } => {
            ChainFold::Flipped { has_optional: has_optional || optional }
        }
        collapse @ ChainFold::Collapse { .. } => collapse,
        ChainFold::Unfolded { has_optional } => {
            try_fold_at_optional(optional_mut, object, has_optional, ctx)
                .unwrap_or(ChainFold::Unfolded { has_optional: has_optional || optional })
        }
    }
}

fn member_expression_optional_and_object_mut<'a, 'b>(
    member: &'b mut MemberExpression<'a>,
) -> (&'b mut bool, &'b mut Expression<'a>) {
    match member {
        MemberExpression::StaticMemberExpression(m) => {
            let m = &mut **m;
            (&mut m.optional, &mut m.object)
        }
        MemberExpression::ComputedMemberExpression(m) => {
            let m = &mut **m;
            (&mut m.optional, &mut m.object)
        }
        MemberExpression::PrivateFieldExpression(m) => {
            let m = &mut **m;
            (&mut m.optional, &mut m.object)
        }
    }
}

fn try_fold_at_optional<'a>(
    optional: &mut bool,
    base: &mut Expression<'a>,
    has_optional: bool,
    ctx: &TraverseCtx<'a>,
) -> Option<ChainFold<'a>> {
    if !*optional || has_optional {
        return None;
    }
    match base.value_type(ctx) {
        ValueType::Null | ValueType::Undefined => {
            let base_has_side_effects = base.may_have_side_effects(ctx);
            let taken = base.take_in(ctx.ast);
            Some(ChainFold::Collapse { base: taken, base_has_side_effects })
        }
        ValueType::Undetermined => None,
        _ => {
            *optional = false;
            Some(ChainFold::Flipped { has_optional: false })
        }
    }
}
