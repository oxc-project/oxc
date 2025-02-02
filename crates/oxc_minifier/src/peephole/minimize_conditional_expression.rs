use oxc_ast::{ast::*, NONE};
use oxc_span::{cmp::ContentEq, GetSpan};
use oxc_syntax::es_target::ESTarget;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_conditional(
        &self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Expression<'a> {
        let mut cond_expr = ctx.ast.conditional_expression(span, test, consequent, alternate);
        self.try_minimize_conditional(&mut cond_expr, ctx)
            .unwrap_or_else(|| Expression::ConditionalExpression(ctx.ast.alloc(cond_expr)))
    }

    /// `MangleIfExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2745>
    pub fn try_minimize_conditional(
        &self,
        expr: &mut ConditionalExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        match &mut expr.test {
            // "(a, b) ? c : d" => "a, b ? c : d"
            Expression::SequenceExpression(sequence_expr) => {
                if sequence_expr.expressions.len() > 1 {
                    let span = expr.span();
                    let mut sequence = ctx.ast.move_expression(&mut expr.test);
                    let Expression::SequenceExpression(sequence_expr) = &mut sequence else {
                        unreachable!()
                    };
                    let expr = self.minimize_conditional(
                        span,
                        sequence_expr.expressions.pop().unwrap(),
                        ctx.ast.move_expression(&mut expr.consequent),
                        ctx.ast.move_expression(&mut expr.alternate),
                        ctx,
                    );
                    sequence_expr.expressions.push(expr);
                    return Some(sequence);
                }
            }
            // "!a ? b : c" => "a ? c : b"
            Expression::UnaryExpression(test_expr) => {
                if test_expr.operator.is_not() {
                    let test = ctx.ast.move_expression(&mut test_expr.argument);
                    let consequent = ctx.ast.move_expression(&mut expr.alternate);
                    let alternate = ctx.ast.move_expression(&mut expr.consequent);
                    return Some(
                        self.minimize_conditional(expr.span, test, consequent, alternate, ctx),
                    );
                }
            }
            Expression::Identifier(id) => {
                // "a ? a : b" => "a || b"
                if let Expression::Identifier(id2) = &expr.consequent {
                    if id.name == id2.name {
                        return Some(Self::join_with_left_associative_op(
                            expr.span,
                            LogicalOperator::Or,
                            ctx.ast.move_expression(&mut expr.test),
                            ctx.ast.move_expression(&mut expr.alternate),
                            ctx,
                        ));
                    }
                }
                // "a ? b : a" => "a && b"
                if let Expression::Identifier(id2) = &expr.alternate {
                    if id.name == id2.name {
                        return Some(Self::join_with_left_associative_op(
                            expr.span,
                            LogicalOperator::And,
                            ctx.ast.move_expression(&mut expr.test),
                            ctx.ast.move_expression(&mut expr.consequent),
                            ctx,
                        ));
                    }
                }
            }
            // `x != y ? b : c` -> `x == y ? c : b`
            Expression::BinaryExpression(test_expr) => {
                if matches!(
                    test_expr.operator,
                    BinaryOperator::Inequality | BinaryOperator::StrictInequality
                ) {
                    test_expr.operator = test_expr.operator.equality_inverse_operator().unwrap();
                    let test = ctx.ast.move_expression(&mut expr.test);
                    let consequent = ctx.ast.move_expression(&mut expr.consequent);
                    let alternate = ctx.ast.move_expression(&mut expr.alternate);
                    return Some(
                        self.minimize_conditional(expr.span, test, alternate, consequent, ctx),
                    );
                }
            }
            _ => {}
        }

        // "a ? true : false" => "!!a"
        // "a ? false : true" => "!a"
        if let (Expression::BooleanLiteral(left), Expression::BooleanLiteral(right)) =
            (&expr.consequent, &expr.alternate)
        {
            match (left.value, right.value) {
                (true, false) => {
                    let test = ctx.ast.move_expression(&mut expr.test);
                    let test = Self::minimize_not(expr.span, test, ctx);
                    let test = Self::minimize_not(expr.span, test, ctx);
                    return Some(test);
                }
                (false, true) => {
                    let test = ctx.ast.move_expression(&mut expr.test);
                    let test = Self::minimize_not(expr.span, test, ctx);
                    return Some(test);
                }
                _ => {}
            }
        }

        // "a ? b ? c : d : d" => "a && b ? c : d"
        if let Expression::ConditionalExpression(consequent) = &mut expr.consequent {
            if ctx.expr_eq(&consequent.alternate, &expr.alternate) {
                return Some(ctx.ast.expression_conditional(
                    expr.span,
                    Self::join_with_left_associative_op(
                        expr.test.span(),
                        LogicalOperator::And,
                        ctx.ast.move_expression(&mut expr.test),
                        ctx.ast.move_expression(&mut consequent.test),
                        ctx,
                    ),
                    ctx.ast.move_expression(&mut consequent.consequent),
                    ctx.ast.move_expression(&mut consequent.alternate),
                ));
            }
        }

        // "a ? b : c ? b : d" => "a || c ? b : d"
        if let Expression::ConditionalExpression(alternate) = &mut expr.alternate {
            if ctx.expr_eq(&alternate.consequent, &expr.consequent) {
                return Some(ctx.ast.expression_conditional(
                    expr.span,
                    Self::join_with_left_associative_op(
                        expr.test.span(),
                        LogicalOperator::Or,
                        ctx.ast.move_expression(&mut expr.test),
                        ctx.ast.move_expression(&mut alternate.test),
                        ctx,
                    ),
                    ctx.ast.move_expression(&mut expr.consequent),
                    ctx.ast.move_expression(&mut alternate.alternate),
                ));
            }
        }

        // "a ? c : (b, c)" => "(a || b), c"
        if let Expression::SequenceExpression(alternate) = &mut expr.alternate {
            if alternate.expressions.len() == 2
                && ctx.expr_eq(&alternate.expressions[1], &expr.consequent)
            {
                return Some(ctx.ast.expression_sequence(
                    expr.span,
                    ctx.ast.vec_from_array([
                        Self::join_with_left_associative_op(
                            expr.test.span(),
                            LogicalOperator::Or,
                            ctx.ast.move_expression(&mut expr.test),
                            ctx.ast.move_expression(&mut alternate.expressions[0]),
                            ctx,
                        ),
                        ctx.ast.move_expression(&mut expr.consequent),
                    ]),
                ));
            }
        }

        // "a ? (b, c) : c" => "(a && b), c"
        if let Expression::SequenceExpression(consequent) = &mut expr.consequent {
            if consequent.expressions.len() == 2
                && ctx.expr_eq(&consequent.expressions[1], &expr.alternate)
            {
                return Some(ctx.ast.expression_sequence(
                    expr.span,
                    ctx.ast.vec_from_array([
                        Self::join_with_left_associative_op(
                            expr.test.span(),
                            LogicalOperator::And,
                            ctx.ast.move_expression(&mut expr.test),
                            ctx.ast.move_expression(&mut consequent.expressions[0]),
                            ctx,
                        ),
                        ctx.ast.move_expression(&mut expr.alternate),
                    ]),
                ));
            }
        }

        // "a ? b || c : c" => "(a && b) || c"
        if let Expression::LogicalExpression(logical_expr) = &mut expr.consequent {
            if logical_expr.operator == LogicalOperator::Or
                && ctx.expr_eq(&logical_expr.right, &expr.alternate)
            {
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    Self::join_with_left_associative_op(
                        expr.test.span(),
                        LogicalOperator::And,
                        ctx.ast.move_expression(&mut expr.test),
                        ctx.ast.move_expression(&mut logical_expr.left),
                        ctx,
                    ),
                    LogicalOperator::Or,
                    ctx.ast.move_expression(&mut expr.alternate),
                ));
            }
        }

        // "a ? c : b && c" => "(a || b) && c"
        if let Expression::LogicalExpression(logical_expr) = &mut expr.alternate {
            if logical_expr.operator == LogicalOperator::And
                && ctx.expr_eq(&logical_expr.right, &expr.consequent)
            {
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    Self::join_with_left_associative_op(
                        expr.test.span(),
                        LogicalOperator::Or,
                        ctx.ast.move_expression(&mut expr.test),
                        ctx.ast.move_expression(&mut logical_expr.left),
                        ctx,
                    ),
                    LogicalOperator::And,
                    ctx.ast.move_expression(&mut expr.consequent),
                ));
            }
        }

        // `a ? b(c, d) : b(e, d)` -> `b(a ? c : e, d)`
        if let (
            Expression::Identifier(test),
            Expression::CallExpression(consequent),
            Expression::CallExpression(alternate),
        ) = (&expr.test, &mut expr.consequent, &mut expr.alternate)
        {
            if consequent.arguments.len() == alternate.arguments.len()
                && !ctx.is_global_reference(test)
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
                    let callee = ctx.ast.move_expression(&mut consequent.callee);
                    let consequent_first_arg = {
                        let Argument::SpreadElement(el) = &mut consequent.arguments[0] else {
                            unreachable!()
                        };
                        ctx.ast.move_expression(&mut el.argument)
                    };
                    let alternate_first_arg = {
                        let Argument::SpreadElement(el) = &mut alternate.arguments[0] else {
                            unreachable!()
                        };
                        ctx.ast.move_expression(&mut el.argument)
                    };
                    let mut args = std::mem::replace(&mut consequent.arguments, ctx.ast.vec());
                    args[0] = ctx.ast.argument_spread_element(
                        expr.span,
                        ctx.ast.expression_conditional(
                            expr.test.span(),
                            ctx.ast.move_expression(&mut expr.test),
                            consequent_first_arg,
                            alternate_first_arg,
                        ),
                    );
                    return Some(ctx.ast.expression_call(expr.span, callee, NONE, args, false));
                }
                // `a ? b(c) : b(e)` -> `b(a ? c : e)`
                if !matches!(consequent.arguments[0], Argument::SpreadElement(_))
                    && !matches!(alternate.arguments[0], Argument::SpreadElement(_))
                {
                    let callee = ctx.ast.move_expression(&mut consequent.callee);

                    let consequent_first_arg =
                        ctx.ast.move_expression(consequent.arguments[0].to_expression_mut());
                    let alternate_first_arg =
                        ctx.ast.move_expression(alternate.arguments[0].to_expression_mut());
                    let mut args = std::mem::replace(&mut consequent.arguments, ctx.ast.vec());
                    let cond_expr = self.minimize_conditional(
                        expr.test.span(),
                        ctx.ast.move_expression(&mut expr.test),
                        consequent_first_arg,
                        alternate_first_arg,
                        ctx,
                    );
                    args[0] = Argument::from(cond_expr);
                    return Some(ctx.ast.expression_call(expr.span, callee, NONE, args, false));
                }
            }
        }

        // Not part of esbuild
        if let Some(e) = self.try_merge_conditional_expression_inside(expr, ctx) {
            return Some(e);
        }

        // Try using the "??" or "?." operators
        if self.target >= ESTarget::ES2020 {
            if let Expression::BinaryExpression(test_binary) = &mut expr.test {
                if let Some(is_negate) = match test_binary.operator {
                    BinaryOperator::Inequality => Some(true),
                    BinaryOperator::Equality => Some(false),
                    _ => None,
                } {
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
                        // `a == null ? b : a` -> `a ?? b`
                        // `a != null ? a : b` -> `a ?? b`
                        // `(a = foo) == null ? b : a` -> `(a = foo) ?? b`
                        // `(a = foo) != null ? a : b` -> `(a = foo) ?? b`
                        let maybe_same_id_expr =
                            if is_negate { &mut expr.consequent } else { &mut expr.alternate };
                        if maybe_same_id_expr.is_specific_id(&target_id_name) {
                            return Some(ctx.ast.expression_logical(
                                expr.span,
                                ctx.ast.move_expression(value_expr),
                                LogicalOperator::Coalesce,
                                ctx.ast.move_expression(if is_negate {
                                    &mut expr.alternate
                                } else {
                                    &mut expr.consequent
                                }),
                            ));
                        }

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
                                return Some(
                                    ctx.ast.move_expression(expr_to_inject_optional_chaining),
                                );
                            }
                        }
                    }
                }
            }
        }

        if ctx.expr_eq(&expr.alternate, &expr.consequent) {
            // TODO:
            // "/* @__PURE__ */ a() ? b : b" => "b"
            // if ctx.ExprCanBeRemovedIfUnused(test) {
            // return yes
            // }

            // "a ? b : b" => "a, b"
            let expressions = ctx.ast.vec_from_array([
                ctx.ast.move_expression(&mut expr.test),
                ctx.ast.move_expression(&mut expr.consequent),
            ]);
            return Some(ctx.ast.expression_sequence(expr.span, expressions));
        }

        None
    }

    /// Merge `consequent` and `alternate` of `ConditionalExpression` inside.
    ///
    /// - `x ? a = 0 : a = 1` -> `a = x ? 0 : 1`
    fn try_merge_conditional_expression_inside(
        &self,
        expr: &mut ConditionalExpression<'a>,
        ctx: Ctx<'a, '_>,
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
        if consequent.right.is_anonymous_function_definition() {
            return None;
        }
        if consequent.operator != AssignmentOperator::Assign
            || consequent.operator != alternate.operator
            || consequent.left.content_ne(&alternate.left)
        {
            return None;
        }
        let cond_expr = self.minimize_conditional(
            expr.span,
            ctx.ast.move_expression(&mut expr.test),
            ctx.ast.move_expression(&mut consequent.right),
            ctx.ast.move_expression(&mut alternate.right),
            ctx,
        );
        Some(ctx.ast.expression_assignment(
            expr.span,
            consequent.operator,
            ctx.ast.move_assignment_target(&mut alternate.left),
            cond_expr,
        ))
    }

    /// Modify `expr` if that has `target_expr` as a parent, and returns true if modified.
    ///
    /// For `target_expr` = `a`, `expr` = `a.b`, this function changes `expr` to `a?.b` and returns true.
    fn inject_optional_chaining_if_matched(
        target_id_name: &str,
        expr_to_inject: &mut Expression<'a>,
        expr: &mut Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> bool {
        if Self::inject_optional_chaining_if_matched_inner(
            target_id_name,
            expr_to_inject,
            expr,
            ctx,
        ) {
            if !matches!(expr, Expression::ChainExpression(_)) {
                *expr = ctx.ast.expression_chain(
                    expr.span(),
                    ctx.ast.move_expression(expr).into_chain_element().unwrap(),
                );
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
        ctx: Ctx<'a, '_>,
    ) -> bool {
        match expr {
            Expression::StaticMemberExpression(e) => {
                if e.object.is_specific_id(target_id_name) {
                    e.optional = true;
                    e.object = ctx.ast.move_expression(expr_to_inject);
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
                    e.object = ctx.ast.move_expression(expr_to_inject);
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
                    e.callee = ctx.ast.move_expression(expr_to_inject);
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
                        e.object = ctx.ast.move_expression(expr_to_inject);
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
                        e.object = ctx.ast.move_expression(expr_to_inject);
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
                        e.callee = ctx.ast.move_expression(expr_to_inject);
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
}

#[cfg(test)]
mod test {
    use oxc_syntax::es_target::ESTarget;

    use crate::{
        tester::{run, test, test_same},
        CompressOptions,
    };

    fn test_es2019(source_text: &str, expected: &str) {
        let target = ESTarget::ES2019;
        assert_eq!(
            run(source_text, Some(CompressOptions { target, ..CompressOptions::default() })),
            run(expected, None)
        );
    }

    #[test]
    fn test_minimize_expr_condition() {
        test("(x ? true : false) && y()", "x && y()");
        test("(x ? false : true) && y()", "!x && y()");
        test("(x ? true : y) && y()", "(x || y) && y();");
        test("(x ? y : false) && y()", "(x && y) && y()");
        test("var x; (x && true) && y()", "var x; x && y()");
        test("var x; (x && false) && y()", "var x; x && !1");
        test("(x && true) && y()", "x && y()");
        test("(x && false) && y()", "x && !1");
        test("var x; (x || true) && y()", "var x; x || !0, y()");
        test("var x; (x || false) && y()", "var x; x && y()");

        test("(x || true) && y()", "x || !0, y()");
        test("(x || false) && y()", "x && y()");

        test("let x = foo ? true : false", "let x = !!foo");
        test("let x = foo ? true : bar", "let x = foo ? !0 : bar");
        test("let x = foo ? bar : false", "let x = foo ? bar : !1");
        test("function x () { return a ? true : false }", "function x() { return !!a }");
        test("function x () { return a ? false : true }", "function x() { return !a }");
        test("function x () { return a ? true : b }", "function x() { return a ? !0 : b }");
        // can't be minified e.g. `a = ''` would return `''`
        test("function x() { return a && true }", "function x() { return a && !0 }");

        test("foo ? bar : bar", "foo, bar");
        test_same("foo ? bar : baz");
        test("foo() ? bar : bar", "foo(), bar");

        test_same("var k = () => !!x;");
    }

    #[test]
    fn minimize_conditional_exprs() {
        test("(a, b) ? c : d", "a, b ? c : d");
        test("!a ? b : c", "a ? c : b");
        // test("/* @__PURE__ */ a() ? b : b", "b");
        test("a ? b : b", "a, b");
        test("a ? true : false", "a");
        test("a ? false : true", "!a");
        test("a ? a : b", "a || b");
        test("a ? b : a", "a && b");
        test("a ? b ? c : d : d", "a && b ? c : d");
        test("a ? b : c ? b : d", "a || c ? b : d");
        test("a ? c : (b, c)", "(a || b), c");
        test("a ? (b, c) : c", "(a && b), c");
        test("a ? b || c : c", "(a && b) || c");
        test("a ? c : b && c", "(a || b) && c");
        test("var a; a ? b(c, d) : b(e, d)", "var a; b(a ? c : e, d)");
        test("var a; a ? b(...c) : b(...e)", "var a; b(...a ? c : e)");
        test("var a; a ? b(c) : b(e)", "var a; b(a ? c : e)");
        test("a() != null ? a() : b", "a() == null ? b : a()");
        test("var a; a != null ? a : b", "var a; a ?? b");
        test("var a; (a = _a) != null ? a : b", "var a; (a = _a) ?? b");
        test("a != null ? a : b", "a == null ? b : a"); // accessing global `a` may have a getter with side effects
        test_es2019("var a; a != null ? a : b", "var a; a == null ? b : a");
        test("var a; a != null ? a.b.c[d](e) : undefined", "var a; a?.b.c[d](e)");
        test("var a; (a = _a) != null ? a.b.c[d](e) : undefined", "var a; (a = _a)?.b.c[d](e)");
        test("a != null ? a.b.c[d](e) : undefined", "a != null && a.b.c[d](e)"); // accessing global `a` may have a getter with side effects
        test(
            "var a, undefined = 1; a != null ? a.b.c[d](e) : undefined",
            "var a, undefined = 1; a == null ? undefined : a.b.c[d](e)",
        );
        test_es2019(
            "var a; a != null ? a.b.c[d](e) : undefined",
            "var a; a != null && a.b.c[d](e)",
        );
        test("cmp !== 0 ? cmp : (bar, cmp);", "cmp === 0 && bar, cmp;");
        test("cmp === 0 ? cmp : (bar, cmp);", "cmp === 0 || bar, cmp;");
        test("cmp !== 0 ? (bar, cmp) : cmp;", "cmp === 0 || bar, cmp;");
        test("cmp === 0 ? (bar, cmp) : cmp;", "cmp === 0 && bar, cmp;");
    }

    #[test]
    fn compress_conditional() {
        test("foo ? foo : bar", "foo || bar");
        test("foo ? bar : foo", "foo && bar");
        test_same("x.y ? x.y : bar");
        test_same("x.y ? bar : x.y");
    }
}
