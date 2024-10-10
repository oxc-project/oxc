use num_traits::Zero;

#[derive(PartialEq)]
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
