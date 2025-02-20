// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    ser::{AppendTo, AppendToConcat},
    ESTree, FlatStructSerializer, Serializer, StructSerializer,
};

use crate::operator::*;

impl ESTree for AssignmentOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            AssignmentOperator::Assign => "=".serialize(serializer),
            AssignmentOperator::Addition => "+=".serialize(serializer),
            AssignmentOperator::Subtraction => "-=".serialize(serializer),
            AssignmentOperator::Multiplication => "*=".serialize(serializer),
            AssignmentOperator::Division => "/=".serialize(serializer),
            AssignmentOperator::Remainder => "%=".serialize(serializer),
            AssignmentOperator::Exponential => "**=".serialize(serializer),
            AssignmentOperator::ShiftLeft => "<<=".serialize(serializer),
            AssignmentOperator::ShiftRight => ">>=".serialize(serializer),
            AssignmentOperator::ShiftRightZeroFill => ">>>=".serialize(serializer),
            AssignmentOperator::BitwiseOR => "|=".serialize(serializer),
            AssignmentOperator::BitwiseXOR => "^=".serialize(serializer),
            AssignmentOperator::BitwiseAnd => "&=".serialize(serializer),
            AssignmentOperator::LogicalOr => "||=".serialize(serializer),
            AssignmentOperator::LogicalAnd => "&&=".serialize(serializer),
            AssignmentOperator::LogicalNullish => "??=".serialize(serializer),
        }
    }
}

impl ESTree for BinaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            BinaryOperator::Equality => "==".serialize(serializer),
            BinaryOperator::Inequality => "!=".serialize(serializer),
            BinaryOperator::StrictEquality => "===".serialize(serializer),
            BinaryOperator::StrictInequality => "!==".serialize(serializer),
            BinaryOperator::LessThan => "<".serialize(serializer),
            BinaryOperator::LessEqualThan => "<=".serialize(serializer),
            BinaryOperator::GreaterThan => ">".serialize(serializer),
            BinaryOperator::GreaterEqualThan => ">=".serialize(serializer),
            BinaryOperator::Addition => "+".serialize(serializer),
            BinaryOperator::Subtraction => "-".serialize(serializer),
            BinaryOperator::Multiplication => "*".serialize(serializer),
            BinaryOperator::Division => "/".serialize(serializer),
            BinaryOperator::Remainder => "%".serialize(serializer),
            BinaryOperator::Exponential => "**".serialize(serializer),
            BinaryOperator::ShiftLeft => "<<".serialize(serializer),
            BinaryOperator::ShiftRight => ">>".serialize(serializer),
            BinaryOperator::ShiftRightZeroFill => ">>>".serialize(serializer),
            BinaryOperator::BitwiseOR => "|".serialize(serializer),
            BinaryOperator::BitwiseXOR => "^".serialize(serializer),
            BinaryOperator::BitwiseAnd => "&".serialize(serializer),
            BinaryOperator::In => "in".serialize(serializer),
            BinaryOperator::Instanceof => "instanceof".serialize(serializer),
        }
    }
}

impl ESTree for LogicalOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            LogicalOperator::Or => "||".serialize(serializer),
            LogicalOperator::And => "&&".serialize(serializer),
            LogicalOperator::Coalesce => "??".serialize(serializer),
        }
    }
}

impl ESTree for UnaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            UnaryOperator::UnaryPlus => "+".serialize(serializer),
            UnaryOperator::UnaryNegation => "-".serialize(serializer),
            UnaryOperator::LogicalNot => "!".serialize(serializer),
            UnaryOperator::BitwiseNot => "~".serialize(serializer),
            UnaryOperator::Typeof => "typeof".serialize(serializer),
            UnaryOperator::Void => "void".serialize(serializer),
            UnaryOperator::Delete => "delete".serialize(serializer),
        }
    }
}

impl ESTree for UpdateOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            UpdateOperator::Increment => "++".serialize(serializer),
            UpdateOperator::Decrement => "--".serialize(serializer),
        }
    }
}
