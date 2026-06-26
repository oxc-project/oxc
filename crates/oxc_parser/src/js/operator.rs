use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    precedence::Precedence,
};

use crate::lexer::Kind;

static PRECEDENCE_TABLE: [Option<Precedence>; 256] = {
    let mut table = [None; 256];
    table[Kind::Question2 as usize] = Some(Precedence::NullishCoalescing);
    table[Kind::Pipe2 as usize] = Some(Precedence::LogicalOr);
    table[Kind::Amp2 as usize] = Some(Precedence::LogicalAnd);
    table[Kind::Pipe as usize] = Some(Precedence::BitwiseOr);
    table[Kind::Caret as usize] = Some(Precedence::BitwiseXor);
    table[Kind::Amp as usize] = Some(Precedence::BitwiseAnd);
    table[Kind::Eq2 as usize] = Some(Precedence::Equals);
    table[Kind::Eq3 as usize] = Some(Precedence::Equals);
    table[Kind::Neq as usize] = Some(Precedence::Equals);
    table[Kind::Neq2 as usize] = Some(Precedence::Equals);
    table[Kind::LAngle as usize] = Some(Precedence::Compare);
    table[Kind::RAngle as usize] = Some(Precedence::Compare);
    table[Kind::LtEq as usize] = Some(Precedence::Compare);
    table[Kind::GtEq as usize] = Some(Precedence::Compare);
    table[Kind::Instanceof as usize] = Some(Precedence::Compare);
    table[Kind::In as usize] = Some(Precedence::Compare);
    table[Kind::ShiftLeft as usize] = Some(Precedence::Shift);
    table[Kind::ShiftRight as usize] = Some(Precedence::Shift);
    table[Kind::ShiftRight3 as usize] = Some(Precedence::Shift);
    table[Kind::Plus as usize] = Some(Precedence::Add);
    table[Kind::Minus as usize] = Some(Precedence::Add);
    table[Kind::Star as usize] = Some(Precedence::Multiply);
    table[Kind::Slash as usize] = Some(Precedence::Multiply);
    table[Kind::Percent as usize] = Some(Precedence::Multiply);
    table[Kind::Star2 as usize] = Some(Precedence::Exponentiation);
    table[Kind::As as usize] = Some(Precedence::Compare);
    table[Kind::Satisfies as usize] = Some(Precedence::Compare);
    table
};

#[inline]
pub fn kind_to_precedence(kind: Kind) -> Option<Precedence> {
    PRECEDENCE_TABLE[kind as usize]
}

#[inline]
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

#[inline]
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

#[inline]
pub fn map_logical_operator(kind: Kind) -> LogicalOperator {
    match kind {
        Kind::Pipe2 => LogicalOperator::Or,
        Kind::Amp2 => LogicalOperator::And,
        Kind::Question2 => LogicalOperator::Coalesce,
        _ => unreachable!("Logical Operator: {kind:?}"),
    }
}

#[inline]
pub fn map_update_operator(kind: Kind) -> UpdateOperator {
    match kind {
        Kind::Plus2 => UpdateOperator::Increment,
        Kind::Minus2 => UpdateOperator::Decrement,
        _ => unreachable!("Update Operator: {kind:?}"),
    }
}

#[inline]
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
        _ => unreachable!("Assignment Operator: {kind:?}"),
    }
}
