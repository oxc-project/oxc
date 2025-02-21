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
            Self::Assign => "=".serialize(serializer),
            Self::Addition => "+=".serialize(serializer),
            Self::Subtraction => "-=".serialize(serializer),
            Self::Multiplication => "*=".serialize(serializer),
            Self::Division => "/=".serialize(serializer),
            Self::Remainder => "%=".serialize(serializer),
            Self::Exponential => "**=".serialize(serializer),
            Self::ShiftLeft => "<<=".serialize(serializer),
            Self::ShiftRight => ">>=".serialize(serializer),
            Self::ShiftRightZeroFill => ">>>=".serialize(serializer),
            Self::BitwiseOR => "|=".serialize(serializer),
            Self::BitwiseXOR => "^=".serialize(serializer),
            Self::BitwiseAnd => "&=".serialize(serializer),
            Self::LogicalOr => "||=".serialize(serializer),
            Self::LogicalAnd => "&&=".serialize(serializer),
            Self::LogicalNullish => "??=".serialize(serializer),
        }
    }
}

impl ESTree for BinaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Equality => "==".serialize(serializer),
            Self::Inequality => "!=".serialize(serializer),
            Self::StrictEquality => "===".serialize(serializer),
            Self::StrictInequality => "!==".serialize(serializer),
            Self::LessThan => "<".serialize(serializer),
            Self::LessEqualThan => "<=".serialize(serializer),
            Self::GreaterThan => ">".serialize(serializer),
            Self::GreaterEqualThan => ">=".serialize(serializer),
            Self::Addition => "+".serialize(serializer),
            Self::Subtraction => "-".serialize(serializer),
            Self::Multiplication => "*".serialize(serializer),
            Self::Division => "/".serialize(serializer),
            Self::Remainder => "%".serialize(serializer),
            Self::Exponential => "**".serialize(serializer),
            Self::ShiftLeft => "<<".serialize(serializer),
            Self::ShiftRight => ">>".serialize(serializer),
            Self::ShiftRightZeroFill => ">>>".serialize(serializer),
            Self::BitwiseOR => "|".serialize(serializer),
            Self::BitwiseXOR => "^".serialize(serializer),
            Self::BitwiseAnd => "&".serialize(serializer),
            Self::In => "in".serialize(serializer),
            Self::Instanceof => "instanceof".serialize(serializer),
        }
    }
}

impl ESTree for LogicalOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Or => "||".serialize(serializer),
            Self::And => "&&".serialize(serializer),
            Self::Coalesce => "??".serialize(serializer),
        }
    }
}

impl ESTree for UnaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::UnaryPlus => "+".serialize(serializer),
            Self::UnaryNegation => "-".serialize(serializer),
            Self::LogicalNot => "!".serialize(serializer),
            Self::BitwiseNot => "~".serialize(serializer),
            Self::Typeof => "typeof".serialize(serializer),
            Self::Void => "void".serialize(serializer),
            Self::Delete => "delete".serialize(serializer),
        }
    }
}

impl ESTree for UpdateOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Increment => "++".serialize(serializer),
            Self::Decrement => "--".serialize(serializer),
        }
    }
}
