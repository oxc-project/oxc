use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{DetermineValueType, IsLiteralValue, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `SimplifyUnusedExpr`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L534>
    pub fn remove_unused_expression(&self, e: &mut Expression<'a>, ctx: Ctx<'a, '_>) -> bool {
        match e {
            Expression::ArrayExpression(_) => Self::fold_array_expression(e, ctx),
            Expression::UnaryExpression(_) => self.fold_unary_expression(e, ctx),
            Expression::NewExpression(e) => Self::fold_new_constructor(e, ctx),
            Expression::LogicalExpression(_) => self.fold_logical_expression(e, ctx),
            Expression::SequenceExpression(_) => self.fold_sequence_expression(e, ctx),
            // TODO
            // Expression::TemplateLiteral(_)
            // | Expression::ObjectExpression(_)
            // | Expression::ConditionalExpression(_)
            // | Expression::BinaryExpression(_)
            // | Expression::CallExpression(_)
            // | Expression::NewExpression(_) => {
            // false
            // }
            _ => !e.may_have_side_effects(&ctx),
        }
    }

    fn fold_unary_expression(&self, e: &mut Expression<'a>, ctx: Ctx<'a, '_>) -> bool {
        let Expression::UnaryExpression(unary_expr) = e else { return false };
        match unary_expr.operator {
            UnaryOperator::Void | UnaryOperator::LogicalNot => {
                *e = ctx.ast.move_expression(&mut unary_expr.argument);
                self.remove_unused_expression(e, ctx)
            }
            UnaryOperator::Typeof => {
                if unary_expr.argument.is_identifier_reference() {
                    true
                } else {
                    *e = ctx.ast.move_expression(&mut unary_expr.argument);
                    self.remove_unused_expression(e, ctx)
                }
            }
            _ => false,
        }
    }

    fn fold_sequence_expression(&self, e: &mut Expression<'a>, ctx: Ctx<'a, '_>) -> bool {
        let Expression::SequenceExpression(sequence_expr) = e else { return false };
        sequence_expr.expressions.retain_mut(|e| !self.remove_unused_expression(e, ctx));
        sequence_expr.expressions.is_empty()
    }

    fn fold_logical_expression(&self, e: &mut Expression<'a>, ctx: Ctx<'a, '_>) -> bool {
        let Expression::LogicalExpression(logical_expr) = e else { return false };
        if !logical_expr.operator.is_coalesce() {
            self.try_fold_expr_in_boolean_context(&mut logical_expr.left, ctx);
        }
        if self.remove_unused_expression(&mut logical_expr.right, ctx) {
            self.remove_unused_expression(&mut logical_expr.left, ctx);
            *e = ctx.ast.move_expression(&mut logical_expr.left);
        }
        false
    }

    // `([1,2,3, foo()])` -> `foo()`
    fn fold_array_expression(e: &mut Expression<'a>, ctx: Ctx<'a, '_>) -> bool {
        let Expression::ArrayExpression(array_expr) = e else {
            return false;
        };

        let mut transformed_elements = ctx.ast.vec();
        let mut pending_spread_elements = ctx.ast.vec();

        if array_expr.elements.len() == 0 {
            return true;
        }

        if array_expr
            .elements
            .iter()
            .any(|el| matches!(el, ArrayExpressionElement::SpreadElement(_)))
        {
            return false;
        }

        for el in &mut array_expr.elements {
            match el {
                ArrayExpressionElement::SpreadElement(_) => {
                    let spread_element = ctx.ast.move_array_expression_element(el);
                    pending_spread_elements.push(spread_element);
                }
                ArrayExpressionElement::Elision(_) => {}
                match_expression!(ArrayExpressionElement) => {
                    let el = el.to_expression_mut();
                    let el_expr = ctx.ast.move_expression(el);
                    if !el_expr.is_literal_value(false)
                        && !matches!(&el_expr, Expression::Identifier(ident) if !ctx.is_global_reference(ident))
                    {
                        if pending_spread_elements.len() > 0 {
                            // flush pending spread elements
                            transformed_elements.push(ctx.ast.expression_array(
                                el_expr.span(),
                                pending_spread_elements,
                                None,
                            ));
                            pending_spread_elements = ctx.ast.vec();
                        }
                        transformed_elements.push(el_expr);
                    }
                }
            }
        }

        if pending_spread_elements.len() > 0 {
            transformed_elements.push(ctx.ast.expression_array(
                array_expr.span,
                pending_spread_elements,
                None,
            ));
        }

        if transformed_elements.is_empty() {
            return true;
        } else if transformed_elements.len() == 1 {
            *e = transformed_elements.pop().unwrap();
            return false;
        }

        *e = ctx.ast.expression_sequence(array_expr.span, transformed_elements);
        false
    }

    fn fold_new_constructor(e: &mut NewExpression<'a>, ctx: Ctx<'a, '_>) -> bool {
        let Expression::Identifier(ident) = &e.callee else { return false };
        let len = e.arguments.len();
        if match ident.name.as_str() {
            "WeakSet" | "WeakMap" if ctx.is_global_reference(ident) => match len {
                0 => true,
                1 => match e.arguments[0].as_expression() {
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
                    let Some(arg) = e.arguments[0].as_expression() else { return false };
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
                1 => match e.arguments[0].as_expression() {
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
        test_same("baz.map((v) => [v])");
    }

    #[test]
    fn test_array_literal_containing_spread() {
        test_same("([...c])");
        // FIXME
        test_same("([4, ...c, a])");
        test_same("var a; ([4, ...c, a])");
        test_same("([foo(), ...c, bar()])");
        test_same("([...a, b, ...c])");
        test_same("var b; ([...a, b, ...c])");
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
    #[ignore]
    fn test_object_literal() {
        test("({})", "");
        test("({a:1})", "");
        test("({a:foo()})", "foo()");
        test("({'a':foo()})", "foo()");
        // Object-spread may trigger getters.
        test_same("({...a})");
        test_same("({...foo()})");

        test("({ [bar()]: foo() })", "bar(), foo()");
        test_same("({ ...baz, [bar()]: foo() })");
    }
}
