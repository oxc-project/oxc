// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    ESTree, FlatStructSerializer, JsonSafeString, Serializer, StructSerializer,
    ser::{AppendTo, AppendToConcat},
};

use crate::operator::*;

impl ESTree for AssignmentOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Assign => JsonSafeString("=").serialize(serializer),
            Self::Addition => JsonSafeString("+=").serialize(serializer),
            Self::Subtraction => JsonSafeString("-=").serialize(serializer),
            Self::Multiplication => JsonSafeString("*=").serialize(serializer),
            Self::Division => JsonSafeString("/=").serialize(serializer),
            Self::Remainder => JsonSafeString("%=").serialize(serializer),
            Self::Exponential => JsonSafeString("**=").serialize(serializer),
            Self::ShiftLeft => JsonSafeString("<<=").serialize(serializer),
            Self::ShiftRight => JsonSafeString(">>=").serialize(serializer),
            Self::ShiftRightZeroFill => JsonSafeString(">>>=").serialize(serializer),
            Self::BitwiseOR => JsonSafeString("|=").serialize(serializer),
            Self::BitwiseXOR => JsonSafeString("^=").serialize(serializer),
            Self::BitwiseAnd => JsonSafeString("&=").serialize(serializer),
            Self::LogicalOr => JsonSafeString("||=").serialize(serializer),
            Self::LogicalAnd => JsonSafeString("&&=").serialize(serializer),
            Self::LogicalNullish => JsonSafeString("??=").serialize(serializer),
        }
    }
}

impl ESTree for BinaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Equality => JsonSafeString("==").serialize(serializer),
            Self::Inequality => JsonSafeString("!=").serialize(serializer),
            Self::StrictEquality => JsonSafeString("===").serialize(serializer),
            Self::StrictInequality => JsonSafeString("!==").serialize(serializer),
            Self::LessThan => JsonSafeString("<").serialize(serializer),
            Self::LessEqualThan => JsonSafeString("<=").serialize(serializer),
            Self::GreaterThan => JsonSafeString(">").serialize(serializer),
            Self::GreaterEqualThan => JsonSafeString(">=").serialize(serializer),
            Self::Addition => JsonSafeString("+").serialize(serializer),
            Self::Subtraction => JsonSafeString("-").serialize(serializer),
            Self::Multiplication => JsonSafeString("*").serialize(serializer),
            Self::Division => JsonSafeString("/").serialize(serializer),
            Self::Remainder => JsonSafeString("%").serialize(serializer),
            Self::Exponential => JsonSafeString("**").serialize(serializer),
            Self::ShiftLeft => JsonSafeString("<<").serialize(serializer),
            Self::ShiftRight => JsonSafeString(">>").serialize(serializer),
            Self::ShiftRightZeroFill => JsonSafeString(">>>").serialize(serializer),
            Self::BitwiseOR => JsonSafeString("|").serialize(serializer),
            Self::BitwiseXOR => JsonSafeString("^").serialize(serializer),
            Self::BitwiseAnd => JsonSafeString("&").serialize(serializer),
            Self::In => JsonSafeString("in").serialize(serializer),
            Self::Instanceof => JsonSafeString("instanceof").serialize(serializer),
        }
    }
}

impl ESTree for LogicalOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Or => JsonSafeString("||").serialize(serializer),
            Self::And => JsonSafeString("&&").serialize(serializer),
            Self::Coalesce => JsonSafeString("??").serialize(serializer),
        }
    }
}

impl ESTree for UnaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::UnaryPlus => JsonSafeString("+").serialize(serializer),
            Self::UnaryNegation => JsonSafeString("-").serialize(serializer),
            Self::LogicalNot => JsonSafeString("!").serialize(serializer),
            Self::BitwiseNot => JsonSafeString("~").serialize(serializer),
            Self::Typeof => JsonSafeString("typeof").serialize(serializer),
            Self::Void => JsonSafeString("void").serialize(serializer),
            Self::Delete => JsonSafeString("delete").serialize(serializer),
        }
    }
}

impl ESTree for UpdateOperator {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Increment => JsonSafeString("++").serialize(serializer),
            Self::Decrement => JsonSafeString("--").serialize(serializer),
        }
    }
}
