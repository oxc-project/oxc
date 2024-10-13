#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::operator::UnaryOperator;

use crate::ToBoolean;

/// `ToNumber`
///
/// <https://tc39.es/ecma262/#sec-tonumber>
pub trait ToNumber<'a> {
    fn to_number(&self) -> Option<f64>;
}

impl<'a> ToNumber<'a> for Expression<'a> {
    fn to_number(&self) -> Option<f64> {
        match self {
            Expression::NumericLiteral(number_literal) => Some(number_literal.value),
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::UnaryPlus => unary_expr.argument.to_number(),
                UnaryOperator::UnaryNegation => unary_expr.argument.to_number().map(|v| -v),
                // UnaryOperator::BitwiseNot => {
                // unary_expr.argument.to_number().map(|value| {
                // match value {
                // NumberValue::Number(num) => NumberValue::Number(f64::from(
                // !NumericLiteral::ecmascript_to_int32(num),
                // )),
                // // ~Infinity -> -1
                // // ~-Infinity -> -1
                // // ~NaN -> -1
                // _ => NumberValue::Number(-1_f64),
                // }
                // })
                // }
                UnaryOperator::LogicalNot => {
                    self.to_boolean().map(|tri| if tri { 1_f64 } else { 0_f64 })
                }
                UnaryOperator::Void => Some(f64::NAN),
                _ => None,
            },
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(1.0)
                } else {
                    Some(0.0)
                }
            }
            Expression::NullLiteral(_) => Some(0.0),
            Expression::Identifier(ident) => match ident.name.as_str() {
                "Infinity" => Some(f64::INFINITY),
                "NaN" | "undefined" => Some(f64::NAN),
                _ => None,
            },
            // TODO: will be implemented in next PR, just for test pass now.
            Expression::StringLiteral(string_literal) => {
                string_literal.value.parse::<f64>().map_or(Some(f64::NAN), Some)
            }
            _ => None,
        }
    }
}
