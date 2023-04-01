use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
    #[must_use]
    pub fn is_logical_operator(self) -> bool {
        matches!(self, Self::LogicalAnd | Self::LogicalOr | Self::LogicalNullish)
    }

    #[must_use]
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

    #[must_use]
    pub fn is_bitwise(self) -> bool {
        matches!(
            self,
            Self::BitwiseOR
                | Self::BitwiseXOR
                | Self::BitwiseAnd
                | Self::ShiftLeft
                | Self::ShiftRight
                | Self::ShiftRightZeroFill
        )
    }

    #[must_use]
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

impl Display for AssignmentOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
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
    #[must_use]
    pub fn is_equality(self) -> bool {
        matches!(
            self,
            Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality
        )
    }

    #[must_use]
    pub fn is_compare(self) -> bool {
        matches!(
            self,
            Self::LessThan | Self::LessEqualThan | Self::GreaterThan | Self::GreaterEqualThan
        )
    }

    #[must_use]
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

    #[must_use]
    pub fn is_relational(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    #[must_use]
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

    #[must_use]
    pub fn is_numeric_or_string_binary_operator(self) -> bool {
        self.is_arithmetic() || self.is_bitwise()
    }

    #[must_use]
    pub fn is_keyword(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    #[must_use]
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

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
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
    #[must_use]
    pub fn as_str(&self) -> &'static str {
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
    #[must_use]
    pub fn operator(&self) -> Operator {
        Operator::UnaryOperator(*self)
    }

    #[must_use]
    pub fn is_arithmetic(self) -> bool {
        matches!(self, Self::UnaryNegation | Self::UnaryPlus)
    }

    #[must_use]
    pub fn is_bitwise(self) -> bool {
        matches!(self, Self::BitwiseNot)
    }

    #[must_use]
    pub fn is_keyword(self) -> bool {
        matches!(self, Self::Typeof | Self::Void | Self::Delete)
    }

    #[must_use]
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

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let operator = self.as_str();
        write!(f, "{operator}")
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
    #[must_use]
    pub fn as_str(&self) -> &'static str {
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
