use crate::TraverseCtx;
use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::{ast::*, builder::NONE};
use oxc_compat::ESFeature;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::{ContentEq, GetSpan};

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_conditional(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // Wrap the fresh conditional in an `Expression` slot so that, if the
        // fold returns a replacement, `ctx.replace_expression` can walk the
        // mutated transient conditional and mark its leaked refs dead. Without
        // the slot wrapping, refs left in untouched slots of the discarded
        // transient `ConditionalExpression` (e.g. the leftover `b` in
        // `b == null ? c : b` -> `b ?? c`) would never reach `PassDirty`.
        let mut as_expr =
            Expression::new_conditional_expression(span, test, consequent, alternate, ctx);
        let Expression::ConditionalExpression(cond_box) = &mut as_expr else { unreachable!() };
        let folded = Self::minimize_conditional_expression(cond_box, ctx);
        if let Some(new_expr) = folded {
            ctx.replace_expression(&mut as_expr, new_expr);
        }
        as_expr
    }

    /// `MangleIfExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2745>
    pub fn minimize_conditional_expression(
        expr: &mut ConditionalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match &mut expr.test {
            // "(a, b) ? c : d" => "a, b ? c : d"
            Expression::SequenceExpression(sequence_expr)
                if sequence_expr.expressions.len() > 1 =>
            {
                let span = expr.span();
                let mut sequence = expr.test.take_in(ctx);
                let Expression::SequenceExpression(sequence_expr) = &mut sequence else {
                    unreachable!()
                };
                let expr = Self::minimize_conditional(
                    span,
                    sequence_expr.expressions.pop().unwrap(),
                    expr.consequent.take_in(ctx),
                    expr.alternate.take_in(ctx),
                    ctx,
                );
                sequence_expr.expressions.push(expr);
                return Some(sequence);
            }
            // "!a ? b : c" => "a ? c : b"
            Expression::UnaryExpression(test_expr) if test_expr.operator.is_not() => {
                let test = test_expr.argument.take_in(ctx);
                let consequent = expr.alternate.take_in(ctx);
                let alternate = expr.consequent.take_in(ctx);
                return Some(Self::minimize_conditional(
                    expr.span, test, consequent, alternate, ctx,
                ));
            }
            Expression::Identifier(id) => {
                // "a ? a : b" => "a || b"
                if let Expression::Identifier(id2) = &expr.consequent
                    && id.name == id2.name
                {
                    return Some(Self::join_with_left_associative_op(
                        expr.span,
                        LogicalOperator::Or,
                        expr.test.take_in(ctx),
                        expr.alternate.take_in(ctx),
                        ctx,
                    ));
                }
                // "a ? b : a" => "a && b"
                if let Expression::Identifier(id2) = &expr.alternate
                    && id.name == id2.name
                {
                    return Some(Self::join_with_left_associative_op(
                        expr.span,
                        LogicalOperator::And,
                        expr.test.take_in(ctx),
                        expr.consequent.take_in(ctx),
                        ctx,
                    ));
                }
            }
            // `x != y ? b : c` -> `x == y ? c : b`
            Expression::BinaryExpression(test_expr) => {
                if matches!(
                    test_expr.operator,
                    BinaryOperator::Inequality | BinaryOperator::StrictInequality
                ) {
                    test_expr.operator = test_expr.operator.equality_inverse_operator().unwrap();
                    let test = expr.test.take_in(ctx);
                    let consequent = expr.consequent.take_in(ctx);
                    let alternate = expr.alternate.take_in(ctx);
                    return Some(Self::minimize_conditional(
                        expr.span, test, alternate, consequent, ctx,
                    ));
                }
            }
            _ => {}
        }

        // "a ? b ? c : d : d" => "a && b ? c : d"
        if let Expression::ConditionalExpression(consequent) = &mut expr.consequent
            && ctx.expr_eq(&consequent.alternate, &expr.alternate)
        {
            return Some(Expression::new_conditional_expression(
                expr.span,
                Self::join_with_left_associative_op(
                    expr.test.span(),
                    LogicalOperator::And,
                    expr.test.take_in(ctx),
                    consequent.test.take_in(ctx),
                    ctx,
                ),
                consequent.consequent.take_in(ctx),
                consequent.alternate.take_in(ctx),
                ctx,
            ));
        }

        // "a ? b : c ? b : d" => "a || c ? b : d"
        if let Expression::ConditionalExpression(alternate) = &mut expr.alternate
            && ctx.expr_eq(&alternate.consequent, &expr.consequent)
        {
            return Some(Expression::new_conditional_expression(
                expr.span,
                Self::join_with_left_associative_op(
                    expr.test.span(),
                    LogicalOperator::Or,
                    expr.test.take_in(ctx),
                    alternate.test.take_in(ctx),
                    ctx,
                ),
                expr.consequent.take_in(ctx),
                alternate.alternate.take_in(ctx),
                ctx,
            ));
        }

        // "a ? c : (b, c)" => "(a || b), c"
        if let Expression::SequenceExpression(alternate) = &mut expr.alternate
            && alternate.expressions.len() == 2
            && ctx.expr_eq(&alternate.expressions[1], &expr.consequent)
        {
            return Some(Expression::new_sequence_expression(
                expr.span,
                ArenaVec::from_array_in(
                    [
                        Self::join_with_left_associative_op(
                            expr.test.span(),
                            LogicalOperator::Or,
                            expr.test.take_in(ctx),
                            alternate.expressions[0].take_in(ctx),
                            ctx,
                        ),
                        expr.consequent.take_in(ctx),
                    ],
                    ctx,
                ),
                ctx,
            ));
        }

        // "a ? (b, c) : c" => "(a && b), c"
        if let Expression::SequenceExpression(consequent) = &mut expr.consequent
            && consequent.expressions.len() == 2
            && ctx.expr_eq(&consequent.expressions[1], &expr.alternate)
        {
            return Some(Expression::new_sequence_expression(
                expr.span,
                ArenaVec::from_array_in(
                    [
                        Self::join_with_left_associative_op(
                            expr.test.span(),
                            LogicalOperator::And,
                            expr.test.take_in(ctx),
                            consequent.expressions[0].take_in(ctx),
                            ctx,
                        ),
                        expr.alternate.take_in(ctx),
                    ],
                    ctx,
                ),
                ctx,
            ));
        }

        // "a ? b || c : c" => "(a && b) || c"
        if let Expression::LogicalExpression(logical_expr) = &mut expr.consequent
            && logical_expr.operator.is_or()
            && ctx.expr_eq(&logical_expr.right, &expr.alternate)
        {
            return Some(Expression::new_logical_expression(
                expr.span,
                Self::join_with_left_associative_op(
                    expr.test.span(),
                    LogicalOperator::And,
                    expr.test.take_in(ctx),
                    logical_expr.left.take_in(ctx),
                    ctx,
                ),
                LogicalOperator::Or,
                expr.alternate.take_in(ctx),
                ctx,
            ));
        }

        // "a ? c : b && c" => "(a || b) && c"
        if let Expression::LogicalExpression(logical_expr) = &mut expr.alternate
            && logical_expr.operator == LogicalOperator::And
            && ctx.expr_eq(&logical_expr.right, &expr.consequent)
        {
            return Some(Expression::new_logical_expression(
                expr.span,
                Self::join_with_left_associative_op(
                    expr.test.span(),
                    LogicalOperator::Or,
                    expr.test.take_in(ctx),
                    logical_expr.left.take_in(ctx),
                    ctx,
                ),
                LogicalOperator::And,
                expr.consequent.take_in(ctx),
                ctx,
            ));
        }

        // `a ? b(c, d) : b(e, d)` -> `b(a ? c : e, d)`
        if let (Expression::CallExpression(consequent), Expression::CallExpression(alternate)) =
            (&mut expr.consequent, &mut expr.alternate)
        {
            // `a ? b() : b()` is handled later
            if !consequent.arguments.is_empty() &&
                consequent.arguments.len() == alternate.arguments.len()
                // we can improve compression by allowing side effects on one side if the other side is
                // an identifier that is not modified after it is declared.
                // but for now, we only perform compression if neither side has side effects.
                && !expr.test.may_have_side_effects(ctx)
                && !consequent.callee.may_have_side_effects(ctx)
                && ctx.expr_eq(&consequent.callee, &alternate.callee)
                && consequent
                    .arguments
                    .iter()
                    .zip(&alternate.arguments)
                    .skip(1)
                    .all(|(a, b)| a.content_eq(b))
            {
                // `a ? b(...c) : b(...e)` -> `b(...a ? c : e)``
                if matches!(consequent.arguments[0], Argument::SpreadElement(_))
                    && matches!(alternate.arguments[0], Argument::SpreadElement(_))
                {
                    let callee = consequent.callee.take_in(ctx);
                    let consequent_first_arg = {
                        let Argument::SpreadElement(el) = &mut consequent.arguments[0] else {
                            unreachable!()
                        };
                        el.argument.take_in(ctx)
                    };
                    let alternate_first_arg = {
                        let Argument::SpreadElement(el) = &mut alternate.arguments[0] else {
                            unreachable!()
                        };
                        el.argument.take_in(ctx)
                    };
                    let mut args =
                        std::mem::replace(&mut consequent.arguments, ArenaVec::new_in(ctx));
                    args[0] = Argument::new_spread_element(
                        expr.span,
                        Expression::new_conditional_expression(
                            expr.test.span(),
                            expr.test.take_in(ctx),
                            consequent_first_arg,
                            alternate_first_arg,
                            ctx,
                        ),
                        ctx,
                    );
                    return Some(Expression::new_call_expression(
                        expr.span, callee, NONE, args, false, ctx,
                    ));
                }
                // `a ? b(c) : b(e)` -> `b(a ? c : e)`
                if !matches!(consequent.arguments[0], Argument::SpreadElement(_))
                    && !matches!(alternate.arguments[0], Argument::SpreadElement(_))
                {
                    let callee = consequent.callee.take_in(ctx);

                    let consequent_first_arg =
                        consequent.arguments[0].to_expression_mut().take_in(ctx);
                    let alternate_first_arg =
                        alternate.arguments[0].to_expression_mut().take_in(ctx);
                    let mut args =
                        std::mem::replace(&mut consequent.arguments, ArenaVec::new_in(ctx));
                    let cond_expr = Self::minimize_conditional(
                        expr.test.span(),
                        expr.test.take_in(ctx),
                        consequent_first_arg,
                        alternate_first_arg,
                        ctx,
                    );
                    args[0] = Argument::from(cond_expr);
                    return Some(Expression::new_call_expression(
                        expr.span, callee, NONE, args, false, ctx,
                    ));
                }
            }
        }

        // Not part of esbuild
        if let Some(e) = Self::try_merge_conditional_expression_inside(expr, ctx) {
            return Some(e);
        }

        // Try using the "??" or "?." operators
        if (ctx.supports_feature(ESFeature::ES2020NullishCoalescingOperator)
            || ctx.supports_feature(ESFeature::ES2020OptionalChaining))
            && let Expression::BinaryExpression(test_binary) = &mut expr.test
            && let Some(is_negate) = match test_binary.operator {
                BinaryOperator::Inequality => Some(true),
                BinaryOperator::Equality => Some(false),
                _ => None,
            }
        {
            // a == null / a != null / (a = foo) == null / (a = foo) != null
            let value_expr_with_id_name = if test_binary.left.is_null() {
                if let Some(id) = Self::extract_id_or_assign_to_id(&test_binary.right)
                    .filter(|id| !ctx.is_global_reference(id))
                {
                    Some((id.name, &mut test_binary.right))
                } else {
                    None
                }
            } else if test_binary.right.is_null() {
                if let Some(id) = Self::extract_id_or_assign_to_id(&test_binary.left)
                    .filter(|id| !ctx.is_global_reference(id))
                {
                    Some((id.name, &mut test_binary.left))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((target_id_name, value_expr)) = value_expr_with_id_name {
                if ctx.supports_feature(ESFeature::ES2020NullishCoalescingOperator) {
                    // `a == null ? b : a` -> `a ?? b`
                    // `a != null ? a : b` -> `a ?? b`
                    // `(a = foo) == null ? b : a` -> `(a = foo) ?? b`
                    // `(a = foo) != null ? a : b` -> `(a = foo) ?? b`
                    let maybe_same_id_expr =
                        if is_negate { &mut expr.consequent } else { &mut expr.alternate };
                    if maybe_same_id_expr.is_specific_id(&target_id_name) {
                        return Some(Expression::new_logical_expression(
                            expr.span,
                            value_expr.take_in(ctx),
                            LogicalOperator::Coalesce,
                            if is_negate {
                                expr.alternate.take_in(ctx)
                            } else {
                                expr.consequent.take_in(ctx)
                            },
                            ctx,
                        ));
                    }
                }
                if ctx.supports_feature(ESFeature::ES2020OptionalChaining) {
                    // "a == null ? undefined : a.b.c[d](e)" => "a?.b.c[d](e)"
                    // "a != null ? a.b.c[d](e) : undefined" => "a?.b.c[d](e)"
                    // "(a = foo) == null ? undefined : a.b.c[d](e)" => "(a = foo)?.b.c[d](e)"
                    // "(a = foo) != null ? a.b.c[d](e) : undefined" => "(a = foo)?.b.c[d](e)"
                    let maybe_undefined_expr =
                        if is_negate { &expr.alternate } else { &expr.consequent };
                    if ctx.is_expression_undefined(maybe_undefined_expr) {
                        let expr_to_inject_optional_chaining =
                            if is_negate { &mut expr.consequent } else { &mut expr.alternate };
                        if Self::inject_optional_chaining_if_matched(
                            &target_id_name,
                            value_expr,
                            expr_to_inject_optional_chaining,
                            ctx,
                        ) {
                            return Some(expr_to_inject_optional_chaining.take_in(ctx));
                        }
                    }
                }
            }
        }

        let consequent_value = expr.consequent.evaluate_value(ctx);
        let alternate_value = expr.alternate.evaluate_value(ctx);

        // "a ? true : false" => "!!a"
        // "a ? false : true" => "!a"
        match (
            consequent_value
                .as_ref()
                .and_then(|v| match v {
                    ConstantValue::Boolean(b) => Some(*b),
                    _ => None,
                })
                .filter(|_| !expr.consequent.may_have_side_effects(ctx)),
            alternate_value
                .as_ref()
                .and_then(|v| match v {
                    ConstantValue::Boolean(b) => Some(*b),
                    _ => None,
                })
                .filter(|_| !expr.alternate.may_have_side_effects(ctx)),
        ) {
            (Some(true), Some(false)) => {
                let test = expr.test.take_in(ctx);
                let test = Self::minimize_not(expr.span, test, ctx);
                let test = Self::minimize_not(expr.span, test, ctx);
                return Some(test);
            }
            (Some(false), Some(true)) => {
                let test = expr.test.take_in(ctx);
                let test = Self::minimize_not(expr.span, test, ctx);
                return Some(test);
            }
            // "c ? false : x" => "!c && x" (exact for any `c`)
            (Some(false), None)
                if Self::can_fold_negated_test(&expr.test)
                    && !Self::logical_operand_needs_parens(
                        &expr.alternate,
                        LogicalOperator::And,
                    ) =>
            {
                let test = expr.test.take_in(ctx);
                let test = Self::minimize_not(expr.span, test, ctx);
                let right = expr.alternate.take_in(ctx);
                return Some(Self::join_with_left_associative_op(
                    expr.span,
                    LogicalOperator::And,
                    test,
                    right,
                    ctx,
                ));
            }
            // "c ? x : true" => "!c || x" (exact for any `c`)
            (None, Some(true))
                if Self::can_fold_negated_test(&expr.test)
                    && !Self::logical_operand_needs_parens(
                        &expr.consequent,
                        LogicalOperator::Or,
                    ) =>
            {
                let test = expr.test.take_in(ctx);
                let test = Self::minimize_not(expr.span, test, ctx);
                let right = expr.consequent.take_in(ctx);
                return Some(Self::join_with_left_associative_op(
                    expr.span,
                    LogicalOperator::Or,
                    test,
                    right,
                    ctx,
                ));
            }
            // "c ? true : x" => "c || x" (only when `c` is boolean-typed; a
            // non-boolean truthy `c` would be returned instead of `true`)
            (Some(true), None)
                if expr.test.value_type(ctx).is_boolean()
                    && !Self::logical_operand_needs_parens(&expr.test, LogicalOperator::Or)
                    && !Self::logical_operand_needs_parens(
                        &expr.alternate,
                        LogicalOperator::Or,
                    ) =>
            {
                let test = expr.test.take_in(ctx);
                let right = expr.alternate.take_in(ctx);
                return Some(Self::join_with_left_associative_op(
                    expr.span,
                    LogicalOperator::Or,
                    test,
                    right,
                    ctx,
                ));
            }
            // "c ? x : false" => "c && x" (only when `c` is boolean-typed; a
            // non-boolean falsy `c` would be returned instead of `false`)
            (None, Some(false))
                if expr.test.value_type(ctx).is_boolean()
                    && !Self::logical_operand_needs_parens(&expr.test, LogicalOperator::And)
                    && !Self::logical_operand_needs_parens(
                        &expr.consequent,
                        LogicalOperator::And,
                    ) =>
            {
                let test = expr.test.take_in(ctx);
                let right = expr.consequent.take_in(ctx);
                return Some(Self::join_with_left_associative_op(
                    expr.span,
                    LogicalOperator::And,
                    test,
                    right,
                    ctx,
                ));
            }
            _ => {}
        }

        // "a ? 1 : 0" => "+a" (if a is boolean) or "+!!a" (if no parens needed)
        // "a ? 0 : 1" => "+!a" (if no parens needed)
        match (
            consequent_value
                .and_then(ConstantValue::into_number)
                .filter(|_| !expr.consequent.may_have_side_effects(ctx)),
            alternate_value
                .and_then(ConstantValue::into_number)
                .filter(|_| !expr.alternate.may_have_side_effects(ctx)),
        ) {
            (Some(1.0), Some(0.0)) => {
                // "a ? 1 : 0"
                let is_boolean = expr.test.value_type(ctx).is_boolean();
                let needs_parens = Self::test_needs_parens(&expr.test);
                if is_boolean {
                    // Known boolean: +a (saves 3 chars: "a?1:0" => "+a")
                    let test = expr.test.take_in(ctx);
                    return Some(Expression::new_unary_expression(expr.span,
                    UnaryOperator::UnaryPlus,
                    test, ctx));
                }
                // Unknown type: +!!a (saves 1 char: "a?1:0" => "+!!a")
                // But skip if parens would be needed (e.g., "a+b?1:0" => "+!!(a+b)" is longer)
                if !needs_parens {
                    let test = expr.test.take_in(ctx);
                    let test = Self::minimize_not(expr.span, test, ctx);
                    let test = Self::minimize_not(expr.span, test, ctx);
                    return Some(Expression::new_unary_expression(expr.span,
                    UnaryOperator::UnaryPlus,
                    test, ctx));
                }
            }
            (Some(0.0), Some(1.0))
                // "a ? 0 : 1" => "+!a"
                // Skip if parens would be needed (e.g., "a+b?0:1" => "+!(a+b)" is same length)
                if !Self::test_needs_parens(&expr.test) => {
                    let test = expr.test.take_in(ctx);
                    let test = Self::minimize_not(expr.span, test, ctx);
                    return Some(Expression::new_unary_expression(expr.span,
                    UnaryOperator::UnaryPlus,
                    test, ctx));
                }
            _ => {}
        }

        if ctx.expr_eq(&expr.alternate, &expr.consequent) {
            // "/* @__PURE__ */ a() ? b : b" => "b"
            if !expr.test.may_have_side_effects(ctx) {
                let result_expr = expr.consequent.take_in(ctx);
                // "(a ? eval : eval)(x)" => "(0, eval)(x)" — the bare branch
                // would form a direct eval call / rebind a member call's `this`.
                if Self::should_keep_indirect_access(&result_expr, ctx) {
                    return Some(Self::preserve_indirect_access(expr.span, result_expr, ctx));
                }
                return Some(result_expr);
            }

            // "a ? b : b" => "a, b"
            let expressions = ArenaVec::from_array_in(
                [expr.test.take_in(ctx), expr.consequent.take_in(ctx)],
                ctx,
            );
            return Some(Expression::new_sequence_expression(expr.span, expressions, ctx));
        }

        None
    }

    /// Merge `consequent` and `alternate` of `ConditionalExpression` inside.
    ///
    /// - `x ? a = 0 : a = 1` -> `a = x ? 0 : 1`
    fn try_merge_conditional_expression_inside(
        expr: &mut ConditionalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let (
            Expression::AssignmentExpression(consequent),
            Expression::AssignmentExpression(alternate),
        ) = (&mut expr.consequent, &mut expr.alternate)
        else {
            return None;
        };
        if !matches!(consequent.left, AssignmentTarget::AssignmentTargetIdentifier(_)) {
            return None;
        }
        // NOTE: if the right hand side is an anonymous function, applying this compression will
        // set the `name` property of that function.
        // Since codes relying on the fact that function's name is undefined should be rare,
        // we do this compression even if `keep_names` is enabled.

        if consequent.operator != AssignmentOperator::Assign
            || consequent.operator != alternate.operator
            || consequent.left.content_ne(&alternate.left)
        {
            return None;
        }
        let cond_expr = Self::minimize_conditional(
            expr.span,
            expr.test.take_in(ctx),
            consequent.right.take_in(ctx),
            alternate.right.take_in(ctx),
            ctx,
        );
        Some(Expression::new_assignment_expression(
            expr.span,
            consequent.operator,
            alternate.left.take_in(ctx),
            cond_expr,
            ctx,
        ))
    }

    /// Modify `expr` if that has `target_expr` as a parent, and returns true if modified.
    ///
    /// For `target_expr` = `a`, `expr` = `a.b`, this function changes `expr` to `a?.b` and returns true.
    pub fn inject_optional_chaining_if_matched(
        target_id_name: &str,
        expr_to_inject: &mut Expression<'a>,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        if Self::inject_optional_chaining_if_matched_inner(
            target_id_name,
            expr_to_inject,
            expr,
            ctx,
        ) {
            if !matches!(expr, Expression::ChainExpression(_)) {
                let new_expr = Expression::new_chain_expression(
                    expr.span(),
                    expr.take_in(ctx).into_chain_element().unwrap(),
                    ctx,
                );
                ctx.replace_expression(expr, new_expr);
            }
            true
        } else {
            false
        }
    }

    /// See [`Self::inject_optional_chaining_if_matched`]
    fn inject_optional_chaining_if_matched_inner(
        target_id_name: &str,
        expr_to_inject: &mut Expression<'a>,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        match expr {
            Expression::StaticMemberExpression(e) => {
                if e.object.is_specific_id(target_id_name) {
                    e.optional = true;
                    let new_object = expr_to_inject.take_in(ctx);
                    ctx.replace_expression(&mut e.object, new_object);
                    return true;
                }
                if Self::inject_optional_chaining_if_matched_inner(
                    target_id_name,
                    expr_to_inject,
                    &mut e.object,
                    ctx,
                ) {
                    return true;
                }
            }
            Expression::ComputedMemberExpression(e) => {
                if e.object.is_specific_id(target_id_name) {
                    e.optional = true;
                    let new_object = expr_to_inject.take_in(ctx);
                    ctx.replace_expression(&mut e.object, new_object);
                    return true;
                }
                if Self::inject_optional_chaining_if_matched_inner(
                    target_id_name,
                    expr_to_inject,
                    &mut e.object,
                    ctx,
                ) {
                    return true;
                }
            }
            Expression::CallExpression(e) => {
                if e.callee.is_specific_id(target_id_name) {
                    e.optional = true;
                    let new_callee = expr_to_inject.take_in(ctx);
                    ctx.replace_expression(&mut e.callee, new_callee);
                    return true;
                }
                if Self::inject_optional_chaining_if_matched_inner(
                    target_id_name,
                    expr_to_inject,
                    &mut e.callee,
                    ctx,
                ) {
                    return true;
                }
            }
            Expression::ChainExpression(e) => match &mut e.expression {
                ChainElement::StaticMemberExpression(e) => {
                    if e.object.is_specific_id(target_id_name) {
                        e.optional = true;
                        let new_object = expr_to_inject.take_in(ctx);
                        ctx.replace_expression(&mut e.object, new_object);
                        return true;
                    }
                    if Self::inject_optional_chaining_if_matched_inner(
                        target_id_name,
                        expr_to_inject,
                        &mut e.object,
                        ctx,
                    ) {
                        return true;
                    }
                }
                ChainElement::ComputedMemberExpression(e) => {
                    if e.object.is_specific_id(target_id_name) {
                        e.optional = true;
                        let new_object = expr_to_inject.take_in(ctx);
                        ctx.replace_expression(&mut e.object, new_object);
                        return true;
                    }
                    if Self::inject_optional_chaining_if_matched_inner(
                        target_id_name,
                        expr_to_inject,
                        &mut e.object,
                        ctx,
                    ) {
                        return true;
                    }
                }
                ChainElement::CallExpression(e) => {
                    if e.callee.is_specific_id(target_id_name) {
                        e.optional = true;
                        let new_callee = expr_to_inject.take_in(ctx);
                        ctx.replace_expression(&mut e.callee, new_callee);
                        return true;
                    }
                    if Self::inject_optional_chaining_if_matched_inner(
                        target_id_name,
                        expr_to_inject,
                        &mut e.callee,
                        ctx,
                    ) {
                        return true;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        false
    }

    /// Returns `true` when `minimize_not(test)` negates `test` without wrapping
    /// it in a `!(...)`, so folding a conditional into `!test && x` / `!test || x`
    /// does not grow the output. Equality comparisons invert their operator in
    /// place (`a === b` -> `a !== b`); anything that is not otherwise
    /// parenthesized as a unary operand negates to a bare `!test`.
    fn can_fold_negated_test(test: &Expression<'_>) -> bool {
        if let Expression::BinaryExpression(e) = test {
            return e.operator.is_equality();
        }
        !Self::test_needs_parens(test)
    }

    /// Returns `true` if `expr` would need parentheses when printed as an operand
    /// of the logical operator `op` (`&&` or `||`). Used as a size guard before
    /// folding a conditional into a logical expression.
    fn logical_operand_needs_parens(expr: &Expression<'_>, op: LogicalOperator) -> bool {
        match expr {
            Expression::SequenceExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::YieldExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ConditionalExpression(_) => true,
            // `??` cannot be mixed with `&&`/`||` without parens, and `||` needs
            // parens as an operand of `&&`. `&&` under `||`, and same-operator
            // nesting (flattened by `join_with_left_associative_op`), do not.
            Expression::LogicalExpression(e) => {
                matches!(e.operator, LogicalOperator::Coalesce)
                    || matches!((e.operator, op), (LogicalOperator::Or, LogicalOperator::And))
            }
            _ => false,
        }
    }

    /// Returns `true` if the expression would need parentheses when used as a unary operand.
    /// Binary and conditional expressions need wrapping; identifiers, calls, unary, etc. do not.
    fn test_needs_parens(expr: &Expression<'_>) -> bool {
        matches!(
            expr,
            Expression::BinaryExpression(_)
                | Expression::LogicalExpression(_)
                | Expression::ConditionalExpression(_)
                | Expression::AssignmentExpression(_)
                | Expression::SequenceExpression(_)
                | Expression::YieldExpression(_)
        )
    }
}
