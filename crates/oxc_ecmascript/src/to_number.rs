use oxc_ast::ast::*;

use crate::is_global_reference::IsGlobalReference;

/// `ToNumber`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumber>
pub trait ToNumber<'a> {
    fn to_number(&self, is_global_reference: &impl IsGlobalReference) -> Option<f64>;
}

impl<'a> ToNumber<'a> for Expression<'a> {
    fn to_number(&self, is_global_reference: &impl IsGlobalReference) -> Option<f64> {
        match self {
            Expression::NumericLiteral(number_literal) => Some(number_literal.value),
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(1.0)
                } else {
                    Some(0.0)
                }
            }
            Expression::NullLiteral(_) => Some(0.0),
            Expression::Identifier(ident) => match ident.name.as_str() {
                "Infinity" if is_global_reference.is_global_reference(ident) == Some(true) => {
                    Some(f64::INFINITY)
                }
                "NaN" | "undefined"
                    if is_global_reference.is_global_reference(ident) == Some(true) =>
                {
                    Some(f64::NAN)
                }
                _ => None,
            },
            Expression::StringLiteral(lit) => {
                use crate::StringToNumber;
                Some(lit.value.as_str().string_to_number())
            }
            Expression::UnaryExpression(unary) if unary.operator.is_not() => {
                let number = unary.argument.to_number(is_global_reference)?;
                Some(if number == 0.0 { 1.0 } else { 0.0 })
            }
            _ => None,
        }
    }
}
