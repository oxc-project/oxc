use std::iter;

use crate::{CompressOptionsUnused, ctx::Ctx};
use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_compat::ESFeature;
use oxc_ecmascript::{
    ToPrimitive,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext},
};
use oxc_span::GetSpan;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `SimplifyUnusedExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L534>
    pub fn remove_unused_expression(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
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
            _ => !e.may_have_side_effects(ctx),
        }
    }

    fn remove_unused_unary_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::UnaryExpression(unary_expr) = e else { return false };
        match unary_expr.operator {
            UnaryOperator::Void | UnaryOperator::LogicalNot => {
                *e = unary_expr.argument.take_in(ctx.ast);
                ctx.state.changed = true;
                Self::remove_unused_expression(e, ctx)
            }
            UnaryOperator::Typeof => {
                if unary_expr.argument.is_identifier_reference() {
                    true
                } else {
                    *e = unary_expr.argument.take_in(ctx.ast);
                    ctx.state.changed = true;
                    Self::remove_unused_expression(e, ctx)
                }
            }
            _ => !e.may_have_side_effects(ctx),
        }
    }

    fn remove_unused_sequence_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::SequenceExpression(sequence_expr) = e else { return false };
        let old_len = sequence_expr.expressions.len();
        sequence_expr.expressions.retain_mut(|e| !Self::remove_unused_expression(e, ctx));
        if sequence_expr.expressions.len() != old_len {
            ctx.state.changed = true;
        }
        sequence_expr.expressions.is_empty()
    }

    fn remove_unused_logical_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        if !e.may_have_side_effects(ctx) {
            return true;
        }
        let Expression::LogicalExpression(logical_expr) = e else { return false };
        if !logical_expr.operator.is_coalesce() {
            Self::minimize_expression_in_boolean_context(&mut logical_expr.left, ctx);
        }
        if Self::remove_unused_expression(&mut logical_expr.right, ctx) {
            Self::remove_unused_expression(&mut logical_expr.left, ctx);
            *e = logical_expr.left.take_in(ctx.ast);
            ctx.state.changed = true;
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
                    | (LogicalOperator::Or, BinaryOperator::Equality) => {
                        if ctx.supports_feature(ESFeature::ES2020OptionalChaining) {
                            let name_and_id = if let Expression::Identifier(id) = &binary_expr.left
                            {
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
                                *e = logical_right.take_in(ctx.ast);
                                ctx.state.changed = true;
                                return false;
                            }
                        }
                    }
                    // "a == null && b" => "a ?? b"
                    // "a != null || b" => "a ?? b"
                    // "a == null && (a = b)" => "a ??= b"
                    // "a != null || (a = b)" => "a ??= b"
                    (LogicalOperator::And, BinaryOperator::Equality)
                    | (LogicalOperator::Or, BinaryOperator::Inequality) => {
                        if ctx.supports_feature(ESFeature::ES2020NullishCoalescingOperator) {
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
                                    *e = logical_right.take_in(ctx.ast);
                                    ctx.state.changed = true;
                                    return false;
                                }

                                *e = ctx.ast.expression_logical(
                                    *logical_span,
                                    new_left_hand_expr.take_in(ctx.ast),
                                    LogicalOperator::Coalesce,
                                    logical_right.take_in(ctx.ast),
                                );
                                ctx.state.changed = true;
                                return false;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        false
    }

    // `([1,2,3, foo()])` -> `foo()`
    fn remove_unused_array_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::ArrayExpression(array_expr) = e else {
            return false;
        };
        if array_expr.elements.is_empty() {
            return true;
        }

        let old_len = array_expr.elements.len();
        array_expr.elements.retain_mut(|el| match el {
            ArrayExpressionElement::SpreadElement(_) => el.may_have_side_effects(ctx),
            ArrayExpressionElement::Elision(_) => false,
            match_expression!(ArrayExpressionElement) => {
                let el_expr = el.to_expression_mut();
                !Self::remove_unused_expression(el_expr, ctx)
            }
        });
        if array_expr.elements.len() != old_len {
            ctx.state.changed = true;
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

        let mut expressions = ctx.ast.vec_from_iter(
            array_expr.elements.drain(..).map(ArrayExpressionElement::into_expression),
        );
        if expressions.is_empty() {
            return true;
        } else if expressions.len() == 1 {
            *e = expressions.pop().unwrap();
            return false;
        }

        *e = ctx.ast.expression_sequence(array_expr.span, expressions);
        false
    }

    fn remove_unused_new_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::NewExpression(new_expr) = e else { return false };
        if (new_expr.pure && ctx.annotations()) || ctx.manual_pure_functions(&new_expr.callee) {
            let mut exprs =
                Self::fold_arguments_into_needed_expressions(&mut new_expr.arguments, ctx);
            if exprs.is_empty() {
                return true;
            } else if exprs.len() == 1 {
                *e = exprs.pop().unwrap();
                ctx.state.changed = true;
                return false;
            }
            *e = ctx.ast.expression_sequence(new_expr.span, exprs);
            ctx.state.changed = true;
            return false;
        }
        false
    }

    // "`${1}2${foo()}3`" -> "`${foo()}`"
    fn remove_unused_template_literal(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::TemplateLiteral(temp_lit) = e else { return false };
        if temp_lit.expressions.is_empty() {
            return true;
        }
        if temp_lit.expressions.iter().all(|e| e.to_primitive(ctx).is_symbol() != Some(false))
            && temp_lit.quasis.iter().all(|q| q.value.raw.is_empty())
        {
            return false;
        }

        let mut transformed_elements = ctx.ast.vec();
        let mut pending_to_string_required_exprs = ctx.ast.vec();

        for mut e in temp_lit.expressions.drain(..) {
            if e.to_primitive(ctx).is_symbol() != Some(false) {
                pending_to_string_required_exprs.push(e);
            } else if !Self::remove_unused_expression(&mut e, ctx) {
                if !pending_to_string_required_exprs.is_empty() {
                    // flush pending to string required expressions
                    let expressions =
                        ctx.ast.vec_from_iter(pending_to_string_required_exprs.drain(..));
                    let mut quasis = ctx.ast.vec_from_iter(
                        iter::repeat_with(|| {
                            ctx.ast.template_element(
                                e.span(),
                                TemplateElementValue { raw: "".into(), cooked: Some("".into()) },
                                false,
                            )
                        })
                        .take(expressions.len() + 1),
                    );
                    quasis
                        .last_mut()
                        .expect("template literal must have at least one quasi")
                        .tail = true;
                    transformed_elements.push(ctx.ast.expression_template_literal(
                        e.span(),
                        quasis,
                        expressions,
                    ));
                }
                transformed_elements.push(e);
            }
        }

        if !pending_to_string_required_exprs.is_empty() {
            let expressions = ctx.ast.vec_from_iter(pending_to_string_required_exprs.drain(..));
            let mut quasis = ctx.ast.vec_from_iter(
                iter::repeat_with(|| {
                    ctx.ast.template_element(
                        temp_lit.span,
                        TemplateElementValue { raw: "".into(), cooked: Some("".into()) },
                        false,
                    )
                })
                .take(expressions.len() + 1),
            );
            quasis.last_mut().expect("template literal must have at least one quasi").tail = true;
            transformed_elements.push(ctx.ast.expression_template_literal(
                temp_lit.span,
                quasis,
                expressions,
            ));
        }

        if transformed_elements.is_empty() {
            return true;
        } else if transformed_elements.len() == 1 {
            *e = transformed_elements.pop().unwrap();
            ctx.state.changed = true;
            return false;
        }

        *e = ctx.ast.expression_sequence(temp_lit.span, transformed_elements);
        ctx.state.changed = true;
        false
    }

    // `({ 1: 1, [foo()]: bar() })` -> `foo(), bar()`
    fn remove_unused_object_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::ObjectExpression(object_expr) = e else {
            return false;
        };
        if object_expr.properties.is_empty() {
            return true;
        }
        if object_expr.properties.iter().all(ObjectPropertyKind::is_spread) {
            return false;
        }

        let mut transformed_elements = ctx.ast.vec();
        let mut pending_spread_elements = ctx.ast.vec();

        for prop in object_expr.properties.drain(..) {
            match prop {
                ObjectPropertyKind::SpreadProperty(_) => {
                    pending_spread_elements.push(prop);
                }
                ObjectPropertyKind::ObjectProperty(prop) => {
                    if !pending_spread_elements.is_empty() {
                        // flush pending spread elements
                        transformed_elements
                            .push(ctx.ast.expression_object(prop.span(), pending_spread_elements));
                        pending_spread_elements = ctx.ast.vec();
                    }

                    let ObjectProperty { key, mut value, .. } = prop.unbox();
                    // ToPropertyKey(key) throws an error when ToPrimitive(key) throws an Error
                    // But we can ignore that by using the assumption.
                    match key {
                        PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {}
                        match_expression!(PropertyKey) => {
                            let mut prop_key = key.into_expression();
                            if !Self::remove_unused_expression(&mut prop_key, ctx) {
                                transformed_elements.push(prop_key);
                            }
                        }
                    }

                    if !Self::remove_unused_expression(&mut value, ctx) {
                        transformed_elements.push(value);
                    }
                }
            }
        }

        if !pending_spread_elements.is_empty() {
            transformed_elements
                .push(ctx.ast.expression_object(object_expr.span, pending_spread_elements));
        }

        if transformed_elements.is_empty() {
            return true;
        } else if transformed_elements.len() == 1 {
            *e = transformed_elements.pop().unwrap();
            ctx.state.changed = true;
            return false;
        }

        *e = ctx.ast.expression_sequence(object_expr.span, transformed_elements);
        ctx.state.changed = true;
        false
    }

    fn remove_unused_conditional_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
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
            *e = conditional_expr.test.take_in(ctx.ast);
            ctx.state.changed = true;
            return false;
        }

        // "foo() ? 1 : bar()" => "foo() || bar()"
        if consequent {
            *e = Self::join_with_left_associative_op(
                conditional_expr.span,
                LogicalOperator::Or,
                conditional_expr.test.take_in(ctx.ast),
                conditional_expr.alternate.take_in(ctx.ast),
                ctx,
            );
            ctx.state.changed = true;
            return false;
        }

        // "foo() ? bar() : 2" => "foo() && bar()"
        if alternate {
            *e = Self::join_with_left_associative_op(
                conditional_expr.span,
                LogicalOperator::And,
                conditional_expr.test.take_in(ctx.ast),
                conditional_expr.consequent.take_in(ctx.ast),
                ctx,
            );
            ctx.state.changed = true;
            return false;
        }

        false
    }

    fn remove_unused_binary_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
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
                        *e = binary_expr.right.take_in(ctx.ast);
                        ctx.state.changed = true;
                        false
                    }
                    (false, true) => {
                        *e = binary_expr.left.take_in(ctx.ast);
                        ctx.state.changed = true;
                        false
                    }
                    (false, false) => {
                        *e = ctx.ast.expression_sequence(
                            binary_expr.span,
                            ctx.ast.vec_from_array([
                                binary_expr.left.take_in(ctx.ast),
                                binary_expr.right.take_in(ctx.ast),
                            ]),
                        );
                        ctx.state.changed = true;
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
    fn fold_string_addition_chain(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
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
                binary_expr.left =
                    ctx.ast.expression_string_literal(binary_expr.left.span(), "", None);
                ctx.state.changed = true;
            }

            let right_as_primitive = binary_expr.right.to_primitive(ctx);
            if right_as_primitive.is_symbol() == Some(false)
                && !binary_expr.right.may_have_side_effects(ctx)
            {
                *e = binary_expr.left.take_in(ctx.ast);
                ctx.state.changed = true;
                return true;
            }
            return true;
        }

        let right_as_primitive = binary_expr.right.to_primitive(ctx);
        if right_as_primitive.is_string() == Some(true) {
            if !binary_expr.right.may_have_side_effects(ctx)
                && !binary_expr.right.is_specific_string_literal("")
            {
                binary_expr.right =
                    ctx.ast.expression_string_literal(binary_expr.right.span(), "", None);
                ctx.state.changed = true;
            }
            return true;
        }
        false
    }

    fn remove_unused_call_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
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
                *e = exprs.pop().unwrap();
                ctx.state.changed = true;
                return false;
            }
            *e = ctx.ast.expression_sequence(call_expr.span, exprs);
            ctx.state.changed = true;
            return false;
        }

        !e.may_have_side_effects(ctx)
    }

    pub fn fold_arguments_into_needed_expressions(
        args: &mut Vec<'a, Argument<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Vec<'a, Expression<'a>> {
        ctx.ast.vec_from_iter(args.drain(..).filter_map(|arg| {
            let mut expr = match arg {
                Argument::SpreadElement(e) => ctx.ast.expression_array(
                    e.span,
                    ctx.ast.vec1(ArrayExpressionElement::SpreadElement(e)),
                ),
                match_expression!(Argument) => arg.into_expression(),
            };
            (!Self::remove_unused_expression(&mut expr, ctx)).then_some(expr)
        }))
    }

    pub fn remove_unused_assignment_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::AssignmentExpression(assign_expr) = e else { return false };
        if matches!(
            ctx.state.options.unused,
            CompressOptionsUnused::Keep | CompressOptionsUnused::KeepAssign
        ) {
            return false;
        }
        let Some(SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)) =
            assign_expr.left.as_simple_assignment_target()
        else {
            return false;
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
        *e = assign_expr.right.take_in(ctx.ast);
        ctx.state.changed = true;
        false
    }

    fn remove_unused_class_expr(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let Expression::ClassExpression(c) = e else { return false };
        if let Some(exprs) = Self::remove_unused_class(c, ctx) {
            if exprs.is_empty() {
                return true;
            }
            *e = ctx.ast.expression_sequence(c.span, exprs);
        }
        false
    }

    pub fn remove_unused_class(
        c: &mut Class<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Vec<'a, Expression<'a>>> {
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
        let mut exprs = ctx.ast.vec();

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
                exprs.push(expr.take_in(ctx.ast));
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

        ctx.state.changed = true;
        Some(exprs)
    }
}
