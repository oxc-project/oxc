use std::fmt::{Display, Formatter, Result};

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum Operator {
    AssignmentOperator(AssignmentOperator),
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
    UnaryOperator(UnaryOperator),
    UpdateOperator(UpdateOperator),
}

impl From<AssignmentOperator> for Operator {
    fn from(op: AssignmentOperator) -> Self {
        Self::AssignmentOperator(op)
    }
}

impl From<BinaryOperator> for Operator {
    fn from(op: BinaryOperator) -> Self {
        Self::BinaryOperator(op)
    }
}

impl From<LogicalOperator> for Operator {
    fn from(op: LogicalOperator) -> Self {
        Self::LogicalOperator(op)
    }
}

impl From<UnaryOperator> for Operator {
    fn from(op: UnaryOperator) -> Self {
        Self::UnaryOperator(op)
    }
}

impl From<UpdateOperator> for Operator {
    fn from(op: UpdateOperator) -> Self {
        Self::UpdateOperator(op)
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum AssignmentOperator {
    #[serde(rename = "=")]
    Assign,
    #[serde(rename = "+=")]
    Addition,
    #[serde(rename = "-=")]
    Subtraction,
    #[serde(rename = "*=")]
    Multiplication,
    #[serde(rename = "/=")]
    Division,
    #[serde(rename = "%=")]
    Remainder,
    #[serde(rename = "<<=")]
    ShiftLeft,
    #[serde(rename = ">>=")]
    ShiftRight,
    #[serde(rename = ">>>=")]
    ShiftRightZeroFill,
    #[serde(rename = "|=")]
    BitwiseOR,
    #[serde(rename = "^=")]
    BitwiseXOR,
    #[serde(rename = "&=")]
    BitwiseAnd,
    #[serde(rename = "&&=")]
    LogicalAnd,
    #[serde(rename = "||=")]
    LogicalOr,
    #[serde(rename = "??=")]
    LogicalNullish,
    #[serde(rename = "**=")]
    Exponential,
}

impl AssignmentOperator {
    #[must_use]
    pub const fn is_logical_operator(self) -> bool {
        matches!(self, Self::LogicalAnd | Self::LogicalOr | Self::LogicalNullish)
    }

    #[must_use]
    pub const fn is_arithmetic(self) -> bool {
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

    #[must_use]
    pub const fn is_bitwise(self) -> bool {
        matches!(self, Self::BitwiseOR | Self::BitwiseXOR | Self::BitwiseAnd)
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
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

impl Display for AssignmentOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    #[serde(rename = "==")]
    Equality,
    #[serde(rename = "!=")]
    Inequality,
    #[serde(rename = "===")]
    StrictEquality,
    #[serde(rename = "!==")]
    StrictInequality,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = "<=")]
    LessEqualThan,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = ">=")]
    GreaterEqualThan,
    #[serde(rename = "<<")]
    ShiftLeft,
    #[serde(rename = ">>")]
    ShiftRight,
    #[serde(rename = ">>>")]
    ShiftRightZeroFill,
    #[serde(rename = "+")]
    Addition,
    #[serde(rename = "-")]
    Subtraction,
    #[serde(rename = "*")]
    Multiplication,
    #[serde(rename = "/")]
    Division,
    #[serde(rename = "%")]
    Remainder,
    #[serde(rename = "|")]
    BitwiseOR,
    #[serde(rename = "^")]
    BitwiseXOR,
    #[serde(rename = "&")]
    BitwiseAnd,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "instanceof")]
    Instanceof,
    #[serde(rename = "**")]
    Exponential,
}

impl BinaryOperator {
    #[must_use]
    pub const fn is_equality(self) -> bool {
        matches!(
            self,
            Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality
        )
    }

    #[must_use]
    pub const fn is_compare(self) -> bool {
        matches!(
            self,
            Self::LessThan | Self::LessEqualThan | Self::GreaterThan | Self::GreaterEqualThan
        )
    }

    #[must_use]
    pub const fn is_arithmetic(self) -> bool {
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

    #[must_use]
    pub const fn is_relational(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    #[must_use]
    pub const fn is_bitwise(self) -> bool {
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

    #[must_use]
    pub const fn is_numeric_or_string_binary_operator(self) -> bool {
        self.is_arithmetic() || self.is_bitwise()
    }

    #[must_use]
    pub const fn is_keyword(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
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

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum LogicalOperator {
    #[serde(rename = "||")]
    Or,
    #[serde(rename = "&&")]
    And,
    #[serde(rename = "??")]
    Coalesce,
}

impl LogicalOperator {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Or => "||",
            Self::And => "&&",
            Self::Coalesce => "??",
        }
    }
}

impl Display for LogicalOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    #[serde(rename = "-")]
    UnaryNegation,
    #[serde(rename = "+")]
    UnaryPlus,
    #[serde(rename = "!")]
    LogicalNot,
    #[serde(rename = "~")]
    BitwiseNot,
    #[serde(rename = "typeof")]
    Typeof,
    #[serde(rename = "void")]
    Void,
    #[serde(rename = "delete")]
    Delete,
}

impl UnaryOperator {
    #[must_use]
    pub const fn operator(&self) -> Operator {
        Operator::UnaryOperator(*self)
    }

    #[must_use]
    pub const fn is_arithmetic(self) -> bool {
        matches!(self, Self::UnaryNegation | Self::UnaryPlus)
    }

    #[must_use]
    pub const fn is_bitwise(self) -> bool {
        matches!(self, Self::BitwiseNot)
    }

    #[must_use]
    pub const fn is_keyword(self) -> bool {
        matches!(self, Self::Typeof | Self::Void | Self::Delete)
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
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

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum UpdateOperator {
    #[serde(rename = "++")]
    Increment,
    #[serde(rename = "--")]
    Decrement,
}

impl UpdateOperator {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }
}

impl Display for UpdateOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
    }
}
