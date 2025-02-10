// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`

#![allow(clippy::default_trait_access)]

use oxc_allocator::{Allocator, CloneIn};

use crate::number::*;
use crate::operator::*;

impl<'alloc> CloneIn<'alloc> for NumberBase {
    type Cloned = NumberBase;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Float => NumberBase::Float,
            Self::Decimal => NumberBase::Decimal,
            Self::Binary => NumberBase::Binary,
            Self::Octal => NumberBase::Octal,
            Self::Hex => NumberBase::Hex,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for BigintBase {
    type Cloned = BigintBase;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Decimal => BigintBase::Decimal,
            Self::Binary => BigintBase::Binary,
            Self::Octal => BigintBase::Octal,
            Self::Hex => BigintBase::Hex,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for AssignmentOperator {
    type Cloned = AssignmentOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Assign => AssignmentOperator::Assign,
            Self::Addition => AssignmentOperator::Addition,
            Self::Subtraction => AssignmentOperator::Subtraction,
            Self::Multiplication => AssignmentOperator::Multiplication,
            Self::Division => AssignmentOperator::Division,
            Self::Remainder => AssignmentOperator::Remainder,
            Self::Exponential => AssignmentOperator::Exponential,
            Self::ShiftLeft => AssignmentOperator::ShiftLeft,
            Self::ShiftRight => AssignmentOperator::ShiftRight,
            Self::ShiftRightZeroFill => AssignmentOperator::ShiftRightZeroFill,
            Self::BitwiseOR => AssignmentOperator::BitwiseOR,
            Self::BitwiseXOR => AssignmentOperator::BitwiseXOR,
            Self::BitwiseAnd => AssignmentOperator::BitwiseAnd,
            Self::LogicalOr => AssignmentOperator::LogicalOr,
            Self::LogicalAnd => AssignmentOperator::LogicalAnd,
            Self::LogicalNullish => AssignmentOperator::LogicalNullish,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for BinaryOperator {
    type Cloned = BinaryOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Equality => BinaryOperator::Equality,
            Self::Inequality => BinaryOperator::Inequality,
            Self::StrictEquality => BinaryOperator::StrictEquality,
            Self::StrictInequality => BinaryOperator::StrictInequality,
            Self::LessThan => BinaryOperator::LessThan,
            Self::LessEqualThan => BinaryOperator::LessEqualThan,
            Self::GreaterThan => BinaryOperator::GreaterThan,
            Self::GreaterEqualThan => BinaryOperator::GreaterEqualThan,
            Self::Addition => BinaryOperator::Addition,
            Self::Subtraction => BinaryOperator::Subtraction,
            Self::Multiplication => BinaryOperator::Multiplication,
            Self::Division => BinaryOperator::Division,
            Self::Remainder => BinaryOperator::Remainder,
            Self::Exponential => BinaryOperator::Exponential,
            Self::ShiftLeft => BinaryOperator::ShiftLeft,
            Self::ShiftRight => BinaryOperator::ShiftRight,
            Self::ShiftRightZeroFill => BinaryOperator::ShiftRightZeroFill,
            Self::BitwiseOR => BinaryOperator::BitwiseOR,
            Self::BitwiseXOR => BinaryOperator::BitwiseXOR,
            Self::BitwiseAnd => BinaryOperator::BitwiseAnd,
            Self::In => BinaryOperator::In,
            Self::Instanceof => BinaryOperator::Instanceof,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for LogicalOperator {
    type Cloned = LogicalOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Or => LogicalOperator::Or,
            Self::And => LogicalOperator::And,
            Self::Coalesce => LogicalOperator::Coalesce,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for UnaryOperator {
    type Cloned = UnaryOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::UnaryPlus => UnaryOperator::UnaryPlus,
            Self::UnaryNegation => UnaryOperator::UnaryNegation,
            Self::LogicalNot => UnaryOperator::LogicalNot,
            Self::BitwiseNot => UnaryOperator::BitwiseNot,
            Self::Typeof => UnaryOperator::Typeof,
            Self::Void => UnaryOperator::Void,
            Self::Delete => UnaryOperator::Delete,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for UpdateOperator {
    type Cloned = UpdateOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Increment => UpdateOperator::Increment,
            Self::Decrement => UpdateOperator::Decrement,
        }
    }
}
