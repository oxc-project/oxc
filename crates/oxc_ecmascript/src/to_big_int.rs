use num_bigint::BigInt;
use num_traits::{Num, One, Zero};

use oxc_ast::ast::{BigIntLiteral, Expression};
use oxc_syntax::operator::UnaryOperator;

use crate::{GlobalContext, StringToBigInt, ToBoolean, ToJsString};

/// `ToBigInt`
///
/// <https://tc39.es/ecma262/#sec-tobigint>
pub trait ToBigInt<'a> {
    fn to_big_int(&self, ctx: &impl GlobalContext<'a>) -> Option<BigInt>;
}

impl<'a> ToBigInt<'a> for Expression<'a> {
    #[expect(clippy::cast_possible_truncation)]
    fn to_big_int(&self, ctx: &impl GlobalContext<'a>) -> Option<BigInt> {
        match self {
            Expression::NumericLiteral(number_literal) => {
                let value = number_literal.value;
                if value.abs() < 2_f64.powi(53) && value.fract() == 0.0 {
                    Some(BigInt::from(value as i64))
                } else {
                    None
                }
            }
            Expression::BigIntLiteral(lit) => lit.to_big_int(ctx),
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(BigInt::one())
                } else {
                    Some(BigInt::zero())
                }
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::LogicalNot => self
                    .to_boolean(ctx)
                    .map(|boolean| if boolean { BigInt::one() } else { BigInt::zero() }),
                UnaryOperator::UnaryNegation => {
                    unary_expr.argument.to_big_int(ctx).map(std::ops::Neg::neg)
                }
                UnaryOperator::BitwiseNot => {
                    unary_expr.argument.to_big_int(ctx).map(std::ops::Not::not)
                }
                UnaryOperator::UnaryPlus => unary_expr.argument.to_big_int(ctx),
                _ => None,
            },
            Expression::StringLiteral(string_literal) => {
                string_literal.value.as_str().string_to_big_int()
            }
            Expression::TemplateLiteral(_) => {
                self.to_js_string(ctx).and_then(|value| value.as_ref().string_to_big_int())
            }
            _ => None,
        }
    }
}

impl<'a> ToBigInt<'a> for BigIntLiteral<'a> {
    fn to_big_int(&self, _ctx: &impl GlobalContext<'a>) -> Option<BigInt> {
        // No need to use `StringToBigInt::string_to_big_int`, because `value` is always base 10
        let value = BigInt::from_str_radix(&self.value, 10).ok();
        debug_assert!(value.is_some(), "Failed to parse {}n", self.value);
        value
    }
}
