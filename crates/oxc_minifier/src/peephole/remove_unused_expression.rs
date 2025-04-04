use std::iter;

use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ecmascript::{
    ToPrimitive,
    constant_evaluation::{DetermineValueType, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_syntax::es_target::ESTarget;

use crate::ctx::Ctx;

use super::{PeepholeOptimizations, State};

impl<'a> PeepholeOptimizations {
    /// `SimplifyUnusedExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L534>
    pub fn remove_unused_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        match e {
            Expression::ArrayExpression(_) => self.fold_array_expression(e, state, ctx),
            Expression::UnaryExpression(_) => self.fold_unary_expression(e, state, ctx),
            Expression::NewExpression(_) => self.fold_new_constructor(e, state, ctx),
            Expression::LogicalExpression(_) => self.fold_logical_expression(e, state, ctx),
            Expression::SequenceExpression(_) => self.fold_sequence_expression(e, state, ctx),
            Expression::TemplateLiteral(_) => self.fold_template_literal(e, state, ctx),
            Expression::ObjectExpression(_) => self.fold_object_expression(e, state, ctx),
            Expression::ConditionalExpression(_) => self.fold_conditional_expression(e, state, ctx),
            Expression::BinaryExpression(_) => self.fold_binary_expression(e, state, ctx),
            Expression::CallExpression(_) => self.fold_call_expression(e, state, ctx),
            _ => !e.may_have_side_effects(&ctx),
        }
    }

    fn fold_unary_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::UnaryExpression(unary_expr) = e else { return false };
        match unary_expr.operator {
            UnaryOperator::Void | UnaryOperator::LogicalNot => {
                *e = unary_expr.argument.take_in(ctx.ast.allocator);
                state.changed = true;
                self.remove_unused_expression(e, state, ctx)
            }
            UnaryOperator::Typeof => {
                if unary_expr.argument.is_identifier_reference() {
                    true
                } else {
                    *e = unary_expr.argument.take_in(ctx.ast.allocator);
                    state.changed = true;
                    self.remove_unused_expression(e, state, ctx)
                }
            }
            _ => false,
        }
    }

    fn fold_sequence_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::SequenceExpression(sequence_expr) = e else { return false };

        let old_len = sequence_expr.expressions.len();
        sequence_expr.expressions.retain_mut(|e| !self.remove_unused_expression(e, state, ctx));
        if sequence_expr.expressions.len() != old_len {
            state.changed = true;
        }

        sequence_expr.expressions.is_empty()
    }

    fn fold_logical_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::LogicalExpression(logical_expr) = e else { return false };
        if !logical_expr.operator.is_coalesce()
            && self.try_fold_expr_in_boolean_context(&mut logical_expr.left, ctx)
        {
            state.changed = true;
        }
        if self.remove_unused_expression(&mut logical_expr.right, state, ctx) {
            self.remove_unused_expression(&mut logical_expr.left, state, ctx);
            *e = logical_expr.left.take_in(ctx.ast.allocator);
            state.changed = true;
            return false;
        }

        // try optional chaining and nullish coalescing
        if self.target >= ESTarget::ES2020 {
            let LogicalExpression {
                span: logical_span,
                left: logical_left,
                right: logical_right,
                operator: logical_op,
            } = logical_expr.as_mut();
            if let Expression::BinaryExpression(binary_expr) = logical_left {
                match (logical_op, binary_expr.operator) {
                    // "a != null && a.b()" => "a?.b()"
                    // "a == null || a.b()" => "a?.b()"
                    (LogicalOperator::And, BinaryOperator::Inequality)
                    | (LogicalOperator::Or, BinaryOperator::Equality) => {
                        let name_and_id = if let Expression::Identifier(id) = &binary_expr.left {
                            (!ctx.is_global_reference(id) && binary_expr.right.is_null())
                                .then_some((id.name, &mut binary_expr.left))
                        } else if let Expression::Identifier(id) = &binary_expr.right {
                            (!ctx.is_global_reference(id) && binary_expr.left.is_null())
                                .then_some((id.name, &mut binary_expr.right))
                        } else {
                            None
                        };
                        if let Some((name, id)) = name_and_id {
                            if Self::inject_optional_chaining_if_matched(
                                &name,
                                id,
                                logical_right,
                                ctx,
                            ) {
                                *e = logical_right.take_in(ctx.ast.allocator);
                                state.changed = true;
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
                        let new_left_hand_expr = if binary_expr.right.is_null() {
                            Some(&mut binary_expr.left)
                        } else if binary_expr.left.is_null() {
                            Some(&mut binary_expr.right)
                        } else {
                            None
                        };
                        if let Some(new_left_hand_expr) = new_left_hand_expr {
                            if let Expression::AssignmentExpression(assignment_expr) = logical_right
                            {
                                if assignment_expr.operator == AssignmentOperator::Assign
                                    && Self::has_no_side_effect_for_evaluation_same_target(
                                        &assignment_expr.left,
                                        new_left_hand_expr,
                                        ctx,
                                    )
                                {
                                    assignment_expr.span = *logical_span;
                                    assignment_expr.operator = AssignmentOperator::LogicalNullish;
                                    *e = logical_right.take_in(ctx.ast.allocator);
                                    state.changed = true;
                                    return false;
                                }
                            }

                            *e = ctx.ast.expression_logical(
                                *logical_span,
                                new_left_hand_expr.take_in(ctx.ast.allocator),
                                LogicalOperator::Coalesce,
                                logical_right.take_in(ctx.ast.allocator),
                            );
                            state.changed = true;
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
    fn fold_array_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::ArrayExpression(array_expr) = e else {
            return false;
        };
        if array_expr.elements.is_empty() {
            return true;
        }

        let old_len = array_expr.elements.len();
        array_expr.elements.retain_mut(|el| match el {
            ArrayExpressionElement::SpreadElement(_) => el.may_have_side_effects(&ctx),
            ArrayExpressionElement::Elision(_) => false,
            match_expression!(ArrayExpressionElement) => {
                let el_expr = el.to_expression_mut();
                !self.remove_unused_expression(el_expr, state, ctx)
            }
        });
        if array_expr.elements.len() != old_len {
            state.changed = true;
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

    fn fold_new_constructor(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::NewExpression(new_expr) = e else { return false };

        if new_expr.pure {
            let mut exprs =
                self.fold_arguments_into_needed_expressions(&mut new_expr.arguments, state, ctx);
            if exprs.is_empty() {
                return true;
            } else if exprs.len() == 1 {
                *e = exprs.pop().unwrap();
                state.changed = true;
                return false;
            }
            *e = ctx.ast.expression_sequence(new_expr.span, exprs);
            state.changed = true;
            return false;
        }

        let Expression::Identifier(ident) = &new_expr.callee else { return false };
        let len = new_expr.arguments.len();
        if match ident.name.as_str() {
            "WeakSet" | "WeakMap" if ctx.is_global_reference(ident) => match len {
                0 => true,
                1 => match new_expr.arguments[0].as_expression() {
                    Some(Expression::NullLiteral(_)) => true,
                    Some(Expression::ArrayExpression(e)) => e.elements.is_empty(),
                    Some(e) if ctx.is_expression_undefined(e) => true,
                    _ => false,
                },
                _ => false,
            },
            "Date" if ctx.is_global_reference(ident) => match len {
                0 => true,
                1 => {
                    let Some(arg) = new_expr.arguments[0].as_expression() else { return false };
                    let ty = arg.value_type(&ctx);
                    matches!(
                        ty,
                        ValueType::Null
                            | ValueType::Undefined
                            | ValueType::Boolean
                            | ValueType::Number
                            | ValueType::String
                    ) && !arg.may_have_side_effects(&ctx)
                }
                _ => false,
            },
            "Set" | "Map" if ctx.is_global_reference(ident) => match len {
                0 => true,
                1 => match new_expr.arguments[0].as_expression() {
                    Some(Expression::NullLiteral(_)) => true,
                    Some(e) if ctx.is_expression_undefined(e) => true,
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        } {
            return true;
        }
        false
    }

    // "`${1}2${foo()}3`" -> "`${foo()}`"
    fn fold_template_literal(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::TemplateLiteral(temp_lit) = e else { return false };
        if temp_lit.expressions.is_empty() {
            return true;
        }
        if temp_lit.expressions.iter().all(|e| e.to_primitive(&ctx).is_symbol() != Some(false))
            && temp_lit.quasis.iter().all(|q| q.value.raw.is_empty())
        {
            return false;
        }

        let mut transformed_elements = ctx.ast.vec();
        let mut pending_to_string_required_exprs = ctx.ast.vec();

        for mut e in temp_lit.expressions.drain(..) {
            if e.to_primitive(&ctx).is_symbol() != Some(false) {
                pending_to_string_required_exprs.push(e);
            } else if !self.remove_unused_expression(&mut e, state, ctx) {
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
            state.changed = true;
            return false;
        }

        *e = ctx.ast.expression_sequence(temp_lit.span, transformed_elements);
        state.changed = true;
        false
    }

    // `({ 1: 1, [foo()]: bar() })` -> `foo(), bar()`
    fn fold_object_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
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
                        transformed_elements.push(ctx.ast.expression_object(
                            prop.span(),
                            pending_spread_elements,
                            None,
                        ));
                        pending_spread_elements = ctx.ast.vec();
                    }

                    let ObjectProperty { key, mut value, .. } = prop.unbox();
                    // ToPropertyKey(key) throws an error when ToPrimitive(key) throws an Error
                    // But we can ignore that by using the assumption.
                    match key {
                        PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {}
                        match_expression!(PropertyKey) => {
                            let mut prop_key = key.into_expression();
                            if !self.remove_unused_expression(&mut prop_key, state, ctx) {
                                transformed_elements.push(prop_key);
                            }
                        }
                    }

                    if !self.remove_unused_expression(&mut value, state, ctx) {
                        transformed_elements.push(value);
                    }
                }
            }
        }

        if !pending_spread_elements.is_empty() {
            transformed_elements.push(ctx.ast.expression_object(
                object_expr.span,
                pending_spread_elements,
                None,
            ));
        }

        if transformed_elements.is_empty() {
            return true;
        } else if transformed_elements.len() == 1 {
            *e = transformed_elements.pop().unwrap();
            state.changed = true;
            return false;
        }

        *e = ctx.ast.expression_sequence(object_expr.span, transformed_elements);
        state.changed = true;
        false
    }

    fn fold_conditional_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::ConditionalExpression(conditional_expr) = e else {
            return false;
        };

        let consequent =
            self.remove_unused_expression(&mut conditional_expr.consequent, state, ctx);
        let alternate = self.remove_unused_expression(&mut conditional_expr.alternate, state, ctx);

        // "foo() ? 1 : 2" => "foo()"
        if consequent && alternate {
            let test = self.remove_unused_expression(&mut conditional_expr.test, state, ctx);
            if test {
                return true;
            }
            *e = conditional_expr.test.take_in(ctx.ast.allocator);
            state.changed = true;
            return false;
        }

        // "foo() ? 1 : bar()" => "foo() || bar()"
        if consequent {
            *e = self.join_with_left_associative_op(
                conditional_expr.span,
                LogicalOperator::Or,
                conditional_expr.test.take_in(ctx.ast.allocator),
                conditional_expr.alternate.take_in(ctx.ast.allocator),
                ctx,
            );
            state.changed = true;
            return false;
        }

        // "foo() ? bar() : 2" => "foo() && bar()"
        if alternate {
            *e = self.join_with_left_associative_op(
                conditional_expr.span,
                LogicalOperator::And,
                conditional_expr.test.take_in(ctx.ast.allocator),
                conditional_expr.consequent.take_in(ctx.ast.allocator),
                ctx,
            );
            state.changed = true;
            return false;
        }

        false
    }

    fn fold_binary_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
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
                let left = self.remove_unused_expression(&mut binary_expr.left, state, ctx);
                let right = self.remove_unused_expression(&mut binary_expr.right, state, ctx);
                match (left, right) {
                    (true, true) => true,
                    (true, false) => {
                        *e = binary_expr.right.take_in(ctx.ast.allocator);
                        state.changed = true;
                        false
                    }
                    (false, true) => {
                        *e = binary_expr.left.take_in(ctx.ast.allocator);
                        state.changed = true;
                        false
                    }
                    (false, false) => {
                        *e = ctx.ast.expression_sequence(
                            binary_expr.span,
                            ctx.ast.vec_from_array([
                                binary_expr.left.take_in(ctx.ast.allocator),
                                binary_expr.right.take_in(ctx.ast.allocator),
                            ]),
                        );
                        state.changed = true;
                        false
                    }
                }
            }
            BinaryOperator::Addition => {
                Self::fold_string_addition_chain(e, state, ctx);
                matches!(e, Expression::StringLiteral(_))
            }
            _ => !e.may_have_side_effects(&ctx),
        }
    }

    /// returns whether the passed expression is a string
    fn fold_string_addition_chain(
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::BinaryExpression(binary_expr) = e else {
            return e.to_primitive(&ctx).is_string() == Some(true);
        };
        if binary_expr.operator != BinaryOperator::Addition {
            return e.to_primitive(&ctx).is_string() == Some(true);
        }

        let left_is_string = Self::fold_string_addition_chain(&mut binary_expr.left, state, ctx);
        if left_is_string {
            if !binary_expr.left.may_have_side_effects(&ctx)
                && !binary_expr.left.is_specific_string_literal("")
            {
                binary_expr.left =
                    ctx.ast.expression_string_literal(binary_expr.left.span(), "", None);
                state.changed = true;
            }

            let right_as_primitive = binary_expr.right.to_primitive(&ctx);
            if right_as_primitive.is_symbol() == Some(false)
                && !binary_expr.right.may_have_side_effects(&ctx)
            {
                *e = binary_expr.left.take_in(ctx.ast.allocator);
                state.changed = true;
                return true;
            }
            return true;
        }

        let right_as_primitive = binary_expr.right.to_primitive(&ctx);
        if right_as_primitive.is_string() == Some(true) {
            if !binary_expr.right.may_have_side_effects(&ctx)
                && !binary_expr.right.is_specific_string_literal("")
            {
                binary_expr.right =
                    ctx.ast.expression_string_literal(binary_expr.right.span(), "", None);
                state.changed = true;
            }
            return true;
        }
        false
    }

    fn fold_call_expression(
        &self,
        e: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        let Expression::CallExpression(call_expr) = e else { return false };

        if call_expr.pure {
            let mut exprs =
                self.fold_arguments_into_needed_expressions(&mut call_expr.arguments, state, ctx);
            if exprs.is_empty() {
                return true;
            } else if exprs.len() == 1 {
                *e = exprs.pop().unwrap();
                state.changed = true;
                return false;
            }
            *e = ctx.ast.expression_sequence(call_expr.span, exprs);
            state.changed = true;
            return false;
        }

        if call_expr.arguments.is_empty() {
            let is_empty_iife = match &call_expr.callee {
                Expression::FunctionExpression(f) => {
                    f.params.is_empty() && f.body.as_ref().is_some_and(|body| body.is_empty())
                }
                Expression::ArrowFunctionExpression(f) => f.params.is_empty() && f.body.is_empty(),
                _ => false,
            };
            if is_empty_iife {
                return true;
            }

            if let Expression::ArrowFunctionExpression(f) = &mut call_expr.callee {
                if !f.r#async && f.params.parameters_count() == 0 && f.body.statements.len() == 1 {
                    if f.expression {
                        // Replace "(() => foo())()" with "foo()"
                        let expr = f.get_expression_mut().unwrap();
                        *e = expr.take_in(ctx.ast.allocator);
                        return self.remove_unused_expression(e, state, ctx);
                    }
                    match &mut f.body.statements[0] {
                        Statement::ExpressionStatement(expr_stmt) => {
                            // Replace "(() => { foo() })" with "foo()"
                            *e = expr_stmt.expression.take_in(ctx.ast.allocator);
                            return self.remove_unused_expression(e, state, ctx);
                        }
                        Statement::ReturnStatement(ret_stmt) => {
                            if let Some(argument) = &mut ret_stmt.argument {
                                // Replace "(() => { return foo() })" with "foo()"
                                *e = argument.take_in(ctx.ast.allocator);
                                return self.remove_unused_expression(e, state, ctx);
                            }
                            // Replace "(() => { return })" with ""
                            return true;
                        }
                        _ => {}
                    }
                }
            }
        }

        false
    }

    fn fold_arguments_into_needed_expressions(
        &self,
        args: &mut Vec<'a, Argument<'a>>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) -> Vec<'a, Expression<'a>> {
        ctx.ast.vec_from_iter(args.drain(..).filter_map(|arg| {
            let mut expr = match arg {
                Argument::SpreadElement(e) => ctx.ast.expression_array(
                    e.span,
                    ctx.ast.vec1(ArrayExpressionElement::SpreadElement(e)),
                    None,
                ),
                match_expression!(Argument) => arg.into_expression(),
            };
            (!self.remove_unused_expression(&mut expr, state, ctx)).then_some(expr)
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    #[test]
    fn test_remove_unused_expression() {
        test("null", "");
        test("true", "");
        test("false", "");
        test("1", "");
        test("1n", "");
        test(";'s'", "");
        test("this", "");
        test("/asdf/", "");
        test("(function () {})", "");
        test("(() => {})", "");
        test("import.meta", "");
        test("var x; x", "var x");
        test("x", "x");
        test("void 0", "");
        test("void x", "x");
    }

    #[test]
    fn test_new_constructor_side_effect() {
        test("new WeakSet()", "");
        test("new WeakSet(null)", "");
        test("new WeakSet(void 0)", "");
        test("new WeakSet([])", "");
        test_same("new WeakSet([x])");
        test_same("new WeakSet(x)");
        test("new WeakMap()", "");
        test("new WeakMap(null)", "");
        test("new WeakMap(void 0)", "");
        test("new WeakMap([])", "");
        test_same("new WeakMap([x])");
        test_same("new WeakMap(x)");
        test("new Date()", "");
        test("new Date('')", "");
        test("new Date(0)", "");
        test("new Date(null)", "");
        test("new Date(true)", "");
        test("new Date(false)", "");
        test("new Date(undefined)", "");
        test_same("new Date(x)");
        test("new Set()", "");
        // test("new Set([a, b, c])", "");
        test("new Set(null)", "");
        test("new Set(undefined)", "");
        test("new Set(void 0)", "");
        test_same("new Set(x)");
        test("new Map()", "");
        test("new Map(null)", "");
        test("new Map(undefined)", "");
        test("new Map(void 0)", "");
        // test_same("new Map([x])");
        test_same("new Map(x)");
        // test("new Map([[a, b], [c, d]])", "");
    }

    #[test]
    fn test_array_literal() {
        test("([])", "");
        test("([1])", "");
        test("([a])", "a");
        test("var a; ([a])", "var a;");
        test("([foo()])", "foo()");
        test("[[foo()]]", "foo()");
        test_same("baz.map((v) => [v])");
    }

    #[test]
    fn test_array_literal_containing_spread() {
        test_same("([...c])");
        test("([4, ...c, a])", "[...c, a]");
        test("var a; ([4, ...c, a])", "var a; [...c]");
        test_same("([foo(), ...c, bar()])");
        test_same("([...a, b, ...c])");
        test("var b; ([...a, b, ...c])", "var b; [...a, ...c]");
        test_same("([...b, ...c])"); // It would also be fine if the spreads were split apart.
    }

    #[test]
    fn test_fold_unary_expression_statement() {
        test("typeof x", "");
        test("typeof x?.y", "x?.y");
        test("typeof x.y", "x.y");
        test("typeof x.y.z()", "x.y.z()");
        test("void x", "x");
        test("void x?.y", "x?.y");
        test("void x.y", "x.y");
        test("void x.y.z()", "x.y.z()");

        test("!x", "x");
        test("!x?.y", "x?.y");
        test("!x.y", "x.y");
        test("!x.y.z()", "x.y.z()");
        test_same("-x.y.z()");

        test_same("delete x");
        test_same("delete x.y");
        test_same("delete x.y.z()");
        test_same("+0n"); // Uncaught TypeError: Cannot convert a BigInt value to a number
    }

    #[test]
    fn test_fold_sequence_expr() {
        test("('foo', 'bar', 'baz')", "");
        test("('foo', 'bar', baz())", "baz()");
        test("('foo', bar(), baz())", "bar(), baz()");
        test("(() => {}, bar(), baz())", "bar(), baz()");
        test("(function k() {}, k(), baz())", "k(), baz()");
        test_same("(0, o.f)();");
        test("var obj = Object((null, 2, 3), 1, 2);", "var obj = Object(3, 1, 2);");
        test_same("(0 instanceof 0, foo)");
        test_same("(0 in 0, foo)");
        test_same(
            "React.useEffect(() => (isMountRef.current = !1, () => { isMountRef.current = !0; }), [])",
        );
    }

    #[test]
    fn test_logical_expression() {
        test("var a; a != null && a.b()", "var a; a?.b()");
        test("var a; a == null || a.b()", "var a; a?.b()");
        test_same("a != null && a.b()"); // a may have a getter
        test_same("a == null || a.b()"); // a may have a getter
        test("var a; null != a && a.b()", "var a; a?.b()");
        test("var a; null == a || a.b()", "var a; a?.b()");

        test("x == null && y", "x ?? y");
        test("x != null || y", "x ?? y");
        test_same("v = x == null && y");
        test_same("v = x != null || y");
        test("a == null && (a = b)", "a ??= b");
        test("a != null || (a = b)", "a ??= b");
        test_same("v = a == null && (a = b)");
        test_same("v = a != null || (a = b)");
        test("void (x == null && y)", "x ?? y");
    }

    #[test]
    fn test_object_literal() {
        test("({})", "");
        test("({a:1})", "");
        test("({a:foo()})", "foo()");
        test("({'a':foo()})", "foo()");
        // Object-spread may trigger getters.
        test_same("({...a})");
        test_same("({...foo()})");
        test("({ [{ foo: foo() }]: 0 })", "foo()");
        test("({ foo: { foo: foo() } })", "foo()");

        test("({ [bar()]: foo() })", "bar(), foo()");
        test("({ ...baz, [bar()]: foo() })", "({ ...baz }), bar(), foo()");
    }

    #[test]
    fn test_fold_template_literal() {
        test("`a${b}c${d}e`", "`${b}${d}`");
        test("`stuff ${x} ${1}`", "`${x}`");
        test("`stuff ${1} ${y}`", "`${y}`");
        test("`stuff ${x} ${y}`", "`${x}${y}`");
        test("`stuff ${x ? 1 : 2} ${y}`", "x, `${y}`");
        test("`stuff ${x} ${y ? 1 : 2}`", "`${x}`, y");
        test("`stuff ${x} ${y ? 1 : 2} ${z}`", "`${x}`, y, `${z}`");

        test("`4${c}${+a}`", "`${c}`, +a");
        test("`${+foo}${c}${+bar}`", "+foo, `${c}`, +bar");
        test("`${a}${+b}${c}`", "`${a}`, +b, `${c}`");
    }

    #[test]
    fn test_fold_conditional_expression() {
        test("(1, foo()) ? 1 : 2", "foo()");
        test("foo() ? 1 : 2", "foo()");
        test("foo() ? 1 : bar()", "foo() || bar()");
        test("foo() ? bar() : 2", "foo() && bar()");
        test_same("foo() ? bar() : baz()");
    }

    #[test]
    fn test_fold_binary_expression() {
        test("var a, b; a === b", "var a, b;");
        test("var a, b; a() === b", "var a, b; a()");
        test("var a, b; a === b()", "var a, b; b()");
        test("var a, b; a() === b()", "var a, b; a(), b()");

        test("var a, b; a !== b", "var a, b;");
        test("var a, b; a == b", "var a, b;");
        test("var a, b; a != b", "var a, b;");
        test("var a, b; a < b", "var a, b;");
        test("var a, b; a > b", "var a, b;");
        test("var a, b; a <= b", "var a, b;");
        test("var a, b; a >= b", "var a, b;");

        test_same("var a, b; a + b");
        test("var a, b; 'a' + b", "var a, b; '' + b");
        test_same("var a, b; a + '' + b");
        test("var a, b, c; 'a' + (b === c)", "var a, b, c;");
        test("var a, b; 'a' + +b", "var a, b; '' + +b"); // can be improved to "var a, b; +b"
        test_same("var a, b; a + ('' + b)");
        test("var a, b, c; a + ('' + (b === c))", "var a, b, c; a + ''");
    }

    #[test]
    fn test_fold_call_expression() {
        test_same("foo()");
        test("/* @__PURE__ */ foo()", "");
        test("/* @__PURE__ */ foo(a)", "a");
        test("/* @__PURE__ */ foo(a, b)", "a, b");
        test("/* @__PURE__ */ foo(...a)", "[...a]");
        test("/* @__PURE__ */ foo(...'a')", "");
        test("/* @__PURE__ */ new Foo()", "");
        test("/* @__PURE__ */ new Foo(a)", "a");
        test("true && /* @__PURE__ */ noEffect()", "");
        test("false || /* @__PURE__ */ noEffect()", "");
    }

    #[test]
    fn test_fold_iife() {
        test_same("var k = () => {}");
        test_same("var k = function () {}");
        // test("var a = (() => {})()", "var a = /* @__PURE__ */ (() => {})();");
        test("(() => {})()", "");
        test("(() => a())()", "a();");
        test("(() => { a() })()", "a();");
        test("(() => { return a() })()", "a();");
        test_same("(a => {})()");
        test_same("((a = foo()) => {})()");
        test_same("(a => { a() })()");
        test("((...a) => {})()", "");
        test_same("((...a) => { a() })()");
        // test("(() => { let b = a; b() })()", "a();");
        // test("(() => { let b = a; return b() })()", "a();");
        test("(async () => {})()", "");
        test_same("(async () => { a() })()");
        // test("(async () => { let b = a; b() })()", "(async () => a())();");
        // test("var a = (function() {})()", "var a = /* @__PURE__ */ function() {}();");
        test("(function() {})()", "");
        test("(function*() {})()", "");
        test("(async function() {})()", "");
        test_same("(function() { a() })()");
        test_same("(function*() { a() })()");
        test_same("(async function() { a() })()");
        test("(() => x)()", "x;");
        test("/* @__PURE__ */ (() => x)()", "");
        test("/* @__PURE__ */ (() => x)(y, z)", "y, z;");
    }

    #[test]
    fn no_side_effects() {
        fn check(source_text: &str) {
            let input = format!("{source_text}; f()");
            test(&input, source_text);

            let input = format!("{source_text}; new f()");
            test(&input, source_text);

            // TODO https://github.com/evanw/esbuild/issues/3511
            // let input = format!("{source_text}; html``");
            // test(&input, source_text);
        }
        check("/* @__NO_SIDE_EFFECTS__ */ function f() {}");
        check("/* @__NO_SIDE_EFFECTS__ */ export function f() {}");
        check("/* @__NO_SIDE_EFFECTS__ */ export default function f() {}");
        check("export default /* @__NO_SIDE_EFFECTS__ */ function f() {}");
        check("const f = /* @__NO_SIDE_EFFECTS__ */ function() {}");
        check("export const f = /* @__NO_SIDE_EFFECTS__ */ function() {}");
        check("/* @__NO_SIDE_EFFECTS__ */ const f = function() {}");
        check("/* @__NO_SIDE_EFFECTS__ */ export const f = function() {}");
        check("const f = /* @__NO_SIDE_EFFECTS__ */ () => {}");
        check("export const f = /* @__NO_SIDE_EFFECTS__ */ () => {}");
        check("/* @__NO_SIDE_EFFECTS__ */ const f = () => {}");
        check("/* @__NO_SIDE_EFFECTS__ */ export const f = () => {}");
    }
}
