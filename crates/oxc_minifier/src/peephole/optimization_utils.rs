//! Common utility functions for peephole optimizations.
//!
//! This module provides reusable helper functions that are commonly needed
//! across different optimization passes. These utilities help reduce code
//! duplication and provide consistent behavior across optimizations.

use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};

use crate::ctx::Ctx;

/// Utility functions for common optimization patterns.
pub struct OptimizationUtils;

impl OptimizationUtils {
    /// Checks if an expression evaluates to a truthy value.
    ///
    /// This function determines if an expression will be truthy when evaluated
    /// in a boolean context, which is useful for conditional optimizations.
    ///
    /// # Examples
    /// - `true`, `1`, `"hello"`, `{}`, `[]` → `Some(true)`
    /// - `false`, `0`, `""`, `null`, `undefined` → `Some(false)`
    /// - `variable`, `function()` → `None` (unknown at compile time)
    ///
    /// # Arguments
    /// * `expr` - The expression to evaluate
    /// * `ctx` - The minification context
    ///
    /// # Returns
    /// `Some(true)` if definitely truthy, `Some(false)` if definitely falsy,
    /// `None` if the truthiness cannot be determined at compile time.
    pub fn is_expression_truthy<'a>(
        expr: &Expression<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<bool> {
        match expr.evaluate_value(ctx) {
            Some(ConstantValue::Boolean(b)) => Some(b),
            Some(ConstantValue::Number(n)) => Some(n != 0.0 && !n.is_nan()),
            Some(ConstantValue::String(s)) => Some(!s.is_empty()),
            Some(ConstantValue::Null) | Some(ConstantValue::Undefined) => Some(false),
            Some(ConstantValue::BigInt(bi)) => Some(bi.to_string() != "0"),
            _ => None,
        }
    }

    /// Checks if an expression is a simple literal value.
    ///
    /// Simple literals are expressions that represent basic constant values
    /// and are safe to duplicate or move around without affecting semantics.
    ///
    /// # Arguments
    /// * `expr` - The expression to check
    ///
    /// # Returns
    /// `true` if the expression is a simple literal (number, string, boolean, null, undefined).
    pub fn is_simple_literal(expr: &Expression<'_>) -> bool {
        matches!(
            expr,
            Expression::NumericLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::NullLiteral(_)
        ) || matches!(expr, Expression::Identifier(ident) if ident.name == "undefined")
    }

