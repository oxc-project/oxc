#[derive(PartialEq)]
pub enum NumberValue {
    Number(f64),
    PositiveInfinity,
    NegativeInfinity,
    NaN,
}

impl NumberValue {
    #[must_use]
    pub fn not(&self) -> Self {
        match self {
            Self::Number(num) => Self::Number(-num),
            Self::PositiveInfinity => Self::NegativeInfinity,
            Self::NegativeInfinity => Self::PositiveInfinity,
            Self::NaN => Self::NaN,
        }
    }

    pub fn is_nan(&self) -> bool {
        matches!(self, Self::NaN)
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
