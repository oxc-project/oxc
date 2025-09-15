use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    precedence::Precedence,
};

use crate::lexer::Kind;

// Lookup table for operator precedence - indexed by Kind discriminant value
// Size: 256 bytes (assuming Kind enum has up to 256 variants)
const PRECEDENCE_TABLE: [Option<Precedence>; 256] = {
    let mut table = [None; 256];

    // Populate the table with known precedences
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

#[inline(always)]
pub fn kind_to_precedence(kind: Kind) -> Option<Precedence> {
    // Direct array lookup - O(1) operation
    // Safe because Kind discriminant is u8 (max 255)
    PRECEDENCE_TABLE[kind as usize]
}

// Lookup table for binary operators - using a simple array since most operators
// will map to a valid value, making Option unnecessary for this case
const BINARY_OP_TABLE: [Option<BinaryOperator>; 256] = {
    let mut table = [None; 256];

    table[Kind::Eq2 as usize] = Some(BinaryOperator::Equality);
    table[Kind::Neq as usize] = Some(BinaryOperator::Inequality);
    table[Kind::Eq3 as usize] = Some(BinaryOperator::StrictEquality);
    table[Kind::Neq2 as usize] = Some(BinaryOperator::StrictInequality);
    table[Kind::LAngle as usize] = Some(BinaryOperator::LessThan);
    table[Kind::LtEq as usize] = Some(BinaryOperator::LessEqualThan);
    table[Kind::RAngle as usize] = Some(BinaryOperator::GreaterThan);
    table[Kind::GtEq as usize] = Some(BinaryOperator::GreaterEqualThan);
    table[Kind::ShiftLeft as usize] = Some(BinaryOperator::ShiftLeft);
    table[Kind::ShiftRight as usize] = Some(BinaryOperator::ShiftRight);
    table[Kind::ShiftRight3 as usize] = Some(BinaryOperator::ShiftRightZeroFill);
    table[Kind::Plus as usize] = Some(BinaryOperator::Addition);
    table[Kind::Minus as usize] = Some(BinaryOperator::Subtraction);
    table[Kind::Star as usize] = Some(BinaryOperator::Multiplication);
    table[Kind::Slash as usize] = Some(BinaryOperator::Division);
    table[Kind::Percent as usize] = Some(BinaryOperator::Remainder);
    table[Kind::Pipe as usize] = Some(BinaryOperator::BitwiseOR);
    table[Kind::Caret as usize] = Some(BinaryOperator::BitwiseXOR);
    table[Kind::Amp as usize] = Some(BinaryOperator::BitwiseAnd);
    table[Kind::In as usize] = Some(BinaryOperator::In);
    table[Kind::Instanceof as usize] = Some(BinaryOperator::Instanceof);
    table[Kind::Star2 as usize] = Some(BinaryOperator::Exponential);

    table
};

#[inline(always)]
pub fn map_binary_operator(kind: Kind) -> BinaryOperator {
    BINARY_OP_TABLE[kind as usize].unwrap_or_else(|| unreachable!("Binary Operator: {kind:?}"))
}

const UNARY_OP_TABLE: [Option<UnaryOperator>; 256] = {
    let mut table = [None; 256];

    table[Kind::Minus as usize] = Some(UnaryOperator::UnaryNegation);
    table[Kind::Plus as usize] = Some(UnaryOperator::UnaryPlus);
    table[Kind::Bang as usize] = Some(UnaryOperator::LogicalNot);
    table[Kind::Tilde as usize] = Some(UnaryOperator::BitwiseNot);
    table[Kind::Typeof as usize] = Some(UnaryOperator::Typeof);
    table[Kind::Void as usize] = Some(UnaryOperator::Void);
    table[Kind::Delete as usize] = Some(UnaryOperator::Delete);

    table
};

#[inline(always)]
pub fn map_unary_operator(kind: Kind) -> UnaryOperator {
    UNARY_OP_TABLE[kind as usize].unwrap_or_else(|| unreachable!("Unary Operator: {kind:?}"))
}

const LOGICAL_OP_TABLE: [Option<LogicalOperator>; 256] = {
    let mut table = [None; 256];

    table[Kind::Pipe2 as usize] = Some(LogicalOperator::Or);
    table[Kind::Amp2 as usize] = Some(LogicalOperator::And);
    table[Kind::Question2 as usize] = Some(LogicalOperator::Coalesce);

    table
};

#[inline(always)]
pub fn map_logical_operator(kind: Kind) -> LogicalOperator {
    LOGICAL_OP_TABLE[kind as usize].unwrap_or_else(|| unreachable!("Logical Operator: {kind:?}"))
}

const UPDATE_OP_TABLE: [Option<UpdateOperator>; 256] = {
    let mut table = [None; 256];

    table[Kind::Plus2 as usize] = Some(UpdateOperator::Increment);
    table[Kind::Minus2 as usize] = Some(UpdateOperator::Decrement);

    table
};

#[inline(always)]
pub fn map_update_operator(kind: Kind) -> UpdateOperator {
    UPDATE_OP_TABLE[kind as usize].unwrap_or_else(|| unreachable!("Update Operator: {kind:?}"))
}

const ASSIGNMENT_OP_TABLE: [Option<AssignmentOperator>; 256] = {
    let mut table = [None; 256];

    table[Kind::Eq as usize] = Some(AssignmentOperator::Assign);
    table[Kind::PlusEq as usize] = Some(AssignmentOperator::Addition);
    table[Kind::MinusEq as usize] = Some(AssignmentOperator::Subtraction);
    table[Kind::StarEq as usize] = Some(AssignmentOperator::Multiplication);
    table[Kind::SlashEq as usize] = Some(AssignmentOperator::Division);
    table[Kind::PercentEq as usize] = Some(AssignmentOperator::Remainder);
    table[Kind::ShiftLeftEq as usize] = Some(AssignmentOperator::ShiftLeft);
    table[Kind::ShiftRightEq as usize] = Some(AssignmentOperator::ShiftRight);
    table[Kind::ShiftRight3Eq as usize] = Some(AssignmentOperator::ShiftRightZeroFill);
    table[Kind::PipeEq as usize] = Some(AssignmentOperator::BitwiseOR);
    table[Kind::CaretEq as usize] = Some(AssignmentOperator::BitwiseXOR);
    table[Kind::AmpEq as usize] = Some(AssignmentOperator::BitwiseAnd);
    table[Kind::Amp2Eq as usize] = Some(AssignmentOperator::LogicalAnd);
    table[Kind::Pipe2Eq as usize] = Some(AssignmentOperator::LogicalOr);
    table[Kind::Question2Eq as usize] = Some(AssignmentOperator::LogicalNullish);
    table[Kind::Star2Eq as usize] = Some(AssignmentOperator::Exponential);

    table
};

#[inline(always)]
pub fn map_assignment_operator(kind: Kind) -> AssignmentOperator {
    ASSIGNMENT_OP_TABLE[kind as usize]
        .unwrap_or_else(|| unreachable!("Assignment Operator: {kind:?}"))
}
