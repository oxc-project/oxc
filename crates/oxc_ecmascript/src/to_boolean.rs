#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{NumberValue, ToNumber};

/// `ToBoolean`
///
/// <https://tc39.es/ecma262/#sec-toboolean>
pub trait ToBoolean<'a> {
    fn to_boolean(&self) -> Option<bool>;
}

impl<'a> ToBoolean<'a> for Expression<'a> {
    fn to_boolean(&self) -> Option<bool> {
        match self {
            Expression::RegExpLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::NewExpression(_)
            | Expression::ObjectExpression(_) => Some(true),
            Expression::NullLiteral(_) => Some(false),
            Expression::BooleanLiteral(boolean_literal) => Some(boolean_literal.value),
            Expression::NumericLiteral(number_literal) => Some(number_literal.value != 0.0),
            Expression::BigIntLiteral(big_int_literal) => Some(!big_int_literal.is_zero()),
            Expression::StringLiteral(string_literal) => Some(!string_literal.value.is_empty()),
            Expression::TemplateLiteral(template_literal) => {
                // only for ``
                template_literal
                    .quasis
                    .first()
                    .filter(|quasi| quasi.tail)
                    .and_then(|quasi| quasi.value.cooked.as_ref())
                    .map(|cooked| !cooked.is_empty())
            }
            Expression::Identifier(ident) => match ident.name.as_str() {
                "NaN" | "undefined" => Some(false),
                "Infinity" => Some(true),
                _ => None,
            },
            Expression::AssignmentExpression(assign_expr) => {
                match assign_expr.operator {
                    AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr => None,
                    // For ASSIGN, the value is the value of the RHS.
                    _ => assign_expr.right.to_boolean(),
                }
            }
            Expression::LogicalExpression(logical_expr) => {
                match logical_expr.operator {
                    // true && true -> true
                    // true && false -> false
                    // a && true -> None
                    LogicalOperator::And => {
                        let left = logical_expr.left.to_boolean();
                        let right = logical_expr.right.to_boolean();
                        match (left, right) {
                            (Some(true), Some(true)) => Some(true),
                            (Some(false), _) | (_, Some(false)) => Some(false),
                            (None, _) | (_, None) => None,
                        }
                    }
                    // true || false -> true
                    // false || false -> false
                    // a || b -> None
                    LogicalOperator::Or => {
                        let left = logical_expr.left.to_boolean();
                        let right = logical_expr.right.to_boolean();

                        match (left, right) {
                            (Some(true), _) | (_, Some(true)) => Some(true),
                            (Some(false), Some(false)) => Some(false),
                            (None, _) | (_, None) => None,
                        }
                    }
                    LogicalOperator::Coalesce => None,
                }
            }
            Expression::SequenceExpression(sequence_expr) => {
                // For sequence expression, the value is the value of the RHS.
                sequence_expr.expressions.last().and_then(ToBoolean::to_boolean)
            }
            Expression::UnaryExpression(unary_expr) => {
                if unary_expr.operator == UnaryOperator::Void {
                    Some(false)
                } else if matches!(
                    unary_expr.operator,
                    UnaryOperator::BitwiseNot
                        | UnaryOperator::UnaryPlus
                        | UnaryOperator::UnaryNegation
                ) {
                    // ~0 -> true
                    // +1 -> true
                    // +0 -> false
                    // -0 -> false
                    self.to_number().map(|value| value != NumberValue::Number(0_f64))
                } else if unary_expr.operator == UnaryOperator::LogicalNot {
                    // !true -> false
                    unary_expr.argument.to_boolean().map(|b| !b)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
