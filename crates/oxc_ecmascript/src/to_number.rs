use num_traits::Zero;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::operator::UnaryOperator;

use crate::ToBoolean;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberValue {
    Number(f64),
    PositiveInfinity,
    NegativeInfinity,
    NaN,
}

impl NumberValue {
    pub fn is_nan(&self) -> bool {
        matches!(self, Self::NaN)
    }
}

impl Zero for NumberValue {
    fn zero() -> Self {
        Self::Number(0.0)
    }

    fn is_zero(&self) -> bool {
        matches!(self, Self::Number(num) if num.is_zero())
    }
}

impl std::ops::Add<Self> for NumberValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Self::Number(num) => match other {
                Self::Number(other_num) => Self::Number(num + other_num),
                Self::PositiveInfinity => Self::PositiveInfinity,
                Self::NegativeInfinity => Self::NegativeInfinity,
                Self::NaN => Self::NaN,
            },
            Self::NaN => Self::NaN,
            Self::PositiveInfinity => match other {
                Self::NaN | Self::NegativeInfinity => Self::NaN,
                _ => Self::PositiveInfinity,
            },
            Self::NegativeInfinity => match other {
                Self::NaN | Self::PositiveInfinity => Self::NaN,
                _ => Self::NegativeInfinity,
            },
        }
    }
}

impl std::ops::Sub<Self> for NumberValue {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

impl std::ops::Mul<Self> for NumberValue {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match self {
            Self::Number(num) => match other {
                Self::Number(other_num) => Self::Number(num * other_num),
                Self::PositiveInfinity | Self::NegativeInfinity if num.is_zero() => Self::NaN,
                Self::PositiveInfinity => Self::PositiveInfinity,
                Self::NegativeInfinity => Self::NegativeInfinity,
                Self::NaN => Self::NaN,
            },
            Self::NaN => Self::NaN,
            Self::PositiveInfinity | Self::NegativeInfinity => match other {
                Self::Number(num) if num > 0.0 => self,
                Self::Number(num) if num < 0.0 => -self,
                Self::PositiveInfinity => self,
                Self::NegativeInfinity => -self,
                _ => Self::NaN,
            },
        }
    }
}

impl std::ops::Div<Self> for NumberValue {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match self {
            Self::Number(num) => match other {
                Self::Number(other_num) if other_num.is_zero() => Self::NaN,
                Self::Number(other_num) => Self::Number(num / other_num),
                Self::PositiveInfinity | Self::NegativeInfinity if num < 0.0 => -other,
                Self::PositiveInfinity | Self::NegativeInfinity if num > 0.0 => other,
                _ => Self::NaN,
            },
            Self::NaN => Self::NaN,
            Self::PositiveInfinity | Self::NegativeInfinity => match other {
                Self::Number(num) if num > 0.0 => self,
                Self::Number(num) if num < 0.0 => -self,
                _ => Self::NaN,
            },
        }
    }
}

impl std::ops::Rem<Self> for NumberValue {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        match self {
            Self::Number(num) => match other {
                Self::Number(other_num) if other_num.is_zero() => Self::NaN,
                Self::Number(other_num) => Self::Number(num % other_num),
                Self::PositiveInfinity | Self::NegativeInfinity if num.is_zero() => Self::NaN,
                Self::PositiveInfinity | Self::NegativeInfinity => self,
                Self::NaN => Self::NaN,
            },
            Self::NaN => Self::NaN,
            Self::PositiveInfinity | Self::NegativeInfinity => match other {
                Self::Number(num) if !num.is_zero() => self,
                _ => Self::NaN,
            },
        }
    }
}

impl std::ops::Neg for NumberValue {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Self::Number(num) => Self::Number(-num),
            Self::PositiveInfinity => Self::NegativeInfinity,
            Self::NegativeInfinity => Self::PositiveInfinity,
            Self::NaN => Self::NaN,
        }
    }
}

impl TryFrom<NumberValue> for f64 {
    type Error = ();

    fn try_from(value: NumberValue) -> Result<Self, Self::Error> {
        match value {
            NumberValue::Number(num) => Ok(num),
            NumberValue::PositiveInfinity => Ok(Self::INFINITY),
            NumberValue::NegativeInfinity => Ok(Self::NEG_INFINITY),
            NumberValue::NaN => Err(()),
        }
    }
}

/// `ToNumber`
///
/// <https://tc39.es/ecma262/#sec-tonumber>
pub trait ToNumber<'a> {
    fn to_number(&self) -> Option<NumberValue>;
}

impl<'a> ToNumber<'a> for Expression<'a> {
    fn to_number(&self) -> Option<NumberValue> {
        match self {
            Expression::NumericLiteral(number_literal) => {
                Some(NumberValue::Number(number_literal.value))
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::UnaryPlus => unary_expr.argument.to_number(),
                UnaryOperator::UnaryNegation => unary_expr.argument.to_number().map(|v| -v),
                UnaryOperator::BitwiseNot => {
                    unary_expr.argument.to_number().map(|value| {
                        match value {
                            NumberValue::Number(num) => NumberValue::Number(f64::from(
                                !NumericLiteral::ecmascript_to_int32(num),
                            )),
                            // ~Infinity -> -1
                            // ~-Infinity -> -1
                            // ~NaN -> -1
                            _ => NumberValue::Number(-1_f64),
                        }
                    })
                }
                UnaryOperator::LogicalNot => self
                    .to_boolean()
                    .map(|tri| if tri { 1_f64 } else { 0_f64 })
                    .map(NumberValue::Number),
                UnaryOperator::Void => Some(NumberValue::NaN),
                _ => None,
            },
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(NumberValue::Number(1.0))
                } else {
                    Some(NumberValue::Number(0.0))
                }
            }
            Expression::NullLiteral(_) => Some(NumberValue::Number(0.0)),
            Expression::Identifier(ident) => match ident.name.as_str() {
                "Infinity" => Some(NumberValue::PositiveInfinity),
                "NaN" | "undefined" => Some(NumberValue::NaN),
                _ => None,
            },
            // TODO: will be implemented in next PR, just for test pass now.
            Expression::StringLiteral(string_literal) => string_literal
                .value
                .parse::<f64>()
                .map_or(Some(NumberValue::NaN), |num| Some(NumberValue::Number(num))),
            _ => None,
        }
    }
}
