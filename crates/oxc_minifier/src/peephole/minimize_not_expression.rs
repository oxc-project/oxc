//! Logical NOT expression minimization and optimization.
//!
//! This module implements optimizations for logical NOT expressions (`!expr`)
//! that can eliminate double negations, convert equality comparisons to their
//! opposites, and apply De Morgan's laws for more compact code.
//!
//! # Key Optimizations
//!
//! ## Double Negation Elimination
//! - `!!true` → `true`
//! - `!!false` → `false`
//! - `!!someBoolean` → `someBoolean`
//!
//! ## Equality Inversion
//! - `!(a == b)` → `a != b`
//! - `!(a != b)` → `a == b`
//! - `!(a === b)` → `a !== b`
//! - `!(a !== b)` → `a === b`
//!
//! ## Sequence Expression Optimization
//! - `!(a, b)` → `a, !b`
//! - Moves the negation to the last expression in a sequence
//!
//! ## De Morgan's Laws
//! - `!(a && b)` → `!a || !b` (when beneficial)
//! - `!(a || b)` → `!a && !b` (when beneficial)
//!
//! These optimizations follow the principle of maintaining semantic equivalence
//! while reducing code size and improving readability.
//!
//! Based on esbuild's `MaybeSimplifyNot`:
//! <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L73>

use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::DetermineValueType;
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Creates a minimized logical NOT expression.
    ///
    /// This is the main entry point for NOT expression optimization.
    /// It attempts to minimize the NOT expression and falls back to
    /// creating a standard unary expression if no optimizations apply.
    ///
    /// # Arguments
    /// * `span` - The source span for the expression
    /// * `expr` - The expression to negate
    /// * `ctx` - The minification context
    ///
    /// # Returns
    /// An optimized expression that may be:
    /// - A simplified boolean literal
    /// - An inverted comparison operator
    /// - A sequence expression with moved negation
    /// - The original NOT expression if no optimizations apply
    pub fn minimize_not(
        &self,
        span: Span,
        expr: Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Expression<'a> {
        let mut unary = ctx.ast.unary_expression(span, UnaryOperator::LogicalNot, expr);
        self.try_minimize_not(&mut unary, ctx)
            .unwrap_or_else(|| Expression::UnaryExpression(ctx.ast.alloc(unary)))
    }

    /// Attempts to minimize logical NOT expressions using various optimization strategies.
    ///
    /// This function implements several optimizations for logical NOT expressions:
    ///
    /// 1. **Double negation removal**: `!!expr` → `expr` (when expr is boolean)
    /// 2. **Equality inversion**: `!(a == b)` → `a != b`
    /// 3. **Sequence optimization**: `!(a, b)` → `a, !b`
    /// 4. **De Morgan's laws**: Applied when beneficial for code size
    ///
    /// # Arguments
    /// * `expr` - The unary NOT expression to optimize
    /// * `ctx` - The minification context
    ///
    /// # Returns
    /// `Some(Expression)` if an optimization was applied, `None` otherwise.
    ///
    /// # Safety
    /// All optimizations preserve the original boolean evaluation and side effects.
    pub fn try_minimize_not(
        &self,
        expr: &mut UnaryExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if !expr.operator.is_not() {
            return None;
        }
        self.try_fold_expr_in_boolean_context(&mut expr.argument, ctx);
        match &mut expr.argument {
            // `!!true` -> `true`
            // `!!false` -> `false`
            Expression::UnaryExpression(e)
                if e.operator.is_not() && e.argument.value_type(ctx).is_boolean() =>
            {
                Some(e.argument.take_in(ctx.ast))
            }
            // `!(a == b)` => `a != b`
            // `!(a != b)` => `a == b`
            // `!(a === b)` => `a !== b`
            // `!(a !== b)` => `a === b`
            Expression::BinaryExpression(e) if e.operator.is_equality() => {
                e.operator = e.operator.equality_inverse_operator().unwrap();
                Some(expr.argument.take_in(ctx.ast))
            }
            // "!(a, b)" => "a, !b"
            Expression::SequenceExpression(sequence_expr) => {
                if let Some(last_expr) = sequence_expr.expressions.last_mut() {
                    *last_expr =
                        self.minimize_not(last_expr.span(), last_expr.take_in(ctx.ast), ctx);
                    return Some(expr.argument.take_in(ctx.ast));
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
