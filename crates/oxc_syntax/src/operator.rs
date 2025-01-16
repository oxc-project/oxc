//! ECMAScript operators.
//!
//! Not all operators are punctuation - some, such as `delete`, are keywords.

use oxc_allocator::CloneIn;
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::cmp::ContentEq;

use crate::precedence::{GetPrecedence, Precedence};

/// Operators that may be used in assignment epxressions.
///
/// ## References
/// - [13.15 Assignment Operators](https://tc39.es/ecma262/#sec-assignment-operators)
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ESTree)]
pub enum AssignmentOperator {
    /// `=`
    #[estree(rename = "=")]
    Assign = 0,
    /// `+=`
    #[estree(rename = "+=")]
    Addition = 1,
    /// `-=`
    #[estree(rename = "-=")]
    Subtraction = 2,
    /// `*=`
    #[estree(rename = "*=")]
    Multiplication = 3,
    /// `/=`
    #[estree(rename = "/=")]
    Division = 4,
    /// `%=`
    #[estree(rename = "%=")]
    Remainder = 5,
    /// `**=`
    #[estree(rename = "**=")]
    Exponential = 6,
    /// `<<=`
    #[estree(rename = "<<=")]
    ShiftLeft = 7,
    /// `>>=`
    #[estree(rename = ">>=")]
    ShiftRight = 8,
    /// `>>>=`
    #[estree(rename = ">>>=")]
    ShiftRightZeroFill = 9,
    /// `|=`
    #[estree(rename = "|=")]
    BitwiseOR = 10,
    /// `^=`
    #[estree(rename = "^=")]
    BitwiseXOR = 11,
    /// `&=`
    #[estree(rename = "&=")]
    BitwiseAnd = 12,
    /// `||=`
    #[estree(rename = "||=")]
    LogicalOr = 13,
    /// `&&=`
    #[estree(rename = "&&=")]
    LogicalAnd = 14,
    /// `??=`
    #[estree(rename = "??=")]
    LogicalNullish = 15,
}

impl AssignmentOperator {
    /// Returns `true` for `=`.
    pub fn is_assign(self) -> bool {
        self == Self::Assign
    }

    /// Returns `true` for '||=`, `&&=`, and `??=`.
    pub fn is_logical(self) -> bool {
        matches!(self, Self::LogicalOr | Self::LogicalAnd | Self::LogicalNullish)
    }

    /// Returns `true` for `+=`, `-=`, `*=`, `/=`, `%=`, and `**=`.
    #[rustfmt::skip]
    pub fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Addition | Self::Subtraction | Self::Multiplication
            | Self::Division | Self::Remainder | Self::Exponential
        )
    }

    /// Returns `true` for `|=`, `^=`, `&=`, `<<=`, `>>=`, and `>>>=`.
    #[rustfmt::skip]
    pub fn is_bitwise(self) -> bool {
        matches!(
            self,
            Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill
            | Self::BitwiseOR | Self::BitwiseXOR | Self::BitwiseAnd
        )
    }

    /// Get [`LogicalOperator`] corresponding to this [`AssignmentOperator`].
    pub fn to_logical_operator(self) -> Option<LogicalOperator> {
        match self {
            Self::LogicalOr => Some(LogicalOperator::Or),
            Self::LogicalAnd => Some(LogicalOperator::And),
            Self::LogicalNullish => Some(LogicalOperator::Coalesce),
            _ => None,
        }
    }

    /// Get [`BinaryOperator`] corresponding to this [`AssignmentOperator`].
    pub fn to_binary_operator(self) -> Option<BinaryOperator> {
        match self {
            Self::Addition => Some(BinaryOperator::Addition),
            Self::Subtraction => Some(BinaryOperator::Subtraction),
            Self::Multiplication => Some(BinaryOperator::Multiplication),
            Self::Division => Some(BinaryOperator::Division),
            Self::Remainder => Some(BinaryOperator::Remainder),
            Self::Exponential => Some(BinaryOperator::Exponential),
            Self::ShiftLeft => Some(BinaryOperator::ShiftLeft),
            Self::ShiftRight => Some(BinaryOperator::ShiftRight),
            Self::ShiftRightZeroFill => Some(BinaryOperator::ShiftRightZeroFill),
            Self::BitwiseOR => Some(BinaryOperator::BitwiseOR),
            Self::BitwiseXOR => Some(BinaryOperator::BitwiseXOR),
            Self::BitwiseAnd => Some(BinaryOperator::BitwiseAnd),
            _ => None,
        }
    }

    /// Get the string representation of this operator.
    ///
    /// This is the same as how the operator appears in source code.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Addition => "+=",
            Self::Subtraction => "-=",
            Self::Multiplication => "*=",
            Self::Division => "/=",
            Self::Remainder => "%=",
            Self::Exponential => "**=",
            Self::ShiftLeft => "<<=",
            Self::ShiftRight => ">>=",
            Self::ShiftRightZeroFill => ">>>=",
            Self::BitwiseOR => "|=",
            Self::BitwiseXOR => "^=",
            Self::BitwiseAnd => "&=",
            Self::LogicalOr => "||=",
            Self::LogicalAnd => "&&=",
            Self::LogicalNullish => "??=",
        }
    }
}

