use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::DetermineValueType;
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn minimize_not(span: Span, expr: Expression<'a>, ctx: &mut Ctx<'a, '_>) -> Expression<'a> {
        let mut unary = ctx.ast.expression_unary(span, UnaryOperator::LogicalNot, expr);
        Self::try_minimize_not(&mut unary, ctx);
        unary
    }

    /// `MaybeSimplifyNot`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L73>
    pub fn try_minimize_not(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::UnaryExpression(e) = expr else { return };
        if !e.operator.is_not() {
            return;
        }
        Self::try_fold_expr_in_boolean_context(&mut e.argument, ctx);
        match &mut e.argument {
            // `!!true` -> `true`
            // `!!false` -> `false`
            Expression::UnaryExpression(e)
                if e.operator.is_not() && e.argument.value_type(ctx).is_boolean() =>
            {
                *expr = e.argument.take_in(ctx.ast);
                ctx.state.changed = true;
            }
            // `!(a == b)` => `a != b`
            // `!(a != b)` => `a == b`
            // `!(a === b)` => `a !== b`
            // `!(a !== b)` => `a === b`
            Expression::BinaryExpression(binary_expr) if binary_expr.operator.is_equality() => {
                binary_expr.operator = binary_expr.operator.equality_inverse_operator().unwrap();
                *expr = e.argument.take_in(ctx.ast);
                ctx.state.changed = true;
            }
            // "!(a, b)" => "a, !b"
            Expression::SequenceExpression(sequence_expr) => {
                if let Some(last_expr) = sequence_expr.expressions.last_mut() {
                    *last_expr =
                        Self::minimize_not(last_expr.span(), last_expr.take_in(ctx.ast), ctx);
                    *expr = e.argument.take_in(ctx.ast);
                    ctx.state.changed = true;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    #[test]
    fn minimize_duplicate_nots() {
        test("!x", "x");
        test("!!x", "x");
        test("!!!x", "x");
        test("!!!!x", "x");
        test("!!!(x && y)", "x && y");
        test("var k = () => { !!x; }", "var k = () => { x }");

        test_same("var k = !!x;");
        test_same("function k () { return !!x; }");
        test("var k = () => { return !!x; }", "var k = () => !!x");
        test_same("var k = () => !!x;");
    }

    #[test]
    fn minimize_nots_with_binary_expressions() {
        test("!(x === undefined)", "x");
        test("!(typeof(x) === 'undefined')", "");
        test("!(typeof(x()) === 'undefined')", "x()");
        test("!(x === void 0)", "x");
        test("!!delete x.y", "delete x.y");
        test("!!!delete x.y", "delete x.y");
        test("!!!!delete x.y", "delete x.y");
        test("var k = !!(foo instanceof bar)", "var k = foo instanceof bar");
        test("!(a === 1 ? void 0 : a.b)", "a !== 1 && a.b;");
        test("!(a, b)", "a, b");
    }
}
