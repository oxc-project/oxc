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
    Lowest,
    Comma,
    Spread,
    Yield,
    Assign,
    Conditional,
    NullishCoalescing,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equals,
    Compare,
    Shift,
    Add,
    Multiply,
    Exponentiation,
    Prefix,
    Postfix,
    New,
    Call,
    Member,
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
