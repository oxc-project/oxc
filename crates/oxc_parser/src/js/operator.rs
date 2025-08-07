use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    precedence::Precedence,
};

use crate::lexer::Kind;

pub fn kind_to_precedence(kind: Kind) -> Option<Precedence> {
    match kind {
        Kind::Question2 => Some(Precedence::NullishCoalescing),
        Kind::Pipe2 => Some(Precedence::LogicalOr),
        Kind::Amp2 => Some(Precedence::LogicalAnd),
        Kind::Pipe => Some(Precedence::BitwiseOr),
        Kind::Caret => Some(Precedence::BitwiseXor),
        Kind::Amp => Some(Precedence::BitwiseAnd),
        kind if is_equality_operator(kind) => Some(Precedence::Equals),
        kind if is_comparison_operator(kind) => Some(Precedence::Compare),
        kind if is_shift_operator(kind) => Some(Precedence::Shift),
        Kind::Plus | Kind::Minus => Some(Precedence::Add),
        kind if is_multiplicative_operator(kind) => Some(Precedence::Multiply),
        Kind::Star2 => Some(Precedence::Exponentiation),
        Kind::As | Kind::Satisfies => Some(Precedence::Compare),
        _ => None,
    }
}

fn is_equality_operator(kind: Kind) -> bool {
    matches!(kind, Kind::Eq2 | Kind::Eq3 | Kind::Neq | Kind::Neq2)
}

fn is_comparison_operator(kind: Kind) -> bool {
    matches!(kind, Kind::LAngle | Kind::RAngle | Kind::LtEq | Kind::GtEq | Kind::Instanceof | Kind::In)
}

fn is_shift_operator(kind: Kind) -> bool {
    matches!(kind, Kind::ShiftLeft | Kind::ShiftRight | Kind::ShiftRight3)
}

fn is_multiplicative_operator(kind: Kind) -> bool {
    matches!(kind, Kind::Star | Kind::Slash | Kind::Percent)
}

pub fn map_binary_operator(kind: Kind) -> BinaryOperator {
    match kind {
        // Equality operators
        Kind::Eq2 => BinaryOperator::Equality,
        Kind::Neq => BinaryOperator::Inequality,
        Kind::Eq3 => BinaryOperator::StrictEquality,
        Kind::Neq2 => BinaryOperator::StrictInequality,
        
        // Comparison operators
        Kind::LAngle => BinaryOperator::LessThan,
        Kind::LtEq => BinaryOperator::LessEqualThan,
        Kind::RAngle => BinaryOperator::GreaterThan,
        Kind::GtEq => BinaryOperator::GreaterEqualThan,
        
        // Shift operators
        Kind::ShiftLeft => BinaryOperator::ShiftLeft,
        Kind::ShiftRight => BinaryOperator::ShiftRight,
        Kind::ShiftRight3 => BinaryOperator::ShiftRightZeroFill,
        
        // Arithmetic operators
        Kind::Plus => BinaryOperator::Addition,
        Kind::Minus => BinaryOperator::Subtraction,
        Kind::Star => BinaryOperator::Multiplication,
        Kind::Slash => BinaryOperator::Division,
        Kind::Percent => BinaryOperator::Remainder,
        Kind::Star2 => BinaryOperator::Exponential,
        
        // Bitwise operators
        Kind::Pipe => BinaryOperator::BitwiseOR,
        Kind::Caret => BinaryOperator::BitwiseXOR,
        Kind::Amp => BinaryOperator::BitwiseAnd,
        
        // Type operators
        Kind::In => BinaryOperator::In,
        Kind::Instanceof => BinaryOperator::Instanceof,
        
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
        // Basic assignment
        Kind::Eq => AssignmentOperator::Assign,
        
        // Arithmetic assignment operators
        Kind::PlusEq => AssignmentOperator::Addition,
        Kind::MinusEq => AssignmentOperator::Subtraction,
        Kind::StarEq => AssignmentOperator::Multiplication,
        Kind::SlashEq => AssignmentOperator::Division,
        Kind::PercentEq => AssignmentOperator::Remainder,
        Kind::Star2Eq => AssignmentOperator::Exponential,
        
        // Shift assignment operators
        Kind::ShiftLeftEq => AssignmentOperator::ShiftLeft,
        Kind::ShiftRightEq => AssignmentOperator::ShiftRight,
        Kind::ShiftRight3Eq => AssignmentOperator::ShiftRightZeroFill,
        
        // Bitwise assignment operators
        Kind::PipeEq => AssignmentOperator::BitwiseOR,
        Kind::CaretEq => AssignmentOperator::BitwiseXOR,
        Kind::AmpEq => AssignmentOperator::BitwiseAnd,
        
        // Logical assignment operators
        Kind::Amp2Eq => AssignmentOperator::LogicalAnd,
        Kind::Pipe2Eq => AssignmentOperator::LogicalOr,
        Kind::Question2Eq => AssignmentOperator::LogicalNullish,
        
        _ => unreachable!("Assignment Operator: {kind:?}"),
    }
}
