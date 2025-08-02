//! Utility functions and extensions for minifier contexts.

use oxc_ast::ast::*;
use oxc_ecmascript::{constant_evaluation::ConstantValue, side_effects::MayHaveSideEffects};

use crate::ctx::Ctx;

/// Extension trait for providing additional utility methods on contexts.
pub trait ContextUtils<'a> {
    /// Check if an expression is semantically equivalent to undefined.
    fn is_expression_undefined(&self, expr: &Expression<'a>) -> bool;

    /// Check if an identifier reference refers to the global `undefined`.
    fn is_identifier_undefined(&self, ident: &IdentifierReference<'a>) -> bool;

    /// Check if two expressions are semantically equivalent.
    fn expressions_eq(&self, a: &Expression<'a>, b: &Expression<'a>) -> bool;

    /// Check if an expression has side effects.
    fn has_side_effects(&self, expr: &Expression<'a>) -> bool;

    /// Convert a constant value to an expression.
    fn constant_to_expression(&self, span: Span, value: ConstantValue<'a>) -> Expression<'a>;
}

impl<'a> ContextUtils<'a> for Ctx<'a, '_> {
    fn is_expression_undefined(&self, expr: &Expression<'a>) -> bool {
        match expr {
            Expression::Identifier(ident) if self.is_identifier_undefined(ident) => true,
            Expression::UnaryExpression(e) if e.operator.is_void() && e.argument.is_number() => {
                true
            }
            _ => false,
        }
    }

    fn is_identifier_undefined(&self, ident: &IdentifierReference<'a>) -> bool {
        ident.name == "undefined" && self.is_global_reference(ident)
    }

    fn expressions_eq(&self, a: &Expression<'a>, b: &Expression<'a>) -> bool {
        self.expr_eq(a, b)
    }

    fn has_side_effects(&self, expr: &Expression<'a>) -> bool {
        expr.may_have_side_effects(self)
    }

    fn constant_to_expression(&self, span: Span, value: ConstantValue<'a>) -> Expression<'a> {
        self.value_to_expr(span, value)
    }
}

/// Helper functions for common optimization patterns.
pub struct OptimizationHelpers;

impl OptimizationHelpers {
    /// Check if a statement is effectively empty and can be removed.
    pub fn is_removable_statement(stmt: &Statement<'_>) -> bool {
        matches!(stmt, Statement::EmptyStatement(_))
    }

    /// Check if an expression is a literal value.
    pub fn is_literal_expression(expr: &Expression<'_>) -> bool {
        matches!(
            expr,
            Expression::BooleanLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::BigIntLiteral(_)
                | Expression::NullLiteral(_)
        )
    }

    /// Check if an expression is a simple identifier reference.
    pub fn is_simple_identifier(expr: &Expression<'_>) -> bool {
        matches!(expr, Expression::Identifier(_))
    }
}
