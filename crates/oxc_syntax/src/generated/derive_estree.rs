// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{__private::ser::FlatMapSerializer, ser::SerializeMap, Serialize, Serializer};

use oxc_estree::ser::{AppendTo, AppendToConcat};

use crate::operator::*;

impl Serialize for AssignmentOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Assign => serializer.serialize_unit_variant("AssignmentOperator", 0, "="),
            Self::Addition => serializer.serialize_unit_variant("AssignmentOperator", 1, "+="),
            Self::Subtraction => serializer.serialize_unit_variant("AssignmentOperator", 2, "-="),
            Self::Multiplication => {
                serializer.serialize_unit_variant("AssignmentOperator", 3, "*=")
            }
            Self::Division => serializer.serialize_unit_variant("AssignmentOperator", 4, "/="),
            Self::Remainder => serializer.serialize_unit_variant("AssignmentOperator", 5, "%="),
            Self::Exponential => serializer.serialize_unit_variant("AssignmentOperator", 6, "**="),
            Self::ShiftLeft => serializer.serialize_unit_variant("AssignmentOperator", 7, "<<="),
            Self::ShiftRight => serializer.serialize_unit_variant("AssignmentOperator", 8, ">>="),
            Self::ShiftRightZeroFill => {
                serializer.serialize_unit_variant("AssignmentOperator", 9, ">>>=")
            }
            Self::BitwiseOR => serializer.serialize_unit_variant("AssignmentOperator", 10, "|="),
            Self::BitwiseXOR => serializer.serialize_unit_variant("AssignmentOperator", 11, "^="),
            Self::BitwiseAnd => serializer.serialize_unit_variant("AssignmentOperator", 12, "&="),
            Self::LogicalOr => serializer.serialize_unit_variant("AssignmentOperator", 13, "||="),
            Self::LogicalAnd => serializer.serialize_unit_variant("AssignmentOperator", 14, "&&="),
            Self::LogicalNullish => {
                serializer.serialize_unit_variant("AssignmentOperator", 15, "??=")
            }
        }
    }
}

impl Serialize for BinaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Equality => serializer.serialize_unit_variant("BinaryOperator", 0, "=="),
            Self::Inequality => serializer.serialize_unit_variant("BinaryOperator", 1, "!="),
            Self::StrictEquality => serializer.serialize_unit_variant("BinaryOperator", 2, "==="),
            Self::StrictInequality => serializer.serialize_unit_variant("BinaryOperator", 3, "!=="),
            Self::LessThan => serializer.serialize_unit_variant("BinaryOperator", 4, "<"),
            Self::LessEqualThan => serializer.serialize_unit_variant("BinaryOperator", 5, "<="),
            Self::GreaterThan => serializer.serialize_unit_variant("BinaryOperator", 6, ">"),
            Self::GreaterEqualThan => serializer.serialize_unit_variant("BinaryOperator", 7, ">="),
            Self::Addition => serializer.serialize_unit_variant("BinaryOperator", 8, "+"),
            Self::Subtraction => serializer.serialize_unit_variant("BinaryOperator", 9, "-"),
            Self::Multiplication => serializer.serialize_unit_variant("BinaryOperator", 10, "*"),
            Self::Division => serializer.serialize_unit_variant("BinaryOperator", 11, "/"),
            Self::Remainder => serializer.serialize_unit_variant("BinaryOperator", 12, "%"),
            Self::Exponential => serializer.serialize_unit_variant("BinaryOperator", 13, "**"),
            Self::ShiftLeft => serializer.serialize_unit_variant("BinaryOperator", 14, "<<"),
            Self::ShiftRight => serializer.serialize_unit_variant("BinaryOperator", 15, ">>"),
            Self::ShiftRightZeroFill => {
                serializer.serialize_unit_variant("BinaryOperator", 16, ">>>")
            }
            Self::BitwiseOR => serializer.serialize_unit_variant("BinaryOperator", 17, "|"),
            Self::BitwiseXOR => serializer.serialize_unit_variant("BinaryOperator", 18, "^"),
            Self::BitwiseAnd => serializer.serialize_unit_variant("BinaryOperator", 19, "&"),
            Self::In => serializer.serialize_unit_variant("BinaryOperator", 20, "in"),
            Self::Instanceof => {
                serializer.serialize_unit_variant("BinaryOperator", 21, "instanceof")
            }
        }
    }
}

impl Serialize for LogicalOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Or => serializer.serialize_unit_variant("LogicalOperator", 0, "||"),
            Self::And => serializer.serialize_unit_variant("LogicalOperator", 1, "&&"),
            Self::Coalesce => serializer.serialize_unit_variant("LogicalOperator", 2, "??"),
        }
    }
}

impl Serialize for UnaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::UnaryPlus => serializer.serialize_unit_variant("UnaryOperator", 0, "+"),
            Self::UnaryNegation => serializer.serialize_unit_variant("UnaryOperator", 1, "-"),
            Self::LogicalNot => serializer.serialize_unit_variant("UnaryOperator", 2, "!"),
            Self::BitwiseNot => serializer.serialize_unit_variant("UnaryOperator", 3, "~"),
            Self::Typeof => serializer.serialize_unit_variant("UnaryOperator", 4, "typeof"),
            Self::Void => serializer.serialize_unit_variant("UnaryOperator", 5, "void"),
            Self::Delete => serializer.serialize_unit_variant("UnaryOperator", 6, "delete"),
        }
    }
}

impl Serialize for UpdateOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Increment => serializer.serialize_unit_variant("UpdateOperator", 0, "++"),
            Self::Decrement => serializer.serialize_unit_variant("UpdateOperator", 1, "--"),
        }
    }
}
