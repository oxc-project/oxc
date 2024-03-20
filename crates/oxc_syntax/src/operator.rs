// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_macros::ast_node;
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use crate::precedence::{GetPrecedence, Precedence};

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum AssignmentOperator {
    #[cfg_attr(feature = "serialize", serde(rename = "="))]
    Assign,
    #[cfg_attr(feature = "serialize", serde(rename = "+="))]
    Addition,
    #[cfg_attr(feature = "serialize", serde(rename = "-="))]
    Subtraction,
    #[cfg_attr(feature = "serialize", serde(rename = "*="))]
    Multiplication,
    #[cfg_attr(feature = "serialize", serde(rename = "/="))]
    Division,
    #[cfg_attr(feature = "serialize", serde(rename = "%="))]
    Remainder,
    #[cfg_attr(feature = "serialize", serde(rename = "<<="))]
    ShiftLeft,
    #[cfg_attr(feature = "serialize", serde(rename = ">>="))]
    ShiftRight,
    #[cfg_attr(feature = "serialize", serde(rename = ">>>="))]
    ShiftRightZeroFill,
    #[cfg_attr(feature = "serialize", serde(rename = "|="))]
    BitwiseOR,
    #[cfg_attr(feature = "serialize", serde(rename = "^="))]
    BitwiseXOR,
    #[cfg_attr(feature = "serialize", serde(rename = "&="))]
    BitwiseAnd,
    #[cfg_attr(feature = "serialize", serde(rename = "&&="))]
    LogicalAnd,
    #[cfg_attr(feature = "serialize", serde(rename = "||="))]
    LogicalOr,
    #[cfg_attr(feature = "serialize", serde(rename = "??="))]
    LogicalNullish,
    #[cfg_attr(feature = "serialize", serde(rename = "**="))]
    Exponential,
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

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum BinaryOperator {
    #[cfg_attr(feature = "serialize", serde(rename = "=="))]
    Equality,
    #[cfg_attr(feature = "serialize", serde(rename = "!="))]
    Inequality,
    #[cfg_attr(feature = "serialize", serde(rename = "==="))]
    StrictEquality,
    #[cfg_attr(feature = "serialize", serde(rename = "!=="))]
    StrictInequality,
    #[cfg_attr(feature = "serialize", serde(rename = "<"))]
    LessThan,
    #[cfg_attr(feature = "serialize", serde(rename = "<="))]
    LessEqualThan,
    #[cfg_attr(feature = "serialize", serde(rename = ">"))]
    GreaterThan,
    #[cfg_attr(feature = "serialize", serde(rename = ">="))]
    GreaterEqualThan,
    #[cfg_attr(feature = "serialize", serde(rename = "<<"))]
    ShiftLeft,
    #[cfg_attr(feature = "serialize", serde(rename = ">>"))]
    ShiftRight,
    #[cfg_attr(feature = "serialize", serde(rename = ">>>"))]
    ShiftRightZeroFill,
    #[cfg_attr(feature = "serialize", serde(rename = "+"))]
    Addition,
    #[cfg_attr(feature = "serialize", serde(rename = "-"))]
    Subtraction,
    #[cfg_attr(feature = "serialize", serde(rename = "*"))]
    Multiplication,
    #[cfg_attr(feature = "serialize", serde(rename = "/"))]
    Division,
    #[cfg_attr(feature = "serialize", serde(rename = "%"))]
    Remainder,
    #[cfg_attr(feature = "serialize", serde(rename = "|"))]
    BitwiseOR,
    #[cfg_attr(feature = "serialize", serde(rename = "^"))]
    BitwiseXOR,
    #[cfg_attr(feature = "serialize", serde(rename = "&"))]
    BitwiseAnd,
    #[cfg_attr(feature = "serialize", serde(rename = "in"))]
    In,
    #[cfg_attr(feature = "serialize", serde(rename = "instanceof"))]
    Instanceof,
    #[cfg_attr(feature = "serialize", serde(rename = "**"))]
    Exponential,
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
            | Self::In => Precedence::Equality,
            Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill => Precedence::Relational,
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
                Precedence::Equality
            }
            Self::LessThan
            | Self::LessEqualThan
            | Self::GreaterThan
            | Self::GreaterEqualThan
            | Self::Instanceof
            | Self::In => Precedence::Relational,
            Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill => Precedence::Shift,
            Self::Subtraction | Self::Addition => Precedence::Add,
            Self::Multiplication | Self::Remainder | Self::Division => Precedence::Multiply,
            Self::Exponential => Precedence::Exponential,
        }
    }
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", derive(Tsify))]
pub enum LogicalOperator {
    #[cfg_attr(feature = "serialize", serde(rename = "||"))]
    Or,
    #[cfg_attr(feature = "serialize", serde(rename = "&&"))]
    And,
    #[cfg_attr(feature = "serialize", serde(rename = "??"))]
    Coalesce,
}

impl LogicalOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Or => "||",
            Self::And => "&&",
            Self::Coalesce => "??",
        }
    }
}

impl GetPrecedence for LogicalOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Or => Precedence::LogicalOr,
            Self::And => Precedence::LogicalAnd,
            Self::Coalesce => Precedence::Coalesce,
        }
    }
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", derive(Tsify))]
pub enum UnaryOperator {
    #[cfg_attr(feature = "serialize", serde(rename = "-"))]
    UnaryNegation,
    #[cfg_attr(feature = "serialize", serde(rename = "+"))]
    UnaryPlus,
    #[cfg_attr(feature = "serialize", serde(rename = "!"))]
    LogicalNot,
    #[cfg_attr(feature = "serialize", serde(rename = "~"))]
    BitwiseNot,
    #[cfg_attr(feature = "serialize", serde(rename = "typeof"))]
    Typeof,
    #[cfg_attr(feature = "serialize", serde(rename = "void"))]
    Void,
    #[cfg_attr(feature = "serialize", serde(rename = "delete"))]
    Delete,
}

impl UnaryOperator {
    pub fn is_arithmetic(self) -> bool {
        matches!(self, Self::UnaryNegation | Self::UnaryPlus)
    }

    pub fn is_bitwise(self) -> bool {
        matches!(self, Self::BitwiseNot)
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

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", derive(Tsify))]
pub enum UpdateOperator {
    #[cfg_attr(feature = "serialize", serde(rename = "++"))]
    Increment,
    #[cfg_attr(feature = "serialize", serde(rename = "--"))]
    Decrement,
}

impl UpdateOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }
}
