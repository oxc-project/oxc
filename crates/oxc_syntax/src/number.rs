#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberBase {
    Float,
    Decimal,
    Binary,
    Octal,
    Hex,
}

impl NumberBase {
    pub fn is_base_10(&self) -> bool {
        matches!(self, Self::Float | Self::Decimal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BigintBase {
    Decimal,
    Binary,
    Octal,
    Hex,
}

impl BigintBase {
    pub fn is_base_10(&self) -> bool {
        self == &Self::Decimal
    }
}
