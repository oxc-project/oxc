use oxc_allocator::CloneIn;
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{cmp::ContentEq, hash::ContentHash};

use crate::precedence::{GetPrecedence, Precedence};

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum AssignmentOperator {
    #[estree(rename = "=")]
    Assign = 0,
    #[estree(rename = "+=")]
    Addition = 1,
    #[estree(rename = "-=")]
    Subtraction = 2,
    #[estree(rename = "*=")]
    Multiplication = 3,
    #[estree(rename = "/=")]
    Division = 4,
    #[estree(rename = "%=")]
    Remainder = 5,
    #[estree(rename = "<<=")]
    ShiftLeft = 6,
    #[estree(rename = ">>=")]
    ShiftRight = 7,
    #[estree(rename = ">>>=")]
    ShiftRightZeroFill = 8,
    #[estree(rename = "|=")]
    BitwiseOR = 9,
    #[estree(rename = "^=")]
    BitwiseXOR = 10,
    #[estree(rename = "&=")]
    BitwiseAnd = 11,
    #[estree(rename = "&&=")]
    LogicalAnd = 12,
    #[estree(rename = "||=")]
    LogicalOr = 13,
    #[estree(rename = "??=")]
    LogicalNullish = 14,
    #[estree(rename = "**=")]
    Exponential = 15,
}

impl AssignmentOperator {
    pub fn is_logical(self) -> bool {
        matches!(self, Self::LogicalAnd | Self::LogicalOr | Self::LogicalNullish)
    }

    #[rustfmt::skip]
    pub fn is_arithmetic(self) -> bool {
        matches!(self, Self::Addition | Self::Subtraction | Self::Multiplication
                | Self::Division | Self::Remainder | Self::Exponential
        )
    }

    #[rustfmt::skip]
    pub fn is_bitwise(self) -> bool {
        matches!(self, Self::BitwiseOR | Self::BitwiseXOR | Self::BitwiseAnd
                | Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Addition => "+=",
            Self::Subtraction => "-=",
            Self::Multiplication => "*=",
            Self::Division => "/=",
            Self::Remainder => "%=",
            Self::ShiftLeft => "<<=",
            Self::ShiftRight => ">>=",
            Self::ShiftRightZeroFill => ">>>=",
            Self::BitwiseOR => "|=",
            Self::BitwiseXOR => "^=",
            Self::BitwiseAnd => "&=",
            Self::LogicalAnd => "&&=",
            Self::LogicalOr => "||=",
            Self::LogicalNullish => "??=",
            Self::Exponential => "**=",
        }
    }
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum BinaryOperator {
    #[estree(rename = "==")]
    Equality = 0,
    #[estree(rename = "!=")]
    Inequality = 1,
    #[estree(rename = "===")]
    StrictEquality = 2,
    #[estree(rename = "!==")]
    StrictInequality = 3,
    #[estree(rename = "<")]
    LessThan = 4,
    #[estree(rename = "<=")]
    LessEqualThan = 5,
    #[estree(rename = ">")]
    GreaterThan = 6,
    #[estree(rename = ">=")]
    GreaterEqualThan = 7,
    #[estree(rename = "<<")]
    ShiftLeft = 8,
    #[estree(rename = ">>")]
    ShiftRight = 9,
    #[estree(rename = ">>>")]
    ShiftRightZeroFill = 10,
    #[estree(rename = "+")]
    Addition = 11,
    #[estree(rename = "-")]
    Subtraction = 12,
    #[estree(rename = "*")]
    Multiplication = 13,
    #[estree(rename = "/")]
    Division = 14,
    #[estree(rename = "%")]
    Remainder = 15,
    #[estree(rename = "|")]
    BitwiseOR = 16,
    #[estree(rename = "^")]
    BitwiseXOR = 17,
    #[estree(rename = "&")]
    BitwiseAnd = 18,
    #[estree(rename = "in")]
    In = 19,
    #[estree(rename = "instanceof")]
    Instanceof = 20,
    #[estree(rename = "**")]
    Exponential = 21,
}

impl BinaryOperator {
    #[rustfmt::skip]
    pub fn is_equality(self) -> bool {
        matches!(self, Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality)
    }

    #[rustfmt::skip]
    pub fn is_compare(self) -> bool {
        matches!(self, Self::LessThan | Self::LessEqualThan | Self::GreaterThan | Self::GreaterEqualThan)
    }

    #[rustfmt::skip]
    pub fn is_arithmetic(self) -> bool {
        matches!(self, Self::Addition | Self::Subtraction | Self::Multiplication
                | Self::Division | Self::Remainder | Self::Exponential)
    }

    pub fn is_multiplicative(self) -> bool {
        matches!(self, Self::Multiplication | Self::Division | Self::Remainder)
    }

