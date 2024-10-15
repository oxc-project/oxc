#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

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
            Expression::StringLiteral(lit) => {
                use crate::StringToNumber;
                Some(lit.value.as_str().string_to_number())
            }
            _ => None,
        }
    }
}
