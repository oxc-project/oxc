/// [Operator Precedence](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Operator_Precedence#table)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Precedence {
    Lowest = 0,
    Comma,
    Spread,
    Yield,
    Assign,
    Conditional,
    Coalesce,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equality,
    Compare,
    Shift,
    Add,
    Multiply,
    Exponential,
    Prefix,
    Postfix,
    New,
    Call,
    Member,
}

impl Precedence {
    pub fn lowest() -> Self {
        Self::Comma
    }

    pub fn is_right_associative(right: Self) -> bool {
        right == Self::Exponential
    }
}

pub trait GetPrecedence {
    fn precedence(&self) -> Precedence;
}
