// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::operator::*;

impl Serialize for AssignmentOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            AssignmentOperator::Assign => {
                serializer.serialize_unit_variant("AssignmentOperator", 0, "=")
            }
            AssignmentOperator::Addition => {
                serializer.serialize_unit_variant("AssignmentOperator", 1, "+=")
            }
            AssignmentOperator::Subtraction => {
                serializer.serialize_unit_variant("AssignmentOperator", 2, "-=")
            }
            AssignmentOperator::Multiplication => {
                serializer.serialize_unit_variant("AssignmentOperator", 3, "*=")
            }
            AssignmentOperator::Division => {
                serializer.serialize_unit_variant("AssignmentOperator", 4, "/=")
            }
            AssignmentOperator::Remainder => {
                serializer.serialize_unit_variant("AssignmentOperator", 5, "%=")
            }
            AssignmentOperator::Exponential => {
                serializer.serialize_unit_variant("AssignmentOperator", 6, "**=")
            }
            AssignmentOperator::ShiftLeft => {
                serializer.serialize_unit_variant("AssignmentOperator", 7, "<<=")
            }
            AssignmentOperator::ShiftRight => {
                serializer.serialize_unit_variant("AssignmentOperator", 8, ">>=")
            }
            AssignmentOperator::ShiftRightZeroFill => {
                serializer.serialize_unit_variant("AssignmentOperator", 9, ">>>=")
            }
            AssignmentOperator::BitwiseOR => {
                serializer.serialize_unit_variant("AssignmentOperator", 10, "|=")
            }
            AssignmentOperator::BitwiseXOR => {
                serializer.serialize_unit_variant("AssignmentOperator", 11, "^=")
            }
            AssignmentOperator::BitwiseAnd => {
                serializer.serialize_unit_variant("AssignmentOperator", 12, "&=")
            }
            AssignmentOperator::LogicalOr => {
                serializer.serialize_unit_variant("AssignmentOperator", 13, "||=")
            }
            AssignmentOperator::LogicalAnd => {
                serializer.serialize_unit_variant("AssignmentOperator", 14, "&&=")
            }
            AssignmentOperator::LogicalNullish => {
                serializer.serialize_unit_variant("AssignmentOperator", 15, "??=")
            }
        }
    }
}

impl Serialize for BinaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            BinaryOperator::Equality => {
                serializer.serialize_unit_variant("BinaryOperator", 0, "==")
            }
            BinaryOperator::Inequality => {
                serializer.serialize_unit_variant("BinaryOperator", 1, "!=")
            }
            BinaryOperator::StrictEquality => {
                serializer.serialize_unit_variant("BinaryOperator", 2, "===")
            }
            BinaryOperator::StrictInequality => {
                serializer.serialize_unit_variant("BinaryOperator", 3, "!==")
            }
            BinaryOperator::LessThan => serializer.serialize_unit_variant("BinaryOperator", 4, "<"),
            BinaryOperator::LessEqualThan => {
                serializer.serialize_unit_variant("BinaryOperator", 5, "<=")
            }
            BinaryOperator::GreaterThan => {
                serializer.serialize_unit_variant("BinaryOperator", 6, ">")
            }
            BinaryOperator::GreaterEqualThan => {
                serializer.serialize_unit_variant("BinaryOperator", 7, ">=")
            }
            BinaryOperator::Addition => serializer.serialize_unit_variant("BinaryOperator", 8, "+"),
            BinaryOperator::Subtraction => {
                serializer.serialize_unit_variant("BinaryOperator", 9, "-")
            }
            BinaryOperator::Multiplication => {
                serializer.serialize_unit_variant("BinaryOperator", 10, "*")
            }
            BinaryOperator::Division => {
                serializer.serialize_unit_variant("BinaryOperator", 11, "/")
            }
            BinaryOperator::Remainder => {
                serializer.serialize_unit_variant("BinaryOperator", 12, "%")
            }
            BinaryOperator::Exponential => {
                serializer.serialize_unit_variant("BinaryOperator", 13, "**")
            }
            BinaryOperator::ShiftLeft => {
                serializer.serialize_unit_variant("BinaryOperator", 14, "<<")
            }
            BinaryOperator::ShiftRight => {
                serializer.serialize_unit_variant("BinaryOperator", 15, ">>")
            }
            BinaryOperator::ShiftRightZeroFill => {
                serializer.serialize_unit_variant("BinaryOperator", 16, ">>>")
            }
            BinaryOperator::BitwiseOR => {
                serializer.serialize_unit_variant("BinaryOperator", 17, "|")
            }
            BinaryOperator::BitwiseXOR => {
                serializer.serialize_unit_variant("BinaryOperator", 18, "^")
            }
            BinaryOperator::BitwiseAnd => {
                serializer.serialize_unit_variant("BinaryOperator", 19, "&")
            }
            BinaryOperator::In => serializer.serialize_unit_variant("BinaryOperator", 20, "in"),
            BinaryOperator::Instanceof => {
                serializer.serialize_unit_variant("BinaryOperator", 21, "instanceof")
            }
        }
    }
}

impl Serialize for LogicalOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            LogicalOperator::Or => serializer.serialize_unit_variant("LogicalOperator", 0, "||"),
            LogicalOperator::And => serializer.serialize_unit_variant("LogicalOperator", 1, "&&"),
            LogicalOperator::Coalesce => {
                serializer.serialize_unit_variant("LogicalOperator", 2, "??")
            }
        }
    }
}

impl Serialize for UnaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            UnaryOperator::UnaryPlus => serializer.serialize_unit_variant("UnaryOperator", 0, "+"),
            UnaryOperator::UnaryNegation => {
                serializer.serialize_unit_variant("UnaryOperator", 1, "-")
            }
            UnaryOperator::LogicalNot => serializer.serialize_unit_variant("UnaryOperator", 2, "!"),
            UnaryOperator::BitwiseNot => serializer.serialize_unit_variant("UnaryOperator", 3, "~"),
            UnaryOperator::Typeof => {
                serializer.serialize_unit_variant("UnaryOperator", 4, "typeof")
            }
            UnaryOperator::Void => serializer.serialize_unit_variant("UnaryOperator", 5, "void"),
            UnaryOperator::Delete => {
                serializer.serialize_unit_variant("UnaryOperator", 6, "delete")
            }
        }
    }
}

impl Serialize for UpdateOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            UpdateOperator::Increment => {
                serializer.serialize_unit_variant("UpdateOperator", 0, "++")
            }
            UpdateOperator::Decrement => {
                serializer.serialize_unit_variant("UpdateOperator", 1, "--")
            }
        }
    }
}
