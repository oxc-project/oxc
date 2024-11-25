#![allow(missing_docs)] // fixme
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
///
/// The values are the same as
/// [esbuild](https://github.com/evanw/esbuild/blob/78f89e41d5e8a7088f4820351c6305cc339f8820/internal/js_ast/js_ast.go#L28)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Precedence {
    Lowest = 0,
    Comma = 1,
    Spread = 2,
    Yield = 3,
    Assign = 4,
    Conditional = 5,
    NullishCoalescing = 6,
    LogicalOr = 7,
    LogicalAnd = 8,
    BitwiseOr = 9,
    BitwiseXor = 10,
    BitwiseAnd = 11,
    Equals = 12,
    Compare = 13,
    Shift = 14,
    Add = 15,
    Multiply = 16,
    Exponentiation = 17,
    Prefix = 18,
    Postfix = 19,
    New = 20,
    Call = 21,
    Member = 22,
}

impl Precedence {
    pub fn is_right_associative(&self) -> bool {
        matches!(self, Self::Exponentiation | Self::Conditional | Self::Assign)
    }

    pub fn is_left_associative(&self) -> bool {
        matches!(
            self,
            Self::Lowest
                | Self::Comma
                | Self::Spread
                | Self::Yield
                | Self::NullishCoalescing
                | Self::LogicalOr
                | Self::LogicalAnd
                | Self::BitwiseOr
                | Self::BitwiseXor
                | Self::BitwiseAnd
                | Self::Equals
                | Self::Compare
                | Self::Shift
                | Self::Add
                | Self::Multiply
                | Self::Prefix
                | Self::Postfix
                | Self::New
                | Self::Call
                | Self::Member
        )
    }
}
