use num_bigint::BigInt;
use num_traits::{One, Zero};

use oxc_ast::ast::{BigIntLiteral, Expression};
use oxc_syntax::operator::UnaryOperator;

use crate::{StringToBigInt, ToBoolean, ToJsString, is_global_reference::IsGlobalReference};

/// `ToBigInt`
///
/// <https://tc39.es/ecma262/#sec-tobigint>
pub trait ToBigInt<'a> {
    fn to_big_int(&self, is_global_reference: &impl IsGlobalReference) -> Option<BigInt>;
}

impl<'a> ToBigInt<'a> for Expression<'a> {
    #[expect(clippy::cast_possible_truncation)]
    fn to_big_int(&self, is_global_reference: &impl IsGlobalReference) -> Option<BigInt> {
        match self {
            Expression::NumericLiteral(number_literal) => {
                let value = number_literal.value;
                if value.abs() < 2_f64.powi(53) && value.fract() == 0.0 {
                    Some(BigInt::from(value as i64))
                } else {
                    None
                }
            }
            Expression::BigIntLiteral(lit) => lit.to_big_int(is_global_reference),
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(BigInt::one())
                } else {
                    Some(BigInt::zero())
                }
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::LogicalNot => self
                    .to_boolean(is_global_reference)
                    .map(|boolean| if boolean { BigInt::one() } else { BigInt::zero() }),
                UnaryOperator::UnaryNegation => {
                    unary_expr.argument.to_big_int(is_global_reference).map(std::ops::Neg::neg)
                }
                UnaryOperator::BitwiseNot => {
                    unary_expr.argument.to_big_int(is_global_reference).map(std::ops::Not::not)
                }
                UnaryOperator::UnaryPlus => unary_expr.argument.to_big_int(is_global_reference),
                _ => None,
            },
            Expression::StringLiteral(string_literal) => {
                string_literal.value.as_str().string_to_big_int()
            }
            Expression::TemplateLiteral(_) => self
                .to_js_string(is_global_reference)
                .and_then(|value| value.as_ref().string_to_big_int()),
            _ => None,
        }
    }
}

impl<'a> ToBigInt<'a> for BigIntLiteral<'a> {
    fn to_big_int(&self, _is_global_reference: &impl IsGlobalReference) -> Option<BigInt> {
        let value = self.raw.as_str().trim_end_matches('n').string_to_big_int();
        debug_assert!(value.is_some(), "Failed to parse {}", self.raw);
        value
    }
}