    /// Checks if an expression has any side effects.
    ///
    /// This is a wrapper around the ECMAScript side effects analysis,
    /// providing a convenient interface for optimization passes.
    ///
    /// # Arguments
    /// * `expr` - The expression to analyze
    /// * `ctx` - The minification context
    ///
    /// # Returns
    /// `true` if the expression may have side effects, `false` otherwise.
    pub fn has_side_effects<'a>(expr: &Expression<'a>, ctx: &Ctx<'a, '_>) -> bool {
        expr.may_have_side_effects(ctx)
    }

    /// Checks if two expressions are structurally equivalent.
    ///
    /// This function performs a deep structural comparison to determine if
    /// two expressions represent the same computation, which is useful for
    /// optimizations like common subexpression elimination.
    ///
    /// # Important Notes
    /// - This only checks structural equivalence, not semantic equivalence
    /// - Side effects are not considered in the comparison
    /// - Variable references are compared by name only
    ///
    /// # Arguments
    /// * `a` - First expression to compare
    /// * `b` - Second expression to compare
    ///
    /// # Returns
    /// `true` if the expressions are structurally equivalent.
    pub fn expressions_structurally_equal(a: &Expression<'_>, b: &Expression<'_>) -> bool {
        match (a, b) {
            (Expression::NumericLiteral(a), Expression::NumericLiteral(b)) => a.value == b.value,
            (Expression::StringLiteral(a), Expression::StringLiteral(b)) => a.value == b.value,
            (Expression::BooleanLiteral(a), Expression::BooleanLiteral(b)) => a.value == b.value,
            (Expression::NullLiteral(_), Expression::NullLiteral(_)) => true,
            (Expression::Identifier(a), Expression::Identifier(b)) => a.name == b.name,
            (Expression::BinaryExpression(a), Expression::BinaryExpression(b)) => {
                a.operator == b.operator
                    && Self::expressions_structurally_equal(&a.left, &b.left)
                    && Self::expressions_structurally_equal(&a.right, &b.right)
            }
            (Expression::UnaryExpression(a), Expression::UnaryExpression(b)) => {
                a.operator == b.operator
                    && Self::expressions_structurally_equal(&a.argument, &b.argument)
            }
            // Add more cases as needed for specific optimizations
            _ => false,
        }
    }

    /// Creates a boolean literal expression.
    ///
    /// Convenience function for creating boolean literals in optimizations.
    ///
    /// # Arguments
    /// * `value` - The boolean value
    /// * `span` - The source span for the literal
    /// * `ctx` - The minification context with AST factory
    ///
    /// # Returns
    /// A boolean literal expression.
    pub fn create_boolean_literal<'a>(
        value: bool,
        span: oxc_span::Span,
        ctx: &mut Ctx<'a, '_>,
    ) -> Expression<'a> {
        Expression::BooleanLiteral(ctx.ast.alloc(BooleanLiteral { span, value }))
    }

    /// Creates a numeric literal expression.
    ///
    /// Convenience function for creating numeric literals in optimizations.
    ///
    /// # Arguments
    /// * `value` - The numeric value
    /// * `span` - The source span for the literal
    /// * `ctx` - The minification context with AST factory
    ///
    /// # Returns
    /// A numeric literal expression.
    pub fn create_numeric_literal<'a>(
        value: f64,
        span: oxc_span::Span,
        ctx: &mut Ctx<'a, '_>,
    ) -> Expression<'a> {
        Expression::NumericLiteral(ctx.ast.alloc(NumericLiteral {
            span,
            value,
            raw: None,
            base: oxc_syntax::number::NumberBase::Decimal,
        }))
    }

    /// Determines if a binary operator is associative.
    ///
    /// Associative operators can be reordered and regrouped without changing
    /// the result, which enables certain optimizations.
    ///
    /// # Arguments
    /// * `op` - The binary operator to check
    ///
    /// # Returns
    /// `true` if the operator is associative.
    pub fn is_associative_operator(op: BinaryOperator) -> bool {
        matches!(
            op,
            BinaryOperator::Addition
                | BinaryOperator::Multiplication
                | BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseOR
                | BinaryOperator::BitwiseXOR
        )
    }

    /// Determines if a binary operator is commutative.
    ///
    /// Commutative operators can have their operands swapped without changing
    /// the result, which enables certain reordering optimizations.
    ///
    /// # Arguments
    /// * `op` - The binary operator to check
    ///
    /// # Returns
    /// `true` if the operator is commutative.
    pub fn is_commutative_operator(op: BinaryOperator) -> bool {
        matches!(
            op,
            BinaryOperator::Addition
                | BinaryOperator::Multiplication
                | BinaryOperator::Equality
                | BinaryOperator::StrictEquality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictInequality
                | BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseOR
                | BinaryOperator::BitwiseXOR
        )
    }

    /// Checks if an expression is likely to be smaller when negated.
    ///
    /// This is used to determine whether to apply De Morgan's laws
    /// or other boolean transformations that involve negation.
    ///
    /// # Examples
    /// - `!a` is smaller than `!(!!a)`
    /// - `a != b` might be smaller than `!(a == b)`
    ///
    /// # Arguments
    /// * `expr` - The expression to analyze
    ///
    /// # Returns
    /// `true` if negating the expression likely results in smaller code.
    pub fn prefer_negated_form(expr: &Expression<'_>) -> bool {
        match expr {
            // !!x -> x when x is boolean context
            Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
                matches!(unary.argument, Expression::UnaryExpression(_))
            }
            // !(a == b) -> a != b
            Expression::BinaryExpression(binary) => matches!(
                binary.operator,
                BinaryOperator::Equality | BinaryOperator::StrictEquality
            ),
            _ => false,
        }
    }
}