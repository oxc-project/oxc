use std::iter;

use crate::{CompressOptionsUnused, TraverseCtx, generated::ancestor::Ancestor};
use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_compat::ESFeature;
use oxc_ecmascript::{
    ToPrimitive,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext},
};
use oxc_span::GetSpan;
use oxc_syntax::symbol::SymbolId;

use super::PeepholeOptimizations;
use super::fold_constants::is_cjs_module_exports_hint;

impl<'a> PeepholeOptimizations {
    /// `SimplifyUnusedExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L534>
    pub fn remove_unused_expression(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        match e {
            Expression::ArrayExpression(_) => Self::remove_unused_array_expr(e, ctx),
            Expression::AssignmentExpression(_) => Self::remove_unused_assignment_expr(e, ctx),
            Expression::BinaryExpression(_) => Self::remove_unused_binary_expr(e, ctx),
            Expression::CallExpression(_) => Self::remove_unused_call_expr(e, ctx),
            Expression::ClassExpression(_) => Self::remove_unused_class_expr(e, ctx),
            Expression::ConditionalExpression(_) => Self::remove_unused_conditional_expr(e, ctx),
            Expression::LogicalExpression(_) => Self::remove_unused_logical_expr(e, ctx),
            Expression::NewExpression(_) => Self::remove_unused_new_expr(e, ctx),
            Expression::ObjectExpression(_) => Self::remove_unused_object_expr(e, ctx),
            Expression::SequenceExpression(_) => Self::remove_unused_sequence_expr(e, ctx),
            Expression::TemplateLiteral(_) => Self::remove_unused_template_literal(e, ctx),
            Expression::UnaryExpression(_) => Self::remove_unused_unary_expr(e, ctx),
            // In a derived class constructor, accessing `this` before `super()` throws
            // a `ReferenceError`, so we must keep it. In all other positions (including
            // non-derived constructors) `this` is always initialized and can be dropped.
            Expression::ThisExpression(_) => !Self::this_is_inside_derived_constructor(ctx),
            _ => !e.may_have_side_effects(ctx),
        }
    }

    /// Whether the nearest non-arrow, non-block function scope is a constructor
    /// of a class that extends another class (derived class).
    /// Only derived constructors have the TDZ for `this` before `super()`.
    pub(crate) fn this_is_inside_derived_constructor(ctx: &TraverseCtx<'a>) -> bool {
        for scope_id in ctx.ancestor_scopes() {
            let flags = ctx.scoping().scope_flags(scope_id);
            if flags.is_block() || flags.is_arrow() {
                continue;
            }
            if !flags.is_constructor() {
                return false;
            }
            // Found a constructor — check if the class has `extends`.
            for ancestor in ctx.ancestors() {
                if let Ancestor::ClassBody(class) = ancestor {
                    return class.super_class().is_some();
                }
            }
            return false;
        }
        false
    }

    fn remove_unused_unary_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::UnaryExpression(unary_expr) = e else { return false };
        match unary_expr.operator {
            UnaryOperator::Void | UnaryOperator::LogicalNot => {
                let new_expr = unary_expr.argument.take_in(ctx);
                ctx.replace_expression(e, new_expr);
                Self::remove_unused_expression(e, ctx)
            }
            UnaryOperator::Typeof => {
                if unary_expr.argument.is_identifier_reference() {
                    true
                } else {
                    let new_expr = unary_expr.argument.take_in(ctx);
                    ctx.replace_expression(e, new_expr);
                    Self::remove_unused_expression(e, ctx)
                }
            }
            _ => !e.may_have_side_effects(ctx),
        }
    }

    fn remove_unused_sequence_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::SequenceExpression(sequence_expr) = e else { return false };
        sequence_expr.expressions.retain_mut(|e| {
            if Self::remove_unused_expression(e, ctx) {
                ctx.drop_expression(e);
                false
            } else {
                true
            }
        });
        sequence_expr.expressions.is_empty()
    }

    fn remove_unused_logical_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        // Preserve `0 && (module.exports = { ... })` — see
        // `is_cjs_module_exports_hint` in `fold_constants.rs`. esbuild emits
        // this shape on Node platform as a `cjs-module-lexer` hint
        // (https://github.com/evanw/esbuild/blob/v0.28.0/internal/linker/linker.go#L5127-L5138).
        // Without this guard, callers that disable
        // `treeshake.property_write_side_effects` (e.g. rolldown / vite)
        // reach the `!may_have_side_effects` branch below and silently drop
        // the hint.
        if let Expression::LogicalExpression(logical_expr) = e
            && is_cjs_module_exports_hint(&logical_expr.right)
        {
            return false;
        }
        if !e.may_have_side_effects(ctx) {
            return true;
        }
        let Expression::LogicalExpression(logical_expr) = e else { return false };
        if !logical_expr.operator.is_coalesce() {
            Self::minimize_expression_in_boolean_context(&mut logical_expr.left, ctx);
        }
        if Self::remove_unused_expression(&mut logical_expr.right, ctx) {
            Self::remove_unused_expression(&mut logical_expr.left, ctx);
            let new_expr = logical_expr.left.take_in(ctx);
            ctx.replace_expression(e, new_expr);
            return false;
        }

        // try optional chaining and nullish coalescing
        if ctx.supports_feature(ESFeature::ES2020OptionalChaining)
            || ctx.supports_feature(ESFeature::ES2020NullishCoalescingOperator)
        {
            let LogicalExpression {
                span: logical_span,
                left: logical_left,
                right: logical_right,
                operator: logical_op,
                ..
            } = logical_expr.as_mut();
            if let Expression::BinaryExpression(binary_expr) = logical_left {
                match (logical_op, binary_expr.operator) {
                    // "a != null && a.b()" => "a?.b()"
                    // "a == null || a.b()" => "a?.b()"
                    (LogicalOperator::And, BinaryOperator::Inequality)
                    | (LogicalOperator::Or, BinaryOperator::Equality)
                        if ctx.supports_feature(ESFeature::ES2020OptionalChaining) =>
                    {
                        let name_and_id = if let Expression::Identifier(id) = &binary_expr.left {
                            (!ctx.is_global_reference(id) && binary_expr.right.is_null())
                                .then_some((id.name, &mut binary_expr.left))
                        } else if let Expression::Identifier(id) = &binary_expr.right {
                            (!ctx.is_global_reference(id) && binary_expr.left.is_null())
                                .then_some((id.name, &mut binary_expr.right))
                        } else {
                            None
                        };
                        if let Some((name, id)) = name_and_id
                            && Self::inject_optional_chaining_if_matched(
                                &name,
                                id,
                                logical_right,
                                ctx,
                            )
                        {
                            let new_expr = logical_right.take_in(ctx);
                            ctx.replace_expression(e, new_expr);
                            return false;
                        }
                    }
                    // "a == null && b" => "a ?? b"
                    // "a != null || b" => "a ?? b"
                    // "a == null && (a = b)" => "a ??= b"
                    // "a != null || (a = b)" => "a ??= b"
                    (LogicalOperator::And, BinaryOperator::Equality)
                    | (LogicalOperator::Or, BinaryOperator::Inequality)
                        if ctx.supports_feature(ESFeature::ES2020NullishCoalescingOperator) =>
                    {
                        let new_left_hand_expr = if binary_expr.right.is_null() {
                            Some(&mut binary_expr.left)
                        } else if binary_expr.left.is_null() {
                            Some(&mut binary_expr.right)
                        } else {
                            None
                        };
                        if let Some(new_left_hand_expr) = new_left_hand_expr {
                            if ctx.supports_feature(ESFeature::ES2021LogicalAssignmentOperators)
                                    && let Expression::AssignmentExpression(assignment_expr) =
                                        logical_right
                                    && assignment_expr.operator == AssignmentOperator::Assign
                                    && Self::has_no_side_effect_for_evaluation_same_target(
                                        &assignment_expr.left,
                                        new_left_hand_expr,
                                        ctx,
                                    )
                                    // Don't transform `x.y != null || (x = {}, x.y = 3)` to `x.y ??= (x = {}, 3)` because
                                    // `??=` evaluates `x.y` (capturing `x`) before the RHS reassigns `x`.
                                    // https://github.com/oxc-project/oxc/pull/16802#discussion_r2619369597
                                    && !Self::member_object_may_be_mutated(&assignment_expr.left, ctx)
                            {
                                assignment_expr.span = *logical_span;
                                assignment_expr.operator = AssignmentOperator::LogicalNullish;
                                // `??=` reads the LHS to check for nullish, so update reference flags.
                                Self::mark_assignment_target_as_read(&assignment_expr.left, ctx);
                                let new_expr = logical_right.take_in(ctx);
                                ctx.replace_expression(e, new_expr);
                                return false;
                            }

                            let new_expr = Expression::new_logical_expression(
                                *logical_span,
                                new_left_hand_expr.take_in(ctx),
                                LogicalOperator::Coalesce,
                                logical_right.take_in(ctx),
                                ctx,
                            );
                            ctx.replace_expression(e, new_expr);
                            return false;
                        }
                    }
                    _ => {}
                }
            }
        }

        false
    }

    // `([1,2,3, foo()])` -> `foo()`
    fn remove_unused_array_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::ArrayExpression(array_expr) = e else {
            return false;
        };
        if array_expr.elements.is_empty() {
            return true;
        }

        let old_len = array_expr.elements.len();
        array_expr.elements.retain_mut(|el| match el {
            // `Elision` carries no subtree, so it can be dropped through the
            // outer `notice_change` accounting below.
            ArrayExpressionElement::Elision(_) => false,
            ArrayExpressionElement::SpreadElement(_) => {
                // Use the `ArrayExpressionElement` `may_have_side_effects`
                // impl (NOT `spread.argument.may_have_side_effects`) so that
                // the iterator-protocol invocation of `[...ident]` is
                // counted as a side effect and the spread is kept.
                if el.may_have_side_effects(ctx) {
                    return true;
                }
                // The spread is being elided — walk its argument so any
                // identifier refs inside are marked dead in `PassDirty`
                // and don't leak across passes.
                let ArrayExpressionElement::SpreadElement(spread) = el else { unreachable!() };
                ctx.drop_expression(&spread.argument);
                false
            }
            match_expression!(ArrayExpressionElement) => {
                let el_expr = el.to_expression_mut();
                if Self::remove_unused_expression(el_expr, ctx) {
                    ctx.drop_expression(el_expr);
                    false
                } else {
                    true
                }
            }
        });
        if array_expr.elements.len() != old_len {
            ctx.notice_change();
        }

        if array_expr.elements.is_empty() {
            return true;
        }

        // try removing the brackets "[]", if the array does not contain spread elements
        // `a(), b()` is shorter than `[a(), b()]`,
        // but `[...a], [...b]` is not shorter than `[...a, ...b]`
        let keep_as_array = array_expr
            .elements
            .iter()
            .any(|el| matches!(el, ArrayExpressionElement::SpreadElement(_)));
        if keep_as_array {
            return false;
        }

        let mut expressions = ArenaVec::from_iter_in(
            array_expr.elements.drain(..).map(ArrayExpressionElement::into_expression),
            ctx,
        );
        if expressions.is_empty() {
            return true;
        } else if expressions.len() == 1 {
            let new_expr = expressions.pop().unwrap();
            ctx.replace_expression(e, new_expr);
            return false;
        }

        let span = array_expr.span;
        let new_expr = Expression::new_sequence_expression(span, expressions, ctx);
        ctx.replace_expression(e, new_expr);
        false
    }

    fn remove_unused_new_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::NewExpression(new_expr) = e else { return false };
        if (new_expr.pure && ctx.annotations()) || ctx.manual_pure_functions(&new_expr.callee) {
            let mut exprs =
                Self::fold_arguments_into_needed_expressions(&mut new_expr.arguments, ctx);
            if exprs.is_empty() {
                return true;
            } else if exprs.len() == 1 {
                let folded = exprs.pop().unwrap();
                ctx.replace_expression(e, folded);
                return false;
            }
            let folded = Expression::new_sequence_expression(new_expr.span, exprs, ctx);
            ctx.replace_expression(e, folded);
            return false;
        }
        false
    }

    // "`${1}2${foo()}3`" -> "`${foo()}`"
    fn remove_unused_template_literal(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::TemplateLiteral(temp_lit) = e else { return false };
        if temp_lit.expressions.is_empty() {
            return true;
        }
        if temp_lit.expressions.iter().all(|e| e.to_primitive(ctx).is_symbol() != Some(false))
            && temp_lit.quasis.iter().all(|q| q.value.raw.is_empty())
        {
            return false;
        }

        let mut transformed_elements = ArenaVec::new_in(ctx);
        let mut pending_to_string_required_exprs = ArenaVec::new_in(ctx);

        for mut e in temp_lit.expressions.drain(..) {
            if e.to_primitive(ctx).is_symbol() != Some(false) {
                pending_to_string_required_exprs.push(e);
            } else if Self::remove_unused_expression(&mut e, ctx) {
                // The element collapsed to nothing and is dropped right here
                // by the `drain` — walk it so refs inside reach `PassDirty`
                // instead of leaking.
                ctx.drop_expression(&e);
            } else {
                if !pending_to_string_required_exprs.is_empty() {
                    // flush pending to string required expressions
                    let expressions =
                        ArenaVec::from_iter_in(pending_to_string_required_exprs.drain(..), ctx);
                    let mut quasis = ArenaVec::from_iter_in(
                        iter::repeat_with(|| {
                            TemplateElement::new(
                                e.span(),
                                TemplateElementValue { raw: "".into(), cooked: Some("".into()) },
                                false,
                                ctx,
                            )
                        })
                        .take(expressions.len() + 1),
                        ctx,
                    );
                    quasis
                        .last_mut()
                        .expect("template literal must have at least one quasi")
                        .tail = true;
                    transformed_elements.push(Expression::new_template_literal(
                        e.span(),
                        quasis,
                        expressions,
                        ctx,
                    ));
                }
                transformed_elements.push(e);
            }
        }

        if !pending_to_string_required_exprs.is_empty() {
            let expressions =
                ArenaVec::from_iter_in(pending_to_string_required_exprs.drain(..), ctx);
            let mut quasis = ArenaVec::from_iter_in(
                iter::repeat_with(|| {
                    TemplateElement::new(
                        temp_lit.span,
                        TemplateElementValue { raw: "".into(), cooked: Some("".into()) },
                        false,
                        ctx,
                    )
                })
                .take(expressions.len() + 1),
                ctx,
            );
            quasis.last_mut().expect("template literal must have at least one quasi").tail = true;
            transformed_elements.push(Expression::new_template_literal(
                temp_lit.span,
                quasis,
                expressions,
                ctx,
            ));
        }

        if transformed_elements.is_empty() {
            return true;
        } else if transformed_elements.len() == 1 {
            let new_expr = transformed_elements.pop().unwrap();
            ctx.replace_expression(e, new_expr);
            return false;
        }

        let new_expr =
            Expression::new_sequence_expression(temp_lit.span, transformed_elements, ctx);
        ctx.replace_expression(e, new_expr);
        false
    }

    // `({ 1: 1, [foo()]: bar() })` -> `foo(), bar()`
    fn remove_unused_object_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::ObjectExpression(object_expr) = e else {
            return false;
        };
        if object_expr.properties.is_empty() {
            return true;
        }
        if object_expr.properties.iter().all(ObjectPropertyKind::is_spread) {
            // All-spread objects like `({...x})` can only be removed if
            // the spread arguments themselves have no side effects.
            return !object_expr
                .properties
                .iter()
                .any(|property| property.may_have_side_effects(ctx));
        }

        let mut transformed_elements = ArenaVec::new_in(ctx);
        let mut pending_spread_elements = ArenaVec::new_in(ctx);

        for prop in object_expr.properties.drain(..) {
            match prop {
                ObjectPropertyKind::SpreadProperty(_) => {
                    pending_spread_elements.push(prop);
                }
                ObjectPropertyKind::ObjectProperty(prop) => {
                    if !pending_spread_elements.is_empty() {
                        // flush pending spread elements
                        transformed_elements.push(Expression::new_object_expression(
                            prop.span(),
                            pending_spread_elements,
                            ctx,
                        ));
                        pending_spread_elements = ArenaVec::new_in(ctx);
                    }

                    let ObjectProperty { key, mut value, .. } = prop.unbox();
                    // ToPropertyKey(key) throws an error when ToPrimitive(key) throws an Error
                    // But we can ignore that by using the assumption.
                    match key {
                        PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {}
                        match_expression!(PropertyKey) => {
                            let mut prop_key = key.into_expression();
                            if Self::remove_unused_expression(&mut prop_key, ctx) {
                                // Mark refs in the dropped key as dead so the per-pass
                                // scoping refresh removes them; otherwise refs inside
                                // (e.g. computed-key identifier references) leak.
                                ctx.drop_expression(&prop_key);
                            } else {
                                transformed_elements.push(prop_key);
                            }
                        }
                    }

                    if Self::remove_unused_expression(&mut value, ctx) {
                        // Same rationale as the key branch above — the property
                        // value is being dropped without a `replace_*` helper,
                        // so its references must be walked into `dirty.dead_refs`.
                        ctx.drop_expression(&value);
                    } else {
                        transformed_elements.push(value);
                    }
                }
            }
        }

        if !pending_spread_elements.is_empty() {
            transformed_elements.push(Expression::new_object_expression(
                object_expr.span,
                pending_spread_elements,
                ctx,
            ));
        }

        if transformed_elements.is_empty() {
            return true;
        } else if transformed_elements.len() == 1 {
            let new_expr = transformed_elements.pop().unwrap();
            ctx.replace_expression(e, new_expr);
            return false;
        }

        let new_expr =
            Expression::new_sequence_expression(object_expr.span, transformed_elements, ctx);
        ctx.replace_expression(e, new_expr);
        false
    }

    fn remove_unused_conditional_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        if !e.may_have_side_effects(ctx) {
            return true;
        }
        let Expression::ConditionalExpression(conditional_expr) = e else {
            return false;
        };

        let consequent = Self::remove_unused_expression(&mut conditional_expr.consequent, ctx);
        let alternate = Self::remove_unused_expression(&mut conditional_expr.alternate, ctx);

        // "foo() ? 1 : 2" => "foo()"
        if consequent && alternate {
            let test = Self::remove_unused_expression(&mut conditional_expr.test, ctx);
            if test {
                return true;
            }
            let new_expr = conditional_expr.test.take_in(ctx);
            ctx.replace_expression(e, new_expr);
            return false;
        }

        // "foo() ? 1 : bar()" => "foo() || bar()"
        if consequent {
            let new_expr = Self::join_with_left_associative_op(
                conditional_expr.span,
                LogicalOperator::Or,
                conditional_expr.test.take_in(ctx),
                conditional_expr.alternate.take_in(ctx),
                ctx,
            );
            ctx.replace_expression(e, new_expr);
            return false;
        }

        // "foo() ? bar() : 2" => "foo() && bar()"
        if alternate {
            let new_expr = Self::join_with_left_associative_op(
                conditional_expr.span,
                LogicalOperator::And,
                conditional_expr.test.take_in(ctx),
                conditional_expr.consequent.take_in(ctx),
                ctx,
            );
            ctx.replace_expression(e, new_expr);
            return false;
        }

        false
    }

    fn remove_unused_binary_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::BinaryExpression(binary_expr) = e else {
            return false;
        };

        match binary_expr.operator {
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => {
                let left = Self::remove_unused_expression(&mut binary_expr.left, ctx);
                let right = Self::remove_unused_expression(&mut binary_expr.right, ctx);
                match (left, right) {
                    (true, true) => true,
                    (true, false) => {
                        let new_expr = binary_expr.right.take_in(ctx);
                        ctx.replace_expression(e, new_expr);
                        false
                    }
                    (false, true) => {
                        let new_expr = binary_expr.left.take_in(ctx);
                        ctx.replace_expression(e, new_expr);
                        false
                    }
                    (false, false) => {
                        let new_expr = Expression::new_sequence_expression(
                            binary_expr.span,
                            ArenaVec::from_array_in(
                                [binary_expr.left.take_in(ctx), binary_expr.right.take_in(ctx)],
                                ctx,
                            ),
                            ctx,
                        );
                        ctx.replace_expression(e, new_expr);
                        false
                    }
                }
            }
            BinaryOperator::Addition => {
                Self::fold_string_addition_chain(e, ctx);
                matches!(e, Expression::StringLiteral(_))
            }
            _ => !e.may_have_side_effects(ctx),
        }
    }

    /// returns whether the passed expression is a string
    fn fold_string_addition_chain(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::BinaryExpression(binary_expr) = e else {
            return e.to_primitive(ctx).is_string() == Some(true);
        };
        if binary_expr.operator != BinaryOperator::Addition {
            return e.to_primitive(ctx).is_string() == Some(true);
        }

        let left_is_string = Self::fold_string_addition_chain(&mut binary_expr.left, ctx);
        if left_is_string {
            if !binary_expr.left.may_have_side_effects(ctx)
                && !binary_expr.left.is_specific_string_literal("")
            {
                let left_span = binary_expr.left.span();
                let new_left = Expression::new_string_literal(left_span, "", None, ctx);
                ctx.replace_expression(&mut binary_expr.left, new_left);
            }

            let right_as_primitive = binary_expr.right.to_primitive(ctx);
            if right_as_primitive.is_symbol() == Some(false)
                && !binary_expr.right.may_have_side_effects(ctx)
            {
                let new_expr = binary_expr.left.take_in(ctx);
                ctx.replace_expression(e, new_expr);
                return true;
            }
            return true;
        }

        let right_as_primitive = binary_expr.right.to_primitive(ctx);
        if right_as_primitive.is_string() == Some(true) {
            if !binary_expr.right.may_have_side_effects(ctx)
                && !binary_expr.right.is_specific_string_literal("")
            {
                let right_span = binary_expr.right.span();
                let new_right = Expression::new_string_literal(right_span, "", None, ctx);
                ctx.replace_expression(&mut binary_expr.right, new_right);
            }
            return true;
        }
        false
    }

    fn remove_unused_call_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::CallExpression(call_expr) = e else { return false };

        let is_pure = {
            (call_expr.pure && ctx.annotations())
                || ctx.manual_pure_functions(&call_expr.callee)
                || (if let Expression::Identifier(id) = &call_expr.callee
                    && let Some(symbol_id) =
                        ctx.scoping().get_reference(id.reference_id()).symbol_id()
                {
                    ctx.state.pure_functions.contains_key(&symbol_id)
                } else {
                    false
                })
        };

        if is_pure {
            let mut exprs =
                Self::fold_arguments_into_needed_expressions(&mut call_expr.arguments, ctx);
            if exprs.is_empty() {
                return true;
            } else if exprs.len() == 1 {
                let new_expr = exprs.pop().unwrap();
                ctx.replace_expression(e, new_expr);
                return false;
            }
            let new_expr = Expression::new_sequence_expression(call_expr.span, exprs, ctx);
            ctx.replace_expression(e, new_expr);
            return false;
        }

        !Self::has_side_effects_or_preserved_iife(e, ctx)
    }

    /// `Expression::may_have_side_effects`, except that in DCE-only mode an IIFE
    /// call (`(function () {...})()` / `(() => {...})()`) is reported as
    /// effectful so its structure survives — matching Rollup / esbuild
    /// tree-shaking (see `preserve_iife_in_dce_mode`). Full minification still
    /// drops a pure-bodied IIFE, just as it inlines IIFE bodies. Every
    /// unused-removal site that can face an IIFE call routes through here, so
    /// the policy lives in one place.
    pub(super) fn has_side_effects_or_preserved_iife(
        e: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        // Check the cheap DCE-preservation case first: in DCE-only mode an IIFE
        // call is always kept, so its (potentially deep) body walk is skipped.
        if ctx.state.dce
            && matches!(e, Expression::CallExpression(call) if call.callee.is_function())
        {
            return true;
        }
        e.may_have_side_effects(ctx)
    }

    pub fn fold_arguments_into_needed_expressions(
        args: &mut ArenaVec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Expression<'a>> {
        // `args.drain(..)` would silently move owned `Argument`s out and the
        // filter would drop any whose inner expression `remove_unused_expression`
        // collapsed to nothing — leaking references in the dropped subtree.
        // Use a manual loop so we can `drop_expression` before discarding.
        let mut out: ArenaVec<'a, Expression<'a>> = ArenaVec::with_capacity_in(args.len(), ctx);
        for arg in args.drain(..) {
            let mut expr = match arg {
                Argument::SpreadElement(e) => Expression::new_array_expression(
                    e.span,
                    ArenaVec::from_value_in(ArrayExpressionElement::SpreadElement(e), ctx),
                    ctx,
                ),
                match_expression!(Argument) => arg.into_expression(),
            };
            if Self::remove_unused_expression(&mut expr, ctx) {
                ctx.drop_expression(&expr);
            } else {
                out.push(expr);
            }
        }
        out
    }

    pub fn remove_unused_assignment_expr(
        e: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        let Expression::AssignmentExpression(assign_expr) = &*e else { return false };
        if matches!(
            ctx.state.options.unused,
            CompressOptionsUnused::Keep | CompressOptionsUnused::KeepAssign
        ) {
            return false;
        }
        // Member expression assignments (e.g. `A.from = () => {}`) use a different path.
        if !matches!(
            assign_expr.left.as_simple_assignment_target(),
            Some(SimpleAssignmentTarget::AssignmentTargetIdentifier(_))
        ) {
            return Self::remove_unused_member_assignment(e, ctx);
        }
        // Identifier assignments (e.g. `A = expr`).
        let Expression::AssignmentExpression(assign_expr) = e else { unreachable!() };
        let Some(SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)) =
            assign_expr.left.as_simple_assignment_target()
        else {
            unreachable!()
        };
        if Self::keep_top_level_var_in_script_mode(ctx)
            || ctx.current_scope_flags().contains_direct_eval()
        {
            return false;
        }
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else {
            return false;
        };
        // Keep error for assigning to `const foo = 1; foo = 2`.
        if ctx.scoping().symbol_flags(symbol_id).is_const_variable() {
            return false;
        }
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return false;
        };
        // Cannot remove assignment to live bindings: `export let foo; foo = 1;`.
        if symbol_value.exported {
            return false;
        }
        if symbol_value.read_references_count > 0 {
            return false;
        }
        let new_expr = assign_expr.right.take_in(ctx);
        ctx.replace_expression(e, new_expr);
        false
    }

    /// Try to remove a member expression assignment (e.g. `A.from = () => {}`).
    /// Checks side-effect analysis (respects `property_write_side_effects`) and
    /// verifies the root object is an unused local binding.
    fn remove_unused_member_assignment(e: &Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return false;
        }
        let Expression::AssignmentExpression(assign_expr) = e else { unreachable!() };

        // Track `__proto__` and computed member writes before checking side effects.
        // Even if this expression has side effects and can't be removed, we need to
        // record proto writes so that subsequent property writes on the same symbol
        // are preserved (the `__proto__` assignment may install setters).
        Self::track_proto_write(assign_expr, ctx);

        if e.may_have_side_effects(ctx) {
            return false;
        }
        Self::is_member_assign_to_unused_binding(assign_expr, ctx)
    }

    /// Track `__proto__` and computed member writes on local bindings.
    /// Must be called before side-effect checks so that proto writes with
    /// side-effectful RHS still mark the symbol.
    fn track_proto_write(assign_expr: &AssignmentExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Skip when property_write_side_effects is true (default) — the optimization
        // that drops member writes is disabled, so tracking is unnecessary.
        if ctx.state.options.treeshake.property_write_side_effects {
            return;
        }
        // Match only potential `__proto__` writes: explicit `a.__proto__` or
        // computed `a[b]` (where the key could evaluate to `"__proto__"`).
        let object = match &assign_expr.left {
            AssignmentTarget::StaticMemberExpression(e) if e.property.name == "__proto__" => {
                &e.object
            }
            // Computed key like `a[b]` could be `"__proto__"`
            AssignmentTarget::ComputedMemberExpression(e) => &e.object,
            _ => return,
        };
        let Expression::Identifier(ident) = object else { return };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else {
            return;
        };
        // Only mark if there are other member writes — if __proto__ is the only
        // reference, the setter is installed but never triggered, so dropping is safe.
        let ref_count = ctx.scoping().get_resolved_reference_ids(symbol_id).len();
        if ref_count > 1 {
            ctx.state.proto_write_symbols.insert(symbol_id);
        }
    }

    /// Resolve the symbol ID of a member assignment's base object identifier.
    /// Returns `None` for chained access (`a.b.c`), non-identifier bases, or
    /// non-member assignment targets.
    fn resolve_member_assign_object_symbol(
        assign_expr: &AssignmentExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<SymbolId> {
        // Only handle single-level member expressions (A.foo, not a.b.c).
        let object = match &assign_expr.left {
            AssignmentTarget::StaticMemberExpression(e) => &e.object,
            AssignmentTarget::ComputedMemberExpression(e) => &e.object,
            AssignmentTarget::PrivateFieldExpression(e) => &e.object,
            _ => return None,
        };
        let Expression::Identifier(ident) = object else { return None };
        ctx.scoping().get_reference(ident.reference_id()).symbol_id()
    }

    /// Check if a member expression assignment (e.g. `A.from = () => {}`) targets
    /// a local binding whose only references are property-write targets.
    ///
    /// Three conditions must hold:
    /// 1. The target is a single-level member expression (`A.foo`, not `a.b.c`)
    /// 2. ALL references to the symbol are member write targets
    /// 3. The symbol creates a fresh value (not an alias) and is not exported
    /// 4. The symbol has no `__proto__` member writes (which could install setters)
    fn is_member_assign_to_unused_binding(
        assign_expr: &AssignmentExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let Some(symbol_id) = Self::resolve_member_assign_object_symbol(assign_expr, ctx) else {
            return false;
        };

        // If this symbol has `__proto__` or computed member writes, don't drop any
        // property writes — the `__proto__` assignment may have installed setters.
        if ctx.state.proto_write_symbols.contains(&symbol_id) {
            return false;
        }

        // Check: symbol creates a fresh value (not an alias) and is not exported.
        let Some(sv) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return false;
        };
        if !sv.is_fresh_value || sv.exported {
            return false;
        }
        // Check: all references are member write targets (O(1) via pre-computed count).
        sv.write_references_count == 0
            && sv.read_references_count == sv.member_write_target_read_count
    }

    fn remove_unused_class_expr(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::ClassExpression(c) = e else { return false };
        if let Some(exprs) = Self::remove_unused_class(c, ctx) {
            if exprs.is_empty() {
                return true;
            }
            let span = c.span;
            let new_expr = Expression::new_sequence_expression(span, exprs, ctx);
            ctx.replace_expression(e, new_expr);
        }
        false
    }

    pub fn remove_unused_class(
        c: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<ArenaVec<'a, Expression<'a>>> {
        // TypeError `class C extends (() => {}) {}`
        if c.super_class
            .as_ref()
            .is_some_and(|e| matches!(e, Expression::ArrowFunctionExpression(_)))
        {
            return None;
        }
        // Don't remove classes with decorators - they may have side effects
        if !c.decorators.is_empty() {
            return None;
        }
        // Keep the entire class if there are class level side effects.
        for e in &c.body.body {
            match e {
                e if e.has_decorator() => return None,
                ClassElement::TSIndexSignature(_) => return None,
                ClassElement::StaticBlock(block) if !block.body.is_empty() => return None,
                ClassElement::PropertyDefinition(prop)
                    if prop.r#static
                        && prop.value.as_ref().is_some_and(|v| v.may_have_side_effects(ctx)) =>
                {
                    return None;
                }
                ClassElement::AccessorProperty(prop)
                    if prop.r#static
                        && prop.value.as_ref().is_some_and(|v| v.may_have_side_effects(ctx)) =>
                {
                    return None;
                }
                _ => {}
            }
        }

        // Otherwise extract the expressions.
        let mut exprs = ArenaVec::new_in(ctx);

        if let Some(e) = &mut c.super_class
            && e.may_have_side_effects(ctx)
        {
            exprs.push(c.super_class.take().unwrap());
        }

        for e in &mut c.body.body {
            // Save computed key.
            if e.computed()
                && let Some(key) = match e {
                    ClassElement::TSIndexSignature(_) | ClassElement::StaticBlock(_) => None,
                    ClassElement::MethodDefinition(def) => Some(&mut def.key),
                    ClassElement::PropertyDefinition(def) => Some(&mut def.key),
                    ClassElement::AccessorProperty(def) => Some(&mut def.key),
                }
                && let Some(expr) = key.as_expression_mut()
                && expr.may_have_side_effects(ctx)
            {
                exprs.push(expr.take_in(ctx));
            }
            // Save static initializer.
            if e.r#static()
                && let Some(init) = match e {
                    ClassElement::TSIndexSignature(_)
                    | ClassElement::StaticBlock(_)
                    | ClassElement::MethodDefinition(_) => None,
                    ClassElement::PropertyDefinition(def) => def.value.take(),
                    ClassElement::AccessorProperty(def) => def.value.take(),
                }
            {
                // Already checked side effects above.
                exprs.push(init);
            }
        }

        ctx.notice_change();
        Some(exprs)
    }
}
