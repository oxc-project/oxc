pub trait GetPrecedence {
    fn precedence(&self) -> Precedence;
}

/// Operator Precedence
///
/// The following values are meaningful relative position, not their individual values.
/// The relative positions are derived from the ECMA Spec by following the grammar bottom up, starting from the "Comma Operator".
///
/// Note: This differs from the the operator precedence table
/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Operator_Precedence#table>
/// but the relative positions are the same, as both are derived from the ECMA specification.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Precedence {
    Comma,
    Assign,
    Arrow,
    Yield,
    Conditional,
    Coalesce,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equality,
    Relational,
    Shift,
    Add,
    Multiply,
    Exponential,
    Prefix,
    Postfix,
    NewWithoutArgs,
    Call,
    Member,
    Grouping,
}

impl Precedence {
    pub fn lowest() -> Self {
        Self::Comma
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Self::Exponential)
    }
}
