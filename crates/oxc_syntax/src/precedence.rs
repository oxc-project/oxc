/// [Operator Precedence](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Operator_Precedence#table)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Precedence {
    Comma = 0,
    // Yield = 1,
    // Assignment = 2,
    // Conditional = 3,
    Coalesce = 4,
    LogicalOr = 5,
    LogicalAnd = 6,
    BitwiseOr = 7,
    BitwiseXor = 8,
    BitwiseAnd = 9,
    Equality = 10,
    Relational = 11,
    Shift = 12,
    Additive = 13,
    Multiplicative = 14,
    Exponential = 15,
    // Unary = 16,
    // Update = 17,
    // LeftHandSide = 18,
    // Member = 19,
    // Primary = 20,
}

impl Precedence {
    pub fn lowest() -> Self {
        Self::Comma
    }

    pub fn is_right_associative(right: Self) -> bool {
        right == Self::Exponential
    }
}
