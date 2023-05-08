use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};

use crate::lexer::Kind;

/// [Operator Precedence](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Operator_Precedence#table)
#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum BindingPower {
    Comma = 0,
    // Yield = 1,
    // Assignment = 2,
    // Conditional = 3,
    Coalesce = 4,
    LogicalOr = 5,
    LogicalAnd = 6,
    BitwiseOr = 7,
    BitwiseXor = 8,
    BitwiseAnd = 9,
    Equality = 10,
    Relational = 11,
    Shift = 12,
    Additive = 13,
    Multiplicative = 14,
    Exponential = 15,
    // Unary = 16,
    // Update = 17,
    // LeftHandSide = 18,
    // Member = 19,
    // Primary = 20,
}

impl BindingPower {
    pub fn lowest() -> Self {
        Self::Comma
    }

    pub fn is_right_associative(binding_power: Self) -> bool {
        binding_power == Self::Exponential
    }

    pub fn value(kind: Kind) -> Option<Self> {
        match kind {
            Kind::Question2 => Some(Self::Coalesce),
            Kind::Pipe2 => Some(Self::LogicalOr),
            Kind::Amp2 => Some(Self::LogicalAnd),
            Kind::Pipe => Some(Self::BitwiseOr),
            Kind::Caret => Some(Self::BitwiseXor),
            Kind::Amp => Some(Self::BitwiseAnd),
            Kind::Eq2 | Kind::Eq3 | Kind::Neq | Kind::Neq2 => Some(Self::Equality),
            Kind::LAngle
            | Kind::RAngle
            | Kind::LtEq
            | Kind::GtEq
            | Kind::Instanceof
            | Kind::In
            | Kind::As
            | Kind::Satisfies => Some(Self::Relational),
            Kind::ShiftLeft | Kind::ShiftRight | Kind::ShiftRight3 => Some(Self::Shift),
            Kind::Plus | Kind::Minus => Some(Self::Additive),
            Kind::Star | Kind::Slash | Kind::Percent => Some(Self::Multiplicative),
            Kind::Star2 => Some(Self::Exponential),
            _ => None,
        }
    }
}

pub fn map_binary_operator(kind: Kind) -> BinaryOperator {
    match kind {
        Kind::Eq2 => BinaryOperator::Equality,
        Kind::Neq => BinaryOperator::Inequality,
        Kind::Eq3 => BinaryOperator::StrictEquality,
        Kind::Neq2 => BinaryOperator::StrictInequality,
        Kind::LAngle => BinaryOperator::LessThan,
        Kind::LtEq => BinaryOperator::LessEqualThan,
        Kind::RAngle => BinaryOperator::GreaterThan,
        Kind::GtEq => BinaryOperator::GreaterEqualThan,
        Kind::ShiftLeft => BinaryOperator::ShiftLeft,
        Kind::ShiftRight => BinaryOperator::ShiftRight,
        Kind::ShiftRight3 => BinaryOperator::ShiftRightZeroFill,
        Kind::Plus => BinaryOperator::Addition,
        Kind::Minus => BinaryOperator::Subtraction,
        Kind::Star => BinaryOperator::Multiplication,
        Kind::Slash => BinaryOperator::Division,
        Kind::Percent => BinaryOperator::Remainder,
        Kind::Pipe => BinaryOperator::BitwiseOR,
        Kind::Caret => BinaryOperator::BitwiseXOR,
        Kind::Amp => BinaryOperator::BitwiseAnd,
        Kind::In => BinaryOperator::In,
        Kind::Instanceof => BinaryOperator::Instanceof,
        Kind::Star2 => BinaryOperator::Exponential,
        _ => unreachable!("Binary Operator: {kind:?}"),
    }
}

pub fn map_unary_operator(kind: Kind) -> UnaryOperator {
    match kind {
        Kind::Minus => UnaryOperator::UnaryNegation,
        Kind::Plus => UnaryOperator::UnaryPlus,
        Kind::Bang => UnaryOperator::LogicalNot,
        Kind::Tilde => UnaryOperator::BitwiseNot,
        Kind::Typeof => UnaryOperator::Typeof,
        Kind::Void => UnaryOperator::Void,
        Kind::Delete => UnaryOperator::Delete,
        _ => unreachable!("Unary Operator: {kind:?}"),
    }
}

pub fn map_logical_operator(kind: Kind) -> LogicalOperator {
    match kind {
        Kind::Pipe2 => LogicalOperator::Or,
        Kind::Amp2 => LogicalOperator::And,
        Kind::Question2 => LogicalOperator::Coalesce,
        _ => unreachable!("Logical Operator: {kind:?}"),
    }
}

pub fn map_update_operator(kind: Kind) -> UpdateOperator {
    match kind {
        Kind::Plus2 => UpdateOperator::Increment,
        Kind::Minus2 => UpdateOperator::Decrement,
        _ => unreachable!("Update Operator: {kind:?}"),
    }
}

pub fn map_assignment_operator(kind: Kind) -> AssignmentOperator {
    match kind {
        Kind::Eq => AssignmentOperator::Assign,
        Kind::PlusEq => AssignmentOperator::Addition,
        Kind::MinusEq => AssignmentOperator::Subtraction,
        Kind::StarEq => AssignmentOperator::Multiplication,
        Kind::SlashEq => AssignmentOperator::Division,
        Kind::PercentEq => AssignmentOperator::Remainder,
        Kind::ShiftLeftEq => AssignmentOperator::ShiftLeft,
        Kind::ShiftRightEq => AssignmentOperator::ShiftRight,
        Kind::ShiftRight3Eq => AssignmentOperator::ShiftRightZeroFill,
        Kind::PipeEq => AssignmentOperator::BitwiseOR,
        Kind::CaretEq => AssignmentOperator::BitwiseXOR,
        Kind::AmpEq => AssignmentOperator::BitwiseAnd,
        Kind::Amp2Eq => AssignmentOperator::LogicalAnd,
        Kind::Pipe2Eq => AssignmentOperator::LogicalOr,
        Kind::Question2Eq => AssignmentOperator::LogicalNullish,
        Kind::Star2Eq => AssignmentOperator::Exponential,
        _ => unreachable!("Update Operator: {kind:?}"),
    }
}