/// Operators used in binary expressions. Does not include logical binary
/// operators, which are in [`LogicalOperator`].
///
/// ## References
/// - [12.10 Binary Logical Operators](https://tc39.es/ecma262/#sec-binary-logical-operators)
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, ContentEq, ESTree)]
pub enum BinaryOperator {
    /// `==`
    #[estree(rename = "==")]
    Equality = 0,
    /// `!=`
    #[estree(rename = "!=")]
    Inequality = 1,
    /// `===`
    #[estree(rename = "===")]
    StrictEquality = 2,
    /// `!==`
    #[estree(rename = "!==")]
    StrictInequality = 3,
    /// `<`
    #[estree(rename = "<")]
    LessThan = 4,
    /// `<=`
    #[estree(rename = "<=")]
    LessEqualThan = 5,
    /// `>`
    #[estree(rename = ">")]
    GreaterThan = 6,
    /// `>=`
    #[estree(rename = ">=")]
    GreaterEqualThan = 7,
    /// `+`
    #[estree(rename = "+")]
    Addition = 8,
    /// `-`
    #[estree(rename = "-")]
    Subtraction = 9,
    /// `*`
    #[estree(rename = "*")]
    Multiplication = 10,
    /// `/`
    #[estree(rename = "/")]
    Division = 11,
    /// `%`
    #[estree(rename = "%")]
    Remainder = 12,
    /// `**`
    #[estree(rename = "**")]
    Exponential = 13,
    /// `<<`
    #[estree(rename = "<<")]
    ShiftLeft = 14,
    /// `>>`
    #[estree(rename = ">>")]
    ShiftRight = 15,
    /// `>>>`
    #[estree(rename = ">>>")]
    ShiftRightZeroFill = 16,
    /// `|`
    #[estree(rename = "|")]
    BitwiseOR = 17,
    /// `^`
    #[estree(rename = "^")]
    BitwiseXOR = 18,
    /// `&`
    #[estree(rename = "&")]
    BitwiseAnd = 19,
    /// `in`
    #[estree(rename = "in")]
    In = 20,
    /// `instanceof`
    #[estree(rename = "instanceof")]
    Instanceof = 21,
}

impl BinaryOperator {
    /// Returns `true` for inequality or inequality operarors
    #[rustfmt::skip]
    pub fn is_equality(self) -> bool {
        matches!(self, Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality)
    }

    /// Returns `true` for logical comparison operators
    #[rustfmt::skip]
    pub fn is_compare(self) -> bool {
        matches!(self, Self::LessThan | Self::LessEqualThan | Self::GreaterThan | Self::GreaterEqualThan)
    }

