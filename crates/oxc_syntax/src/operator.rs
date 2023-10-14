#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum AssignmentOperator {
    #[cfg_attr(feature = "serde", serde(rename = "="))]
    Assign,
    #[cfg_attr(feature = "serde", serde(rename = "+="))]
    Addition,
    #[cfg_attr(feature = "serde", serde(rename = "-="))]
    Subtraction,
    #[cfg_attr(feature = "serde", serde(rename = "*="))]
    Multiplication,
    #[cfg_attr(feature = "serde", serde(rename = "/="))]
    Division,
    #[cfg_attr(feature = "serde", serde(rename = "%="))]
    Remainder,
    #[cfg_attr(feature = "serde", serde(rename = "<<="))]
    ShiftLeft,
    #[cfg_attr(feature = "serde", serde(rename = ">>="))]
    ShiftRight,
    #[cfg_attr(feature = "serde", serde(rename = ">>>="))]
    ShiftRightZeroFill,
    #[cfg_attr(feature = "serde", serde(rename = "|="))]
    BitwiseOR,
    #[cfg_attr(feature = "serde", serde(rename = "^="))]
    BitwiseXOR,
    #[cfg_attr(feature = "serde", serde(rename = "&="))]
    BitwiseAnd,
    #[cfg_attr(feature = "serde", serde(rename = "&&="))]
    LogicalAnd,
    #[cfg_attr(feature = "serde", serde(rename = "||="))]
    LogicalOr,
    #[cfg_attr(feature = "serde", serde(rename = "??="))]
    LogicalNullish,
    #[cfg_attr(feature = "serde", serde(rename = "**="))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum BinaryOperator {
    #[cfg_attr(feature = "serde", serde(rename = "=="))]
    Equality,
    #[cfg_attr(feature = "serde", serde(rename = "!="))]
    Inequality,
    #[cfg_attr(feature = "serde", serde(rename = "==="))]
    StrictEquality,
    #[cfg_attr(feature = "serde", serde(rename = "!=="))]
    StrictInequality,
    #[cfg_attr(feature = "serde", serde(rename = "<"))]
    LessThan,
    #[cfg_attr(feature = "serde", serde(rename = "<="))]
    LessEqualThan,
    #[cfg_attr(feature = "serde", serde(rename = ">"))]
    GreaterThan,
    #[cfg_attr(feature = "serde", serde(rename = ">="))]
    GreaterEqualThan,
    #[cfg_attr(feature = "serde", serde(rename = "<<"))]
    ShiftLeft,
    #[cfg_attr(feature = "serde", serde(rename = ">>"))]
    ShiftRight,
    #[cfg_attr(feature = "serde", serde(rename = ">>>"))]
    ShiftRightZeroFill,
    #[cfg_attr(feature = "serde", serde(rename = "+"))]
    Addition,
    #[cfg_attr(feature = "serde", serde(rename = "-"))]
    Subtraction,
    #[cfg_attr(feature = "serde", serde(rename = "*"))]
    Multiplication,
    #[cfg_attr(feature = "serde", serde(rename = "/"))]
    Division,
    #[cfg_attr(feature = "serde", serde(rename = "%"))]
    Remainder,
    #[cfg_attr(feature = "serde", serde(rename = "|"))]
    BitwiseOR,
    #[cfg_attr(feature = "serde", serde(rename = "^"))]
    BitwiseXOR,
    #[cfg_attr(feature = "serde", serde(rename = "&"))]
    BitwiseAnd,
    #[cfg_attr(feature = "serde", serde(rename = "in"))]
    In,
    #[cfg_attr(feature = "serde", serde(rename = "instanceof"))]
    Instanceof,
    #[cfg_attr(feature = "serde", serde(rename = "**"))]
    Exponential,
}

impl BinaryOperator {
    pub fn is_equality(self) -> bool {
        matches!(
            self,
            Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality
        )
    }

    pub fn is_compare(self) -> bool {
        matches!(
            self,
            Self::LessThan | Self::LessEqualThan | Self::GreaterThan | Self::GreaterEqualThan
        )
    }

    pub fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Addition
                | Self::Subtraction
                | Self::Multiplication
                | Self::Division
                | Self::Remainder
                | Self::Exponential
        )
    }

    pub fn is_relational(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    pub fn is_bitwise(self) -> bool {
        matches!(
            self,
            Self::BitwiseOR
                | Self::BitwiseXOR
                | Self::BitwiseAnd
                | Self::ShiftLeft
                | Self::ShiftRight
                | Self::ShiftRightZeroFill,
        )
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum LogicalOperator {
    #[cfg_attr(feature = "serde", serde(rename = "||"))]
    Or,
    #[cfg_attr(feature = "serde", serde(rename = "&&"))]
    And,
    #[cfg_attr(feature = "serde", serde(rename = "??"))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum UnaryOperator {
    #[cfg_attr(feature = "serde", serde(rename = "-"))]
    UnaryNegation,
    #[cfg_attr(feature = "serde", serde(rename = "+"))]
    UnaryPlus,
    #[cfg_attr(feature = "serde", serde(rename = "!"))]
    LogicalNot,
    #[cfg_attr(feature = "serde", serde(rename = "~"))]
    BitwiseNot,
    #[cfg_attr(feature = "serde", serde(rename = "typeof"))]
    Typeof,
    #[cfg_attr(feature = "serde", serde(rename = "void"))]
    Void,
    #[cfg_attr(feature = "serde", serde(rename = "delete"))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum UpdateOperator {
    #[cfg_attr(feature = "serde", serde(rename = "++"))]
    Increment,
    #[cfg_attr(feature = "serde", serde(rename = "--"))]
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
