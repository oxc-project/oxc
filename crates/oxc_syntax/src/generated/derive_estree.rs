// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

#[allow(clippy::wildcard_imports)]
use crate::operator::*;

impl Serialize for AssignmentOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            AssignmentOperator::Assign => {
                serializer.serialize_unit_variant("AssignmentOperator", 0u32, "=")
            }
            AssignmentOperator::Addition => {
                serializer.serialize_unit_variant("AssignmentOperator", 1u32, "+=")
            }
            AssignmentOperator::Subtraction => {
                serializer.serialize_unit_variant("AssignmentOperator", 2u32, "-=")
            }
            AssignmentOperator::Multiplication => {
                serializer.serialize_unit_variant("AssignmentOperator", 3u32, "*=")
            }
            AssignmentOperator::Division => {
                serializer.serialize_unit_variant("AssignmentOperator", 4u32, "/=")
            }
            AssignmentOperator::Remainder => {
                serializer.serialize_unit_variant("AssignmentOperator", 5u32, "%=")
            }
            AssignmentOperator::ShiftLeft => {
                serializer.serialize_unit_variant("AssignmentOperator", 6u32, "<<=")
            }
            AssignmentOperator::ShiftRight => {
                serializer.serialize_unit_variant("AssignmentOperator", 7u32, ">>=")
            }
            AssignmentOperator::ShiftRightZeroFill => {
                serializer.serialize_unit_variant("AssignmentOperator", 8u32, ">>>=")
            }
            AssignmentOperator::BitwiseOR => {
                serializer.serialize_unit_variant("AssignmentOperator", 9u32, "|=")
            }
            AssignmentOperator::BitwiseXOR => {
                serializer.serialize_unit_variant("AssignmentOperator", 10u32, "^=")
            }
            AssignmentOperator::BitwiseAnd => {
                serializer.serialize_unit_variant("AssignmentOperator", 11u32, "&=")
            }
            AssignmentOperator::LogicalAnd => {
                serializer.serialize_unit_variant("AssignmentOperator", 12u32, "&&=")
            }
            AssignmentOperator::LogicalOr => {
                serializer.serialize_unit_variant("AssignmentOperator", 13u32, "||=")
            }
            AssignmentOperator::LogicalNullish => {
                serializer.serialize_unit_variant("AssignmentOperator", 14u32, "??=")
            }
            AssignmentOperator::Exponential => {
                serializer.serialize_unit_variant("AssignmentOperator", 15u32, "**=")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentOperator = '=' | '+=' | '-=' | '*=' | '/=' | '%=' | '<<=' | '>>=' | '>>>=' | '|=' | '^=' | '&=' | '&&=' | '||=' | '??=' | '**=';";

impl Serialize for BinaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            BinaryOperator::Equality => {
                serializer.serialize_unit_variant("BinaryOperator", 0u32, "==")
            }
            BinaryOperator::Inequality => {
                serializer.serialize_unit_variant("BinaryOperator", 1u32, "!=")
            }
            BinaryOperator::StrictEquality => {
                serializer.serialize_unit_variant("BinaryOperator", 2u32, "===")
            }
            BinaryOperator::StrictInequality => {
                serializer.serialize_unit_variant("BinaryOperator", 3u32, "!==")
            }
            BinaryOperator::LessThan => {
                serializer.serialize_unit_variant("BinaryOperator", 4u32, "<")
            }
            BinaryOperator::LessEqualThan => {
                serializer.serialize_unit_variant("BinaryOperator", 5u32, "<=")
            }
            BinaryOperator::GreaterThan => {
                serializer.serialize_unit_variant("BinaryOperator", 6u32, ">")
            }
            BinaryOperator::GreaterEqualThan => {
                serializer.serialize_unit_variant("BinaryOperator", 7u32, ">=")
            }
            BinaryOperator::ShiftLeft => {
                serializer.serialize_unit_variant("BinaryOperator", 8u32, "<<")
            }
            BinaryOperator::ShiftRight => {
                serializer.serialize_unit_variant("BinaryOperator", 9u32, ">>")
            }
            BinaryOperator::ShiftRightZeroFill => {
                serializer.serialize_unit_variant("BinaryOperator", 10u32, ">>>")
            }
            BinaryOperator::Addition => {
                serializer.serialize_unit_variant("BinaryOperator", 11u32, "+")
            }
            BinaryOperator::Subtraction => {
                serializer.serialize_unit_variant("BinaryOperator", 12u32, "-")
            }
            BinaryOperator::Multiplication => {
                serializer.serialize_unit_variant("BinaryOperator", 13u32, "*")
            }
            BinaryOperator::Division => {
                serializer.serialize_unit_variant("BinaryOperator", 14u32, "/")
            }
            BinaryOperator::Remainder => {
                serializer.serialize_unit_variant("BinaryOperator", 15u32, "%")
            }
            BinaryOperator::BitwiseOR => {
                serializer.serialize_unit_variant("BinaryOperator", 16u32, "|")
            }
            BinaryOperator::BitwiseXOR => {
                serializer.serialize_unit_variant("BinaryOperator", 17u32, "^")
            }
            BinaryOperator::BitwiseAnd => {
                serializer.serialize_unit_variant("BinaryOperator", 18u32, "&")
            }
            BinaryOperator::In => serializer.serialize_unit_variant("BinaryOperator", 19u32, "in"),
            BinaryOperator::Instanceof => {
                serializer.serialize_unit_variant("BinaryOperator", 20u32, "instanceof")
            }
            BinaryOperator::Exponential => {
                serializer.serialize_unit_variant("BinaryOperator", 21u32, "**")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BinaryOperator = '==' | '!=' | '===' | '!==' | '<' | '<=' | '>' | '>=' | '<<' | '>>' | '>>>' | '+' | '-' | '*' | '/' | '%' | '|' | '^' | '&' | 'in' | 'instanceof' | '**';";

impl Serialize for LogicalOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            LogicalOperator::Or => serializer.serialize_unit_variant("LogicalOperator", 0u32, "||"),
            LogicalOperator::And => {
                serializer.serialize_unit_variant("LogicalOperator", 1u32, "&&")
            }
            LogicalOperator::Coalesce => {
                serializer.serialize_unit_variant("LogicalOperator", 2u32, "??")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type LogicalOperator = '||' | '&&' | '??';";

impl Serialize for UnaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            UnaryOperator::UnaryNegation => {
                serializer.serialize_unit_variant("UnaryOperator", 0u32, "-")
            }
            UnaryOperator::UnaryPlus => {
                serializer.serialize_unit_variant("UnaryOperator", 1u32, "+")
            }
            UnaryOperator::LogicalNot => {
                serializer.serialize_unit_variant("UnaryOperator", 2u32, "!")
            }
            UnaryOperator::BitwiseNot => {
                serializer.serialize_unit_variant("UnaryOperator", 3u32, "~")
            }
            UnaryOperator::Typeof => {
                serializer.serialize_unit_variant("UnaryOperator", 4u32, "typeof")
            }
            UnaryOperator::Void => serializer.serialize_unit_variant("UnaryOperator", 5u32, "void"),
            UnaryOperator::Delete => {
                serializer.serialize_unit_variant("UnaryOperator", 6u32, "delete")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type UnaryOperator = '-' | '+' | '!' | '~' | 'typeof' | 'void' | 'delete';";

impl Serialize for UpdateOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            UpdateOperator::Increment => {
                serializer.serialize_unit_variant("UpdateOperator", 0u32, "++")
            }
            UpdateOperator::Decrement => {
                serializer.serialize_unit_variant("UpdateOperator", 1u32, "--")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type UpdateOperator = '++' | '--';";