    /// Returns `true` for arithmetic operators
    #[rustfmt::skip]
    pub fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Addition | Self::Subtraction | Self::Multiplication
            | Self::Division | Self::Remainder | Self::Exponential
        )
    }

    /// Returns `true` for multiplication (`*`), division (`/`), and remainder
    /// (`%`) operators
    pub fn is_multiplicative(self) -> bool {
        matches!(self, Self::Multiplication | Self::Division | Self::Remainder)
    }

    /// Returns `true` for object relation operators
    pub fn is_relational(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    /// Returns `true` if this is an [`In`](BinaryOperator::In) operator.
    pub fn is_in(self) -> bool {
        self == Self::In
    }

    /// Returns `true` if this is an [`In`](BinaryOperator::Instanceof) operator.
    pub fn is_instance_of(self) -> bool {
        self == Self::Instanceof
    }

    /// Returns `true` for any bitwise operator
    #[rustfmt::skip]
    pub fn is_bitwise(self) -> bool {
        self.is_bitshift() || matches!(self, Self::BitwiseOR | Self::BitwiseXOR | Self::BitwiseAnd)
    }

    /// Returns `true` for any bitshift operator
    pub fn is_bitshift(self) -> bool {
        matches!(self, Self::ShiftLeft | Self::ShiftRight | Self::ShiftRightZeroFill)
    }

    /// Returns `true` for any numeric or string binary operator
    pub fn is_numeric_or_string_binary_operator(self) -> bool {
        self.is_arithmetic() || self.is_bitwise()
    }

    /// Returns `true` if this operator is a keyword instead of punctuation.
    pub fn is_keyword(self) -> bool {
        matches!(self, Self::In | Self::Instanceof)
    }

    /// Try to get the operator that performs the inverse comparison operation.
    /// [`None`] if this is not a comparison operator.
    pub fn compare_inverse_operator(self) -> Option<Self> {
        match self {
            Self::LessThan => Some(Self::GreaterThan),
            Self::LessEqualThan => Some(Self::GreaterEqualThan),
            Self::GreaterThan => Some(Self::LessThan),
            Self::GreaterEqualThan => Some(Self::LessEqualThan),
            _ => None,
        }
    }

    /// Try to get the operator that performs the inverse equality operation.
    /// [`None`] if this is not an equality operator.
    pub fn equality_inverse_operator(self) -> Option<Self> {
        match self {
            Self::Equality => Some(Self::Inequality),
            Self::Inequality => Some(Self::Equality),
            Self::StrictEquality => Some(Self::StrictInequality),
            Self::StrictInequality => Some(Self::StrictEquality),
            _ => None,
        }
    }

    /// Get [`AssignmentOperator`] corresponding to this [`BinaryOperator`].
    pub fn to_assignment_operator(self) -> Option<AssignmentOperator> {
        match self {
            Self::Addition => Some(AssignmentOperator::Addition),
            Self::Subtraction => Some(AssignmentOperator::Subtraction),
            Self::Multiplication => Some(AssignmentOperator::Multiplication),
            Self::Division => Some(AssignmentOperator::Division),
            Self::Remainder => Some(AssignmentOperator::Remainder),
            Self::Exponential => Some(AssignmentOperator::Exponential),
            Self::ShiftLeft => Some(AssignmentOperator::ShiftLeft),
            Self::ShiftRight => Some(AssignmentOperator::ShiftRight),
            Self::ShiftRightZeroFill => Some(AssignmentOperator::ShiftRightZeroFill),
            Self::BitwiseOR => Some(AssignmentOperator::BitwiseOR),
            Self::BitwiseXOR => Some(AssignmentOperator::BitwiseXOR),
            Self::BitwiseAnd => Some(AssignmentOperator::BitwiseAnd),
            _ => None,
        }
    }

    /// The string representation of this operator as it appears in source code.
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
            Self::Addition => "+",
            Self::Subtraction => "-",
            Self::Multiplication => "*",
            Self::Division => "/",
            Self::Remainder => "%",
            Self::Exponential => "**",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::ShiftRightZeroFill => ">>>",
            Self::BitwiseOR => "|",
            Self::BitwiseXOR => "^",
            Self::BitwiseAnd => "&",
            Self::In => "in",
            Self::Instanceof => "instanceof",
        }
    }

    /// Get the operator that has a lower precedence than this operator by a
    /// single level. Use [`BinaryOperator::precedence`] to get the operator
    /// with a higher precedence.
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

