#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use phf::{phf_set, Set};

use crate::context::LintContext;

pub const STRICT_MODE_NAMES: Set<&'static str> = phf_set! {
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
};

/// Test if an AST node is a boolean value that never changes. Specifically we
/// test for:
/// 1. Literal booleans (`true` or `false`)
/// 2. Unary `!` expressions with a constant value
/// 3. Constant booleans created via the `Boolean` global function
pub fn is_static_boolean<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr {
        Expression::BooleanLiteral(_) => true,
        Expression::CallExpression(call_expr) => call_expr.is_constant(true, ctx),
        Expression::UnaryExpression(unary_expr) => {
            unary_expr.operator == UnaryOperator::LogicalNot
                && unary_expr.argument.is_constant(true, ctx)
        }
        _ => false,
    }
}

/// Checks if a branch node of `LogicalExpression` short circuits the whole condition
fn is_logical_identity(op: LogicalOperator, expr: &Expression) -> bool {
    match expr {
        expr if expr.is_literal_expression() => {
            let boolean_value = expr.get_boolean_value();
            (op == LogicalOperator::Or && boolean_value == Some(true))
                || (op == LogicalOperator::And && boolean_value == Some(false))
        }
        Expression::UnaryExpression(unary_expr) => {
            op == LogicalOperator::And && unary_expr.operator == UnaryOperator::Void
        }
        Expression::LogicalExpression(logical_expr) => {
            op == logical_expr.operator
                && (is_logical_identity(logical_expr.operator, &logical_expr.left)
                    || is_logical_identity(logical_expr.operator, &logical_expr.right))
        }
        Expression::AssignmentExpression(assign_expr) => {
            matches!(
                assign_expr.operator,
                AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr
            ) && ((op == LogicalOperator::And
                && assign_expr.operator == AssignmentOperator::LogicalAnd)
                || (op == LogicalOperator::Or
                    && assign_expr.operator == AssignmentOperator::LogicalOr))
                && is_logical_identity(op, &assign_expr.right)
        }
        Expression::ParenthesizedExpression(expr) => is_logical_identity(op, &expr.expression),
        _ => false,
    }
}

/// Checks if a  node has a constant truthiness value.
/// `inBooleanPosition`:
///   `true` if checking the test of a condition.
///   `false` in all other cases.
///   When `false`, checks if -- for both string and number --
///   if coerced to that type, the value will be constant.
pub trait IsConstant<'a, 'b> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool;
}

impl<'a, 'b> IsConstant<'a, 'b> for Expression<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        match self {
            Self::ArrowFunctionExpression(_)
            | Self::FunctionExpression(_)
            | Self::ClassExpression(_)
            | Self::ObjectExpression(_) => true,
            Self::TemplateLiteral(template) => {
                let test_quasis = in_boolean_position
                    && template.quasis.iter().any(|quasi| {
                        quasi.value.cooked.as_ref().map_or(false, |cooked| !cooked.is_empty())
                    });
                let test_expressions =
                    template.expressions.iter().all(|expr| expr.is_constant(false, ctx));
                test_quasis || test_expressions
            }
            Self::ArrayExpression(expr) => {
                if in_boolean_position {
                    return true;
                }
                expr.elements
                    .iter()
                    .all(|element| element.as_ref().map_or(true, |e| e.is_constant(false, ctx)))
            }
            Self::UnaryExpression(expr) => match expr.operator {
                UnaryOperator::Void => true,
                UnaryOperator::Typeof if in_boolean_position => true,
                UnaryOperator::LogicalNot => expr.argument.is_constant(true, ctx),
                _ => expr.argument.is_constant(false, ctx),
            },
            Self::BinaryExpression(expr) => {
                expr.operator != BinaryOperator::In
                    && expr.left.is_constant(false, ctx)
                    && expr.right.is_constant(false, ctx)
            }
            Self::LogicalExpression(expr) => {
                let is_left_constant = expr.left.is_constant(in_boolean_position, ctx);
                let is_right_constant = expr.right.is_constant(in_boolean_position, ctx);
                let is_left_short_circuit =
                    is_left_constant && is_logical_identity(expr.operator, &expr.left);
                let is_right_short_circuit = in_boolean_position
                    && is_right_constant
                    && is_logical_identity(expr.operator, &expr.right);
                (is_left_constant && is_right_constant)
                    || is_left_short_circuit
                    || is_right_short_circuit
            }
            Self::NewExpression(_) => in_boolean_position,
            Self::AssignmentExpression(expr) => match expr.operator {
                AssignmentOperator::Assign => expr.right.is_constant(in_boolean_position, ctx),
                AssignmentOperator::LogicalAnd if in_boolean_position => {
                    is_logical_identity(LogicalOperator::And, &expr.right)
                }
                AssignmentOperator::LogicalOr if in_boolean_position => {
                    is_logical_identity(LogicalOperator::Or, &expr.right)
                }
                _ => false,
            },
            Self::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .map_or(false, |last| last.is_constant(in_boolean_position, ctx)),
            Self::CallExpression(call_expr) => call_expr.is_constant(in_boolean_position, ctx),
            Self::ParenthesizedExpression(paren_expr) => {
                paren_expr.expression.is_constant(in_boolean_position, ctx)
            }
            Self::Identifier(ident) => {
                ident.name == "undefined" && ctx.is_reference_to_global_variable(ident)
            }
            _ if self.is_literal_expression() => true,
            _ => false,
        }
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for CallExpression<'a> {
    fn is_constant(&self, _in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        if let Expression::Identifier(ident) = &self.callee {
            if ident.name == "Boolean"
                && self.arguments.iter().next().map_or(true, |first| first.is_constant(true, ctx))
            {
                return ctx.is_reference_to_global_variable(ident);
            }
        }
        false
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for Argument<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_constant(in_boolean_position, ctx),
            Self::Expression(expr) => expr.is_constant(in_boolean_position, ctx),
        }
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for SpreadElement<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        self.argument.is_constant(in_boolean_position, ctx)
    }
}
