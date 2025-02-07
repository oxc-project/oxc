use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::ValueType;
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_not(span: Span, expr: Expression<'a>, ctx: Ctx<'a, '_>) -> Expression<'a> {
        let mut unary = ctx.ast.unary_expression(span, UnaryOperator::LogicalNot, expr);
        Self::try_minimize_not(&mut unary, ctx)
            .unwrap_or_else(|| Expression::UnaryExpression(ctx.ast.alloc(unary)))
    }

    /// `MaybeSimplifyNot`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L73>
    pub fn try_minimize_not(
        expr: &mut UnaryExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if !expr.operator.is_not() {
            return None;
        }
        match &mut expr.argument {
            // `!!true` -> `true`
            // `!!false` -> `false`
            Expression::UnaryExpression(e)
                if e.operator.is_not() && ValueType::from(&e.argument).is_boolean() =>
            {
                Some(ctx.ast.move_expression(&mut e.argument))
            }
            // `!(a == b)` => `a != b`
            // `!(a != b)` => `a == b`
            // `!(a === b)` => `a !== b`
            // `!(a !== b)` => `a === b`
            Expression::BinaryExpression(e) if e.operator.is_equality() => {
                e.operator = e.operator.equality_inverse_operator().unwrap();
                Some(ctx.ast.move_expression(&mut expr.argument))
            }
            // "!(a, b)" => "a, !b"
            Expression::SequenceExpression(sequence_expr) => {
                if let Some(e) = sequence_expr.expressions.pop() {
                    let e = ctx.ast.expression_unary(e.span(), UnaryOperator::LogicalNot, e);
                    let expressions = ctx.ast.vec_from_iter(
                        sequence_expr.expressions.drain(..).chain(std::iter::once(e)),
                    );
                    return Some(ctx.ast.expression_sequence(sequence_expr.span, expressions));
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    #[test]
    fn minimize_duplicate_nots() {
        // test("!x", "x"); // TODO: in ExpressionStatement
        test("!!x", "x");
        test("!!!x", "!x");
        test("!!!!x", "x");
        test("!!!(x && y)", "!(x && y)");
        test_same("var k = () => { !!x; }");

        test_same("var k = !!x;");
        test_same("function k () { return !!x; }");
        test("var k = () => { return !!x; }", "var k = () => !!x");
        test_same("var k = () => !!x;");
    }

    #[test]
    fn minimize_nots_with_binary_expressions() {
        test("!(x === undefined)", "x !== void 0");
        test("!(typeof(x) === 'undefined')", "!(typeof x > 'u')");
        test("!(x === void 0)", "x !== void 0");
        test("!!delete x.y", "delete x.y");
        test("!!!delete x.y", "!delete x.y");
        test("!!!!delete x.y", "delete x.y");
        test("var k = !!(foo instanceof bar)", "var k = foo instanceof bar");
        test_same("!(a === 1 ? void 0 : a.b)"); // FIXME: can be compressed to `a === 1 || !a.b`
        test("!(a, b)", "a, !b");
    }
}