/// Logical binary operators
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ESTree)]
pub enum LogicalOperator {
    /// `||`
    #[estree(rename = "||")]
    Or = 0,
    /// `&&`
    #[estree(rename = "&&")]
    And = 1,
    /// `??`
    #[estree(rename = "??")]
    Coalesce = 2,
}

impl LogicalOperator {
    /// Get the string representation of this operator as it appears in source code.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Or => "||",
            Self::And => "&&",
            Self::Coalesce => "??",
        }
    }

    /// Get the operator that has a lower precedence than this operator by a
    /// single level. Use [`BinaryOperator::precedence`] to get the operator
    /// with a higher precedence.
    pub fn lower_precedence(&self) -> Precedence {
        match self {
            Self::Or => Precedence::NullishCoalescing,
            Self::And => Precedence::LogicalOr,
            Self::Coalesce => Precedence::Conditional,
        }
    }

    /// Get [`AssignmentOperator`] corresponding to this [`LogicalOperator`].
    pub fn to_assignment_operator(self) -> AssignmentOperator {
        match self {
            Self::Or => AssignmentOperator::LogicalOr,
            Self::And => AssignmentOperator::LogicalAnd,
            Self::Coalesce => AssignmentOperator::LogicalNullish,
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

/// Operators used in unary operators.
///
/// Does not include self-modifying operators, which are in [`UpdateOperator`].
///
/// ## References
/// - [12.5 Unary Operators](https://tc39.es/ecma262/#sec-unary-operators)
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, ContentEq, ESTree)]
pub enum UnaryOperator {
    /// `+`
    #[estree(rename = "+")]
    UnaryPlus = 0,
    /// `-`
    #[estree(rename = "-")]
    UnaryNegation = 1,
    /// `!`
    #[estree(rename = "!")]
    LogicalNot = 2,
    /// `~`
    #[estree(rename = "~")]
    BitwiseNot = 3,
    /// `typeof`
    #[estree(rename = "typeof")]
    Typeof = 4,
    /// `void`
    #[estree(rename = "void")]
    Void = 5,
    /// `delete`
    #[estree(rename = "delete")]
    Delete = 6,
}

impl UnaryOperator {
    /// Returns `true` if this operator is a unary arithmetic operator.
    pub fn is_arithmetic(self) -> bool {
        matches!(self, Self::UnaryPlus | Self::UnaryNegation)
    }

    /// Returns `true` if this operator is a [`LogicalNot`].
    ///
    /// [`LogicalNot`]: UnaryOperator::LogicalNot
    pub fn is_not(self) -> bool {
        self == Self::LogicalNot
    }

    /// Returns `true` if this operator is a bitwise operator.
    pub fn is_bitwise(self) -> bool {
        self == Self::BitwiseNot
    }

    /// Returns `true` if this is the [`void`](UnaryOperator::Typeof) operator.
    pub fn is_typeof(self) -> bool {
        self == Self::Typeof
    }

    /// Returns `true` if this is the [`void`](UnaryOperator::Void) operator.
    pub fn is_void(self) -> bool {
        self == Self::Void
    }

    /// Returns `true` if this is the [`delete`](UnaryOperator::Delete) operator.
    pub fn is_delete(self) -> bool {
        self == Self::Delete
    }

    /// Returns `true` if this operator is a keyword instead of punctuation.
    pub fn is_keyword(self) -> bool {
        matches!(self, Self::Typeof | Self::Void | Self::Delete)
    }

    /// Get the string representation of this operator as it appears in source code.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnaryPlus => "+",
            Self::UnaryNegation => "-",
            Self::LogicalNot => "!",
            Self::BitwiseNot => "~",
            Self::Typeof => "typeof",
            Self::Void => "void",
            Self::Delete => "delete",
        }
    }
}

/// Unary update operators.
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, ContentEq, ESTree)]
pub enum UpdateOperator {
    /// `++`
    #[estree(rename = "++")]
    Increment = 0,
    /// `--`
    #[estree(rename = "--")]
    Decrement = 1,
}

impl UpdateOperator {
    /// Get the string representation of this operator as it appears in source code.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }
}