    pub fn is_relational(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    pub fn is_in(self) -> bool {
        matches!(self, Self::In)
    }

    #[rustfmt::skip]
    pub fn is_bitwise(self) -> bool {
        self.is_bitshift() || matches!(self, Self::BitwiseOR | Self::BitwiseXOR | Self::BitwiseAnd)
    }

    pub fn is_bitshift(self) -> bool {
        matches!(self, Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill)
    }

    pub fn is_numeric_or_string_binary_operator(self) -> bool {
        self.is_arithmetic() || self.is_bitwise()
    }

    pub fn is_keyword(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    pub fn compare_inverse_operator(self) -> Option<Self> {
        match self {
            Self::LessThan => Some(Self::GreaterThan),
            Self::LessEqualThan => Some(Self::GreaterEqualThan),
            Self::GreaterThan => Some(Self::LessThan),
            Self::GreaterEqualThan => Some(Self::LessEqualThan),
            _ => None,
        }
    }

    pub fn equality_inverse_operator(self) -> Option<Self> {
        match self {
            Self::Equality => Some(Self::Inequality),
            Self::Inequality => Some(Self::Equality),
            Self::StrictEquality => Some(Self::StrictInequality),
            Self::StrictInequality => Some(Self::StrictEquality),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equality => "==",
            Self::Inequality => "!=",
            Self::StrictEquality => "===",
            Self::StrictInequality => "!==",
            Self::LessThan => "<",
            Self::LessEqualThan => "<=",
            Self::GreaterThan => ">",
            Self::GreaterEqualThan => ">=",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::ShiftRightZeroFill => ">>>",
            Self::Addition => "+",
            Self::Subtraction => "-",
            Self::Multiplication => "*",
            Self::Division => "/",
            Self::Remainder => "%",
            Self::BitwiseOR => "|",
            Self::BitwiseXOR => "^",
            Self::BitwiseAnd => "&",
            Self::In => "in",
            Self::Instanceof => "instanceof",
            Self::Exponential => "**",
        }
    }

    pub fn lower_precedence(&self) -> Precedence {
        match self {
            Self::BitwiseOR => Precedence::LogicalAnd,
            Self::BitwiseXOR => Precedence::BitwiseOr,
            Self::BitwiseAnd => Precedence::BitwiseXor,
            Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality => {
                Precedence::BitwiseAnd
            }
            Self::LessThan
            | Self::LessEqualThan
            | Self::GreaterThan
            | Self::GreaterEqualThan
            | Self::Instanceof
            | Self::In => Precedence::Equals,
            Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill => Precedence::Compare,
            Self::Addition | Self::Subtraction => Precedence::Shift,
            Self::Multiplication | Self::Remainder | Self::Division => Precedence::Add,
            Self::Exponential => Precedence::Multiply,
        }
    }
}

impl GetPrecedence for BinaryOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::BitwiseOR => Precedence::BitwiseOr,
            Self::BitwiseXOR => Precedence::BitwiseXor,
            Self::BitwiseAnd => Precedence::BitwiseAnd,
            Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality => {
                Precedence::Equals
            }
            Self::LessThan
            | Self::LessEqualThan
            | Self::GreaterThan
            | Self::GreaterEqualThan
            | Self::Instanceof
            | Self::In => Precedence::Compare,
            Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill => Precedence::Shift,
            Self::Subtraction | Self::Addition => Precedence::Add,
            Self::Multiplication | Self::Remainder | Self::Division => Precedence::Multiply,
            Self::Exponential => Precedence::Exponentiation,
        }
    }
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum LogicalOperator {
    #[estree(rename = "||")]
    Or = 0,
    #[estree(rename = "&&")]
    And = 1,
    #[estree(rename = "??")]
    Coalesce = 2,
}

impl LogicalOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Or => "||",
            Self::And => "&&",
            Self::Coalesce => "??",
        }
    }

    pub fn lower_precedence(&self) -> Precedence {
        match self {
            Self::Or => Precedence::NullishCoalescing,
            Self::And => Precedence::LogicalOr,
            Self::Coalesce => Precedence::Conditional,
        }
    }
}

impl GetPrecedence for LogicalOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Or => Precedence::LogicalOr,
            Self::And => Precedence::LogicalAnd,
            Self::Coalesce => Precedence::NullishCoalescing,
        }
    }
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum UnaryOperator {
    #[estree(rename = "-")]
    UnaryNegation = 0,
    #[estree(rename = "+")]
    UnaryPlus = 1,
    #[estree(rename = "!")]
    LogicalNot = 2,
    #[estree(rename = "~")]
    BitwiseNot = 3,
    #[estree(rename = "typeof")]
    Typeof = 4,
    #[estree(rename = "void")]
    Void = 5,
    #[estree(rename = "delete")]
    Delete = 6,
}

impl UnaryOperator {
    pub fn is_arithmetic(self) -> bool {
        matches!(self, Self::UnaryNegation | Self::UnaryPlus)
    }

    /// Returns `true` if this operator is a [`LogicalNot`].
    ///
    /// [`LogicalNot`]: UnaryOperator::LogicalNot
    pub fn is_not(self) -> bool {
        matches!(self, Self::LogicalNot)
    }

    pub fn is_bitwise(self) -> bool {
        matches!(self, Self::BitwiseNot)
    }

    pub fn is_void(self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn is_keyword(self) -> bool {
        matches!(self, Self::Typeof | Self::Void | Self::Delete)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnaryNegation => "-",
            Self::UnaryPlus => "+",
            Self::LogicalNot => "!",
            Self::BitwiseNot => "~",
            Self::Typeof => "typeof",
            Self::Void => "void",
            Self::Delete => "delete",
        }
    }
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum UpdateOperator {
    #[estree(rename = "++")]
    Increment = 0,
    #[estree(rename = "--")]
    Decrement = 1,
}

impl UpdateOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }
}
