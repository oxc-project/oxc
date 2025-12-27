// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/from_estree.rs`.

#![allow(
    unused_imports,
    clippy::match_same_arms,
    clippy::semicolon_if_nothing_returned,
    clippy::too_many_lines
)]

use crate::deserialize::{
    DeserError, DeserResult, ESTreeField, ESTreeType, FromESTree, parse_span, parse_span_or_empty,
};
use oxc_allocator::{Allocator, Box as ABox, Vec as AVec};

use crate::ast::comment::*;
use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl<'a> FromESTree<'a> for Program<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let source_type = FromESTree::from_estree(json.estree_field("sourceType")?, allocator)?;
        let source_text = Default::default();
        let comments = AVec::new_in(allocator);
        let hashbang = match json.estree_field_opt("hashbang") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let directives = AVec::new_in(allocator);
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(Program {
            span,
            source_type,
            source_text,
            comments,
            hashbang,
            directives,
            body,
            scope_id,
        })
    }
}

impl<'a> FromESTree<'a> for Expression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Super" => {
                Ok(Self::Super(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "TaggedTemplateExpression" => Ok(Self::TaggedTemplateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::JSXElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::NullLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::RegExpLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "V8IntrinsicExpression" => Ok(Self::V8IntrinsicExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MetaProperty" => Ok(Self::MetaProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AwaitExpression" => Ok(Self::AwaitExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ChainExpression" => Ok(Self::ChainExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SequenceExpression" => Ok(Self::SequenceExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrowFunctionExpression" => Ok(Self::ArrowFunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "JSXFragment" => Ok(Self::JSXFragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ConditionalExpression" => Ok(Self::ConditionalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "LogicalExpression" => Ok(Self::LogicalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentExpression" => Ok(Self::AssignmentExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Function" => Ok(Self::FunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ParenthesizedExpression" => Ok(Self::ParenthesizedExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ObjectExpression" => Ok(Self::ObjectExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UpdateExpression" => Ok(Self::UpdateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInstantiationExpression" => Ok(Self::TSInstantiationExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayExpression" => Ok(Self::ArrayExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportExpression" => Ok(Self::ImportExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "NewExpression" => Ok(Self::NewExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "YieldExpression" => Ok(Self::YieldExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BinaryExpression" => {
                let operator = json.get("operator").and_then(|v| v.as_str());
                let left_type =
                    json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if operator == Some("in") && left_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateInExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BinaryExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for IdentifierName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        Ok(IdentifierName { span, name })
    }
}

impl<'a> FromESTree<'a> for IdentifierReference<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        let reference_id = std::cell::Cell::default();
        Ok(IdentifierReference { span, name, reference_id })
    }
}

impl<'a> FromESTree<'a> for BindingIdentifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        let symbol_id = std::cell::Cell::default();
        Ok(BindingIdentifier { span, name, symbol_id })
    }
}

impl<'a> FromESTree<'a> for LabelIdentifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        Ok(LabelIdentifier { span, name })
    }
}

impl<'a> FromESTree<'a> for ThisExpression {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(ThisExpression { span })
    }
}

impl<'a> FromESTree<'a> for ArrayExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let elements = FromESTree::from_estree(json.estree_field("elements")?, allocator)?;
        Ok(ArrayExpression { span, elements })
    }
}

impl<'a> FromESTree<'a> for ArrayExpressionElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "JSXFragment" => Ok(Self::JSXFragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SpreadElement" => Ok(Self::SpreadElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayExpression" => Ok(Self::ArrayExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "LogicalExpression" => Ok(Self::LogicalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Function" => Ok(Self::FunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TaggedTemplateExpression" => Ok(Self::TaggedTemplateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "NewExpression" => Ok(Self::NewExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ConditionalExpression" => Ok(Self::ConditionalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Super" => {
                Ok(Self::Super(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "MetaProperty" => Ok(Self::MetaProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::JSXElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UpdateExpression" => Ok(Self::UpdateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::NullLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::RegExpLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "AwaitExpression" => Ok(Self::AwaitExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "YieldExpression" => Ok(Self::YieldExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "V8IntrinsicExpression" => Ok(Self::V8IntrinsicExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentExpression" => Ok(Self::AssignmentExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ParenthesizedExpression" => Ok(Self::ParenthesizedExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ObjectExpression" => Ok(Self::ObjectExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInstantiationExpression" => Ok(Self::TSInstantiationExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BinaryExpression" => {
                let operator = json.get("operator").and_then(|v| v.as_str());
                let left_type =
                    json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if operator == Some("in") && left_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateInExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BinaryExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ChainExpression" => Ok(Self::ChainExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Elision" => Ok(Self::Elision(FromESTree::from_estree(json, allocator)?)),
            "ArrowFunctionExpression" => Ok(Self::ArrowFunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportExpression" => Ok(Self::ImportExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SequenceExpression" => Ok(Self::SequenceExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for Elision {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(Elision { span })
    }
}

impl<'a> FromESTree<'a> for ObjectExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let properties = FromESTree::from_estree(json.estree_field("properties")?, allocator)?;
        Ok(ObjectExpression { span, properties })
    }
}

impl<'a> FromESTree<'a> for ObjectPropertyKind<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "SpreadElement" => Ok(Self::SpreadProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Property" => Ok(Self::ObjectProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ObjectProperty<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let kind = FromESTree::from_estree(json.estree_field("kind")?, allocator)?;
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let method = FromESTree::from_estree(json.estree_field("method")?, allocator)?;
        let shorthand = FromESTree::from_estree(json.estree_field("shorthand")?, allocator)?;
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        Ok(ObjectProperty { span, kind, key, value, method, shorthand, computed })
    }
}

impl<'a> FromESTree<'a> for PropertyKey<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TaggedTemplateExpression" => Ok(Self::TaggedTemplateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ObjectExpression" => Ok(Self::ObjectExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SequenceExpression" => Ok(Self::SequenceExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Function" => Ok(Self::FunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "NewExpression" => Ok(Self::NewExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::NullLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::RegExpLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "ParenthesizedExpression" => Ok(Self::ParenthesizedExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ConditionalExpression" => Ok(Self::ConditionalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "V8IntrinsicExpression" => Ok(Self::V8IntrinsicExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "LogicalExpression" => Ok(Self::LogicalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AwaitExpression" => Ok(Self::AwaitExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UpdateExpression" => Ok(Self::UpdateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "PrivateIdentifier" => Ok(Self::PrivateIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportExpression" => Ok(Self::ImportExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInstantiationExpression" => Ok(Self::TSInstantiationExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Super" => {
                Ok(Self::Super(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "YieldExpression" => Ok(Self::YieldExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ChainExpression" => Ok(Self::ChainExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BinaryExpression" => {
                let operator = json.get("operator").and_then(|v| v.as_str());
                let left_type =
                    json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if operator == Some("in") && left_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateInExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BinaryExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "MetaProperty" => Ok(Self::MetaProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::StaticIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXFragment" => Ok(Self::JSXFragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrowFunctionExpression" => Ok(Self::ArrowFunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayExpression" => Ok(Self::ArrayExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentExpression" => Ok(Self::AssignmentExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::JSXElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for PropertyKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "init" => Ok(Self::Init),
            "get" => Ok(Self::Get),
            "set" => Ok(Self::Set),
            other => Err(DeserError::InvalidFieldValue("PropertyKind", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TemplateLiteral<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let quasis = FromESTree::from_estree(json.estree_field("quasis")?, allocator)?;
        let expressions = FromESTree::from_estree(json.estree_field("expressions")?, allocator)?;
        Ok(TemplateLiteral { span, quasis, expressions })
    }
}

impl<'a> FromESTree<'a> for TaggedTemplateExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let tag = FromESTree::from_estree(json.estree_field("tag")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let quasi = FromESTree::from_estree(json.estree_field("quasi")?, allocator)?;
        Ok(TaggedTemplateExpression { span, tag, type_arguments, quasi })
    }
}

impl<'a> FromESTree<'a> for TemplateElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let tail = FromESTree::from_estree(json.estree_field("tail")?, allocator)?;
        let lone_surrogates = false;
        Ok(TemplateElement { span, value, tail, lone_surrogates })
    }
}

impl<'a> FromESTree<'a> for TemplateElementValue<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let raw = FromESTree::from_estree(json.estree_field("raw")?, allocator)?;
        let cooked = match json.estree_field_opt("cooked") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TemplateElementValue { raw, cooked })
    }
}

impl<'a> FromESTree<'a> for MemberExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ComputedMemberExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let object = FromESTree::from_estree(json.estree_field("object")?, allocator)?;
        let expression = FromESTree::from_estree(json.estree_field("property")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        Ok(ComputedMemberExpression { span, object, expression, optional })
    }
}

impl<'a> FromESTree<'a> for StaticMemberExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let object = FromESTree::from_estree(json.estree_field("object")?, allocator)?;
        let property = FromESTree::from_estree(json.estree_field("property")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        Ok(StaticMemberExpression { span, object, property, optional })
    }
}

impl<'a> FromESTree<'a> for PrivateFieldExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let object = FromESTree::from_estree(json.estree_field("object")?, allocator)?;
        let field = FromESTree::from_estree(json.estree_field("property")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        Ok(PrivateFieldExpression { span, object, field, optional })
    }
}

impl<'a> FromESTree<'a> for CallExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let callee = FromESTree::from_estree(json.estree_field("callee")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let arguments = FromESTree::from_estree(json.estree_field("arguments")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        let pure = false;
        Ok(CallExpression { span, callee, type_arguments, arguments, optional, pure })
    }
}

impl<'a> FromESTree<'a> for NewExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let callee = FromESTree::from_estree(json.estree_field("callee")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let arguments = FromESTree::from_estree(json.estree_field("arguments")?, allocator)?;
        let pure = false;
        Ok(NewExpression { span, callee, type_arguments, arguments, pure })
    }
}

impl<'a> FromESTree<'a> for MetaProperty<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let meta = FromESTree::from_estree(json.estree_field("meta")?, allocator)?;
        let property = FromESTree::from_estree(json.estree_field("property")?, allocator)?;
        Ok(MetaProperty { span, meta, property })
    }
}

impl<'a> FromESTree<'a> for SpreadElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let argument = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(SpreadElement { span, argument })
    }
}

impl<'a> FromESTree<'a> for Argument<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "BinaryExpression" => {
                let operator = json.get("operator").and_then(|v| v.as_str());
                let left_type =
                    json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if operator == Some("in") && left_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateInExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BinaryExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "UpdateExpression" => Ok(Self::UpdateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MetaProperty" => Ok(Self::MetaProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TaggedTemplateExpression" => Ok(Self::TaggedTemplateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SequenceExpression" => Ok(Self::SequenceExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::JSXElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AwaitExpression" => Ok(Self::AwaitExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "YieldExpression" => Ok(Self::YieldExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportExpression" => Ok(Self::ImportExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXFragment" => Ok(Self::JSXFragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ParenthesizedExpression" => Ok(Self::ParenthesizedExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInstantiationExpression" => Ok(Self::TSInstantiationExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SpreadElement" => Ok(Self::SpreadElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ObjectExpression" => Ok(Self::ObjectExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayExpression" => Ok(Self::ArrayExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentExpression" => Ok(Self::AssignmentExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "NewExpression" => Ok(Self::NewExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrowFunctionExpression" => Ok(Self::ArrowFunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "LogicalExpression" => Ok(Self::LogicalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ConditionalExpression" => Ok(Self::ConditionalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Function" => Ok(Self::FunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::NullLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::RegExpLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "Super" => {
                Ok(Self::Super(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "ChainExpression" => Ok(Self::ChainExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "V8IntrinsicExpression" => Ok(Self::V8IntrinsicExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for UpdateExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let operator = FromESTree::from_estree(json.estree_field("operator")?, allocator)?;
        let prefix = FromESTree::from_estree(json.estree_field("prefix")?, allocator)?;
        let argument = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(UpdateExpression { span, operator, prefix, argument })
    }
}

impl<'a> FromESTree<'a> for UnaryExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let operator = FromESTree::from_estree(json.estree_field("operator")?, allocator)?;
        let argument = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(UnaryExpression { span, operator, argument })
    }
}

impl<'a> FromESTree<'a> for BinaryExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let operator = FromESTree::from_estree(json.estree_field("operator")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(BinaryExpression { span, left, operator, right })
    }
}

impl<'a> FromESTree<'a> for PrivateInExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(PrivateInExpression { span, left, right })
    }
}

impl<'a> FromESTree<'a> for LogicalExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let operator = FromESTree::from_estree(json.estree_field("operator")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(LogicalExpression { span, left, operator, right })
    }
}

impl<'a> FromESTree<'a> for ConditionalExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let test = FromESTree::from_estree(json.estree_field("test")?, allocator)?;
        let consequent = FromESTree::from_estree(json.estree_field("consequent")?, allocator)?;
        let alternate = FromESTree::from_estree(json.estree_field("alternate")?, allocator)?;
        Ok(ConditionalExpression { span, test, consequent, alternate })
    }
}

impl<'a> FromESTree<'a> for AssignmentExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let operator = FromESTree::from_estree(json.estree_field("operator")?, allocator)?;
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(AssignmentExpression { span, operator, left, right })
    }
}

impl<'a> FromESTree<'a> for AssignmentTarget<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Identifier" => Ok(Self::AssignmentTargetIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayPattern" => Ok(Self::ArrayAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "ObjectPattern" => Ok(Self::ObjectAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for SimpleAssignmentTarget<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::AssignmentTargetIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for AssignmentTargetPattern<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ObjectPattern" => Ok(Self::ObjectAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayPattern" => Ok(Self::ArrayAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ArrayAssignmentTarget<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let elements = FromESTree::from_estree(json.estree_field("elements")?, allocator)?;
        let rest = None;
        Ok(ArrayAssignmentTarget { span, elements, rest })
    }
}

impl<'a> FromESTree<'a> for ObjectAssignmentTarget<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let properties = FromESTree::from_estree(json.estree_field("properties")?, allocator)?;
        let rest = None;
        Ok(ObjectAssignmentTarget { span, properties, rest })
    }
}

impl<'a> FromESTree<'a> for AssignmentTargetRest<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let target = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(AssignmentTargetRest { span, target })
    }
}

impl<'a> FromESTree<'a> for AssignmentTargetMaybeDefault<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ObjectPattern" => Ok(Self::ObjectAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::AssignmentTargetIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayPattern" => Ok(Self::ArrayAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentPattern" => Ok(Self::AssignmentTargetWithDefault(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for AssignmentTargetWithDefault<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let binding = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let init = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(AssignmentTargetWithDefault { span, binding, init })
    }
}

impl<'a> FromESTree<'a> for AssignmentTargetProperty<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Property" => Ok(Self::AssignmentTargetPropertyIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for AssignmentTargetPropertyIdentifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let binding = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let init = match json.estree_field_opt("value") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(AssignmentTargetPropertyIdentifier { span, binding, init })
    }
}

impl<'a> FromESTree<'a> for AssignmentTargetPropertyProperty<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let binding = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        Ok(AssignmentTargetPropertyProperty { span, name, binding, computed })
    }
}

impl<'a> FromESTree<'a> for SequenceExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expressions = FromESTree::from_estree(json.estree_field("expressions")?, allocator)?;
        Ok(SequenceExpression { span, expressions })
    }
}

impl<'a> FromESTree<'a> for Super {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(Super { span })
    }
}

impl<'a> FromESTree<'a> for AwaitExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let argument = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(AwaitExpression { span, argument })
    }
}

impl<'a> FromESTree<'a> for ChainExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(ChainExpression { span, expression })
    }
}

impl<'a> FromESTree<'a> for ChainElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ParenthesizedExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(ParenthesizedExpression { span, expression })
    }
}

impl<'a> FromESTree<'a> for Statement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "EmptyStatement" => Ok(Self::EmptyStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ForStatement" => Ok(Self::ForStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Function" => Ok(Self::FunctionDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BlockStatement" => Ok(Self::BlockStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BreakStatement" => Ok(Self::BreakStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ForOfStatement" => Ok(Self::ForOfStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ForInStatement" => Ok(Self::ForInStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TryStatement" => Ok(Self::TryStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "WithStatement" => Ok(Self::WithStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSEnumDeclaration" => Ok(Self::TSEnumDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ExportDefaultDeclaration" => Ok(Self::ExportDefaultDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "DebuggerStatement" => Ok(Self::DebuggerStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "DoWhileStatement" => Ok(Self::DoWhileStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ExpressionStatement" => Ok(Self::ExpressionStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSImportEqualsDeclaration" => Ok(Self::TSImportEqualsDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "WhileStatement" => Ok(Self::WhileStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "VariableDeclaration" => Ok(Self::VariableDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ExportNamedDeclaration" => Ok(Self::ExportNamedDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSModuleDeclaration" => Ok(Self::TSModuleDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportDeclaration" => Ok(Self::ImportDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "IfStatement" => Ok(Self::IfStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ReturnStatement" => Ok(Self::ReturnStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ContinueStatement" => Ok(Self::ContinueStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNamespaceExportDeclaration" => Ok(Self::TSNamespaceExportDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInterfaceDeclaration" => Ok(Self::TSInterfaceDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThrowStatement" => Ok(Self::ThrowStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "LabeledStatement" => Ok(Self::LabeledStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SwitchStatement" => Ok(Self::SwitchStatement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ExportAllDeclaration" => Ok(Self::ExportAllDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSExportAssignment" => Ok(Self::TSExportAssignment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAliasDeclaration" => Ok(Self::TSTypeAliasDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for Directive<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        let directive = FromESTree::from_estree(json.estree_field("directive")?, allocator)?;
        Ok(Directive { span, expression, directive })
    }
}

impl<'a> FromESTree<'a> for Hashbang<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        Ok(Hashbang { span, value })
    }
}

impl<'a> FromESTree<'a> for BlockStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(BlockStatement { span, body, scope_id })
    }
}

impl<'a> FromESTree<'a> for Declaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Function" => Ok(Self::FunctionDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "VariableDeclaration" => Ok(Self::VariableDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSModuleDeclaration" => Ok(Self::TSModuleDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInterfaceDeclaration" => Ok(Self::TSInterfaceDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSEnumDeclaration" => Ok(Self::TSEnumDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAliasDeclaration" => Ok(Self::TSTypeAliasDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSImportEqualsDeclaration" => Ok(Self::TSImportEqualsDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for VariableDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let kind = FromESTree::from_estree(json.estree_field("kind")?, allocator)?;
        let declarations = FromESTree::from_estree(json.estree_field("declarations")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        Ok(VariableDeclaration { span, kind, declarations, declare })
    }
}

impl<'a> FromESTree<'a> for VariableDeclarationKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "var" => Ok(Self::Var),
            "let" => Ok(Self::Let),
            "const" => Ok(Self::Const),
            "using" => Ok(Self::Using),
            "await using" => Ok(Self::AwaitUsing),
            other => {
                Err(DeserError::InvalidFieldValue("VariableDeclarationKind", other.to_string()))
            }
        }
    }
}

impl<'a> FromESTree<'a> for VariableDeclarator<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let kind = crate::ast::js::VariableDeclarationKind::Var;
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        let type_annotation = None;
        let init = match json.estree_field_opt("init") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let definite = FromESTree::from_estree(json.estree_field("definite")?, allocator)?;
        Ok(VariableDeclarator { span, kind, id, type_annotation, init, definite })
    }
}

impl<'a> FromESTree<'a> for EmptyStatement {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(EmptyStatement { span })
    }
}

impl<'a> FromESTree<'a> for ExpressionStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(ExpressionStatement { span, expression })
    }
}

impl<'a> FromESTree<'a> for IfStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let test = FromESTree::from_estree(json.estree_field("test")?, allocator)?;
        let consequent = FromESTree::from_estree(json.estree_field("consequent")?, allocator)?;
        let alternate = match json.estree_field_opt("alternate") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(IfStatement { span, test, consequent, alternate })
    }
}

impl<'a> FromESTree<'a> for DoWhileStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let test = FromESTree::from_estree(json.estree_field("test")?, allocator)?;
        Ok(DoWhileStatement { span, body, test })
    }
}

impl<'a> FromESTree<'a> for WhileStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let test = FromESTree::from_estree(json.estree_field("test")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        Ok(WhileStatement { span, test, body })
    }
}

impl<'a> FromESTree<'a> for ForStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let init = match json.estree_field_opt("init") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let test = match json.estree_field_opt("test") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let update = match json.estree_field_opt("update") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(ForStatement { span, init, test, update, body, scope_id })
    }
}

impl<'a> FromESTree<'a> for ForStatementInit<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "NewExpression" => Ok(Self::NewExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrowFunctionExpression" => Ok(Self::ArrowFunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::JSXElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::NullLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::RegExpLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "LogicalExpression" => Ok(Self::LogicalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentExpression" => Ok(Self::AssignmentExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TaggedTemplateExpression" => Ok(Self::TaggedTemplateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXFragment" => Ok(Self::JSXFragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "V8IntrinsicExpression" => Ok(Self::V8IntrinsicExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInstantiationExpression" => Ok(Self::TSInstantiationExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "YieldExpression" => Ok(Self::YieldExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ChainExpression" => Ok(Self::ChainExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Function" => Ok(Self::FunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportExpression" => Ok(Self::ImportExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BinaryExpression" => {
                let operator = json.get("operator").and_then(|v| v.as_str());
                let left_type =
                    json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if operator == Some("in") && left_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateInExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BinaryExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "UpdateExpression" => Ok(Self::UpdateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ConditionalExpression" => Ok(Self::ConditionalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayExpression" => Ok(Self::ArrayExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SequenceExpression" => Ok(Self::SequenceExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Super" => {
                Ok(Self::Super(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "ObjectExpression" => Ok(Self::ObjectExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ParenthesizedExpression" => Ok(Self::ParenthesizedExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "VariableDeclaration" => Ok(Self::VariableDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AwaitExpression" => Ok(Self::AwaitExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MetaProperty" => Ok(Self::MetaProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ForInStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(ForInStatement { span, left, right, body, scope_id })
    }
}

impl<'a> FromESTree<'a> for ForStatementLeft<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "VariableDeclaration" => Ok(Self::VariableDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayPattern" => Ok(Self::ArrayAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "ObjectPattern" => Ok(Self::ObjectAssignmentTarget(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::AssignmentTargetIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ForOfStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let r#await = FromESTree::from_estree(json.estree_field("await")?, allocator)?;
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(ForOfStatement { span, r#await, left, right, body, scope_id })
    }
}

impl<'a> FromESTree<'a> for ContinueStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let label = match json.estree_field_opt("label") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(ContinueStatement { span, label })
    }
}

impl<'a> FromESTree<'a> for BreakStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let label = match json.estree_field_opt("label") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(BreakStatement { span, label })
    }
}

impl<'a> FromESTree<'a> for ReturnStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let argument = match json.estree_field_opt("argument") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(ReturnStatement { span, argument })
    }
}

impl<'a> FromESTree<'a> for WithStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let object = FromESTree::from_estree(json.estree_field("object")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(WithStatement { span, object, body, scope_id })
    }
}

impl<'a> FromESTree<'a> for SwitchStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let discriminant = FromESTree::from_estree(json.estree_field("discriminant")?, allocator)?;
        let cases = FromESTree::from_estree(json.estree_field("cases")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(SwitchStatement { span, discriminant, cases, scope_id })
    }
}

impl<'a> FromESTree<'a> for SwitchCase<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let test = match json.estree_field_opt("test") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let consequent = FromESTree::from_estree(json.estree_field("consequent")?, allocator)?;
        Ok(SwitchCase { span, test, consequent })
    }
}

impl<'a> FromESTree<'a> for LabeledStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let label = FromESTree::from_estree(json.estree_field("label")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        Ok(LabeledStatement { span, label, body })
    }
}

impl<'a> FromESTree<'a> for ThrowStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let argument = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(ThrowStatement { span, argument })
    }
}

impl<'a> FromESTree<'a> for TryStatement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let block = FromESTree::from_estree(json.estree_field("block")?, allocator)?;
        let handler = match json.estree_field_opt("handler") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let finalizer = match json.estree_field_opt("finalizer") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TryStatement { span, block, handler, finalizer })
    }
}

impl<'a> FromESTree<'a> for CatchClause<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let param = match json.estree_field_opt("param") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(CatchClause { span, param, body, scope_id })
    }
}

impl<'a> FromESTree<'a> for CatchParameter<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let pattern = FromESTree::from_estree(json.estree_field("pattern")?, allocator)?;
        let type_annotation = None;
        Ok(CatchParameter { span, pattern, type_annotation })
    }
}

impl<'a> FromESTree<'a> for DebuggerStatement {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(DebuggerStatement { span })
    }
}

impl<'a> FromESTree<'a> for BindingPattern<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ObjectPattern" => Ok(Self::ObjectPattern(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayPattern" => Ok(Self::ArrayPattern(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::BindingIdentifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentPattern" => Ok(Self::AssignmentPattern(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for AssignmentPattern<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(AssignmentPattern { span, left, right })
    }
}

impl<'a> FromESTree<'a> for ObjectPattern<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let properties = FromESTree::from_estree(json.estree_field("properties")?, allocator)?;
        let rest = None;
        Ok(ObjectPattern { span, properties, rest })
    }
}

impl<'a> FromESTree<'a> for BindingProperty<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let shorthand = FromESTree::from_estree(json.estree_field("shorthand")?, allocator)?;
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        Ok(BindingProperty { span, key, value, shorthand, computed })
    }
}

impl<'a> FromESTree<'a> for ArrayPattern<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let elements = FromESTree::from_estree(json.estree_field("elements")?, allocator)?;
        let rest = None;
        Ok(ArrayPattern { span, elements, rest })
    }
}

impl<'a> FromESTree<'a> for BindingRestElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let argument = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(BindingRestElement { span, argument })
    }
}

impl<'a> FromESTree<'a> for Function<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let r#type = FromESTree::from_estree(json.estree_field("type")?, allocator)?;
        let id = match json.estree_field_opt("id") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let generator = FromESTree::from_estree(json.estree_field("generator")?, allocator)?;
        let r#async = FromESTree::from_estree(json.estree_field("async")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let this_param = None;
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        let return_type = match json.estree_field_opt("returnType") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let body = match json.estree_field_opt("body") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let scope_id = std::cell::Cell::default();
        let pure = false;
        let pife = false;
        Ok(Function {
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            scope_id,
            pure,
            pife,
        })
    }
}

impl<'a> FromESTree<'a> for FunctionType {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "FunctionDeclaration" => Ok(Self::FunctionDeclaration),
            "FunctionExpression" => Ok(Self::FunctionExpression),
            "TSDeclareFunction" => Ok(Self::TSDeclareFunction),
            "TSEmptyBodyFunctionExpression" => Ok(Self::TSEmptyBodyFunctionExpression),
            other => Err(DeserError::InvalidFieldValue("FunctionType", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for FormalParameters<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let kind = FromESTree::from_estree(json.estree_field("kind")?, allocator)?;
        let items = FromESTree::from_estree(json.estree_field("items")?, allocator)?;
        let rest = None;
        Ok(FormalParameters { span, kind, items, rest })
    }
}

impl<'a> FromESTree<'a> for FormalParameter<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let decorators = FromESTree::from_estree(json.estree_field("decorators")?, allocator)?;
        let pattern = FromESTree::from_estree(json.estree_field("pattern")?, allocator)?;
        let type_annotation = match json.estree_field_opt("typeAnnotation") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let initializer = match json.estree_field_opt("initializer") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        let accessibility = None;
        let readonly = false;
        let r#override = false;
        Ok(FormalParameter {
            span,
            decorators,
            pattern,
            type_annotation,
            initializer,
            optional,
            accessibility,
            readonly,
            r#override,
        })
    }
}

impl<'a> FromESTree<'a> for FormalParameterKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "FormalParameter" => Ok(Self::FormalParameter),
            "UniqueFormalParameters" => Ok(Self::UniqueFormalParameters),
            "ArrowFormalParameters" => Ok(Self::ArrowFormalParameters),
            "Signature" => Ok(Self::Signature),
            other => Err(DeserError::InvalidFieldValue("FormalParameterKind", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for FunctionBody<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let directives = AVec::new_in(allocator);
        let statements = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        Ok(FunctionBody { span, directives, statements })
    }
}

impl<'a> FromESTree<'a> for ArrowFunctionExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        let r#async = FromESTree::from_estree(json.estree_field("async")?, allocator)?;
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        let return_type = match json.estree_field_opt("returnType") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        let pure = false;
        let pife = false;
        Ok(ArrowFunctionExpression {
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            scope_id,
            pure,
            pife,
        })
    }
}

impl<'a> FromESTree<'a> for YieldExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let delegate = FromESTree::from_estree(json.estree_field("delegate")?, allocator)?;
        let argument = match json.estree_field_opt("argument") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(YieldExpression { span, delegate, argument })
    }
}

impl<'a> FromESTree<'a> for Class<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let r#type = FromESTree::from_estree(json.estree_field("type")?, allocator)?;
        let decorators = FromESTree::from_estree(json.estree_field("decorators")?, allocator)?;
        let id = match json.estree_field_opt("id") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let super_class = match json.estree_field_opt("superClass") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let super_type_arguments = match json.estree_field_opt("superTypeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let implements = FromESTree::from_estree(json.estree_field("implements")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let r#abstract = FromESTree::from_estree(json.estree_field("abstract")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(Class {
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
        })
    }
}

impl<'a> FromESTree<'a> for ClassType {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "ClassDeclaration" => Ok(Self::ClassDeclaration),
            "ClassExpression" => Ok(Self::ClassExpression),
            other => Err(DeserError::InvalidFieldValue("ClassType", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ClassBody<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        Ok(ClassBody { span, body })
    }
}

impl<'a> FromESTree<'a> for ClassElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "PropertyDefinition" => Ok(Self::PropertyDefinition(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "StaticBlock" => Ok(Self::StaticBlock(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AccessorProperty" => Ok(Self::AccessorProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MethodDefinition" => Ok(Self::MethodDefinition(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIndexSignature" => Ok(Self::TSIndexSignature(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for MethodDefinition<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let r#type = FromESTree::from_estree(json.estree_field("type")?, allocator)?;
        let decorators = FromESTree::from_estree(json.estree_field("decorators")?, allocator)?;
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let kind = FromESTree::from_estree(json.estree_field("kind")?, allocator)?;
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        let r#static = FromESTree::from_estree(json.estree_field("static")?, allocator)?;
        let r#override = FromESTree::from_estree(json.estree_field("override")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        let accessibility = match json.estree_field_opt("accessibility") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(MethodDefinition {
            span,
            r#type,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            accessibility,
        })
    }
}

impl<'a> FromESTree<'a> for MethodDefinitionType {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "MethodDefinition" => Ok(Self::MethodDefinition),
            "TSAbstractMethodDefinition" => Ok(Self::TSAbstractMethodDefinition),
            other => Err(DeserError::InvalidFieldValue("MethodDefinitionType", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for PropertyDefinition<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let r#type = FromESTree::from_estree(json.estree_field("type")?, allocator)?;
        let decorators = FromESTree::from_estree(json.estree_field("decorators")?, allocator)?;
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let type_annotation = match json.estree_field_opt("typeAnnotation") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let value = match json.estree_field_opt("value") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        let r#static = FromESTree::from_estree(json.estree_field("static")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let r#override = FromESTree::from_estree(json.estree_field("override")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        let definite = FromESTree::from_estree(json.estree_field("definite")?, allocator)?;
        let readonly = FromESTree::from_estree(json.estree_field("readonly")?, allocator)?;
        let accessibility = match json.estree_field_opt("accessibility") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(PropertyDefinition {
            span,
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            accessibility,
        })
    }
}

impl<'a> FromESTree<'a> for PropertyDefinitionType {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "PropertyDefinition" => Ok(Self::PropertyDefinition),
            "TSAbstractPropertyDefinition" => Ok(Self::TSAbstractPropertyDefinition),
            other => {
                Err(DeserError::InvalidFieldValue("PropertyDefinitionType", other.to_string()))
            }
        }
    }
}

impl<'a> FromESTree<'a> for MethodDefinitionKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "constructor" => Ok(Self::Constructor),
            "method" => Ok(Self::Method),
            "get" => Ok(Self::Get),
            "set" => Ok(Self::Set),
            other => Err(DeserError::InvalidFieldValue("MethodDefinitionKind", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for PrivateIdentifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        Ok(PrivateIdentifier { span, name })
    }
}

impl<'a> FromESTree<'a> for StaticBlock<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(StaticBlock { span, body, scope_id })
    }
}

impl<'a> FromESTree<'a> for ModuleDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ExportNamedDeclaration" => Ok(Self::ExportNamedDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ExportDefaultDeclaration" => Ok(Self::ExportDefaultDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ExportAllDeclaration" => Ok(Self::ExportAllDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSExportAssignment" => Ok(Self::TSExportAssignment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNamespaceExportDeclaration" => Ok(Self::TSNamespaceExportDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportDeclaration" => Ok(Self::ImportDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for AccessorPropertyType {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "AccessorProperty" => Ok(Self::AccessorProperty),
            "TSAbstractAccessorProperty" => Ok(Self::TSAbstractAccessorProperty),
            other => Err(DeserError::InvalidFieldValue("AccessorPropertyType", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for AccessorProperty<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let r#type = FromESTree::from_estree(json.estree_field("type")?, allocator)?;
        let decorators = FromESTree::from_estree(json.estree_field("decorators")?, allocator)?;
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let type_annotation = match json.estree_field_opt("typeAnnotation") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let value = match json.estree_field_opt("value") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        let r#static = FromESTree::from_estree(json.estree_field("static")?, allocator)?;
        let r#override = FromESTree::from_estree(json.estree_field("override")?, allocator)?;
        let definite = FromESTree::from_estree(json.estree_field("definite")?, allocator)?;
        let accessibility = match json.estree_field_opt("accessibility") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(AccessorProperty {
            span,
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            r#override,
            definite,
            accessibility,
        })
    }
}

impl<'a> FromESTree<'a> for ImportExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let source = FromESTree::from_estree(json.estree_field("source")?, allocator)?;
        let options = match json.estree_field_opt("options") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let phase = match json.estree_field_opt("phase") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(ImportExpression { span, source, options, phase })
    }
}

impl<'a> FromESTree<'a> for ImportDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let specifiers = match json.estree_field_opt("specifiers") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let source = FromESTree::from_estree(json.estree_field("source")?, allocator)?;
        let phase = match json.estree_field_opt("phase") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let with_clause = match json.estree_field_opt("attributes") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let import_kind = FromESTree::from_estree(json.estree_field("importKind")?, allocator)?;
        Ok(ImportDeclaration { span, specifiers, source, phase, with_clause, import_kind })
    }
}

impl<'a> FromESTree<'a> for ImportPhase {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "source" => Ok(Self::Source),
            "defer" => Ok(Self::Defer),
            other => Err(DeserError::InvalidFieldValue("ImportPhase", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ImportDeclarationSpecifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ImportSpecifier" => Ok(Self::ImportSpecifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportDefaultSpecifier" => Ok(Self::ImportDefaultSpecifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportNamespaceSpecifier" => Ok(Self::ImportNamespaceSpecifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ImportSpecifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let imported = FromESTree::from_estree(json.estree_field("imported")?, allocator)?;
        let local = FromESTree::from_estree(json.estree_field("local")?, allocator)?;
        let import_kind = FromESTree::from_estree(json.estree_field("importKind")?, allocator)?;
        Ok(ImportSpecifier { span, imported, local, import_kind })
    }
}

impl<'a> FromESTree<'a> for ImportDefaultSpecifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let local = FromESTree::from_estree(json.estree_field("local")?, allocator)?;
        Ok(ImportDefaultSpecifier { span, local })
    }
}

impl<'a> FromESTree<'a> for ImportNamespaceSpecifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let local = FromESTree::from_estree(json.estree_field("local")?, allocator)?;
        Ok(ImportNamespaceSpecifier { span, local })
    }
}

impl<'a> FromESTree<'a> for WithClause<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let keyword = crate::ast::js::WithClauseKeyword::With;
        let with_entries = FromESTree::from_estree(json.estree_field("attributes")?, allocator)?;
        Ok(WithClause { span, keyword, with_entries })
    }
}

impl<'a> FromESTree<'a> for ImportAttribute<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        Ok(ImportAttribute { span, key, value })
    }
}

impl<'a> FromESTree<'a> for ImportAttributeKey<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Literal" => Ok(Self::StringLiteral(FromESTree::from_estree(json, allocator)?)),
            "Identifier" => Ok(Self::Identifier(FromESTree::from_estree(json, allocator)?)),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ExportNamedDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let declaration = match json.estree_field_opt("declaration") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let specifiers = FromESTree::from_estree(json.estree_field("specifiers")?, allocator)?;
        let source = match json.estree_field_opt("source") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let export_kind = FromESTree::from_estree(json.estree_field("exportKind")?, allocator)?;
        let with_clause = match json.estree_field_opt("attributes") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(ExportNamedDeclaration {
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        })
    }
}

impl<'a> FromESTree<'a> for ExportDefaultDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let declaration = FromESTree::from_estree(json.estree_field("declaration")?, allocator)?;
        Ok(ExportDefaultDeclaration { span, declaration })
    }
}

impl<'a> FromESTree<'a> for ExportAllDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let exported = match json.estree_field_opt("exported") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let source = FromESTree::from_estree(json.estree_field("source")?, allocator)?;
        let with_clause = match json.estree_field_opt("attributes") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let export_kind = FromESTree::from_estree(json.estree_field("exportKind")?, allocator)?;
        Ok(ExportAllDeclaration { span, exported, source, with_clause, export_kind })
    }
}

impl<'a> FromESTree<'a> for ExportSpecifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let local = FromESTree::from_estree(json.estree_field("local")?, allocator)?;
        let exported = FromESTree::from_estree(json.estree_field("exported")?, allocator)?;
        let export_kind = FromESTree::from_estree(json.estree_field("exportKind")?, allocator)?;
        Ok(ExportSpecifier { span, local, exported, export_kind })
    }
}

impl<'a> FromESTree<'a> for ExportDefaultDeclarationKind<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ArrayExpression" => Ok(Self::ArrayExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrowFunctionExpression" => Ok(Self::ArrowFunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ParenthesizedExpression" => Ok(Self::ParenthesizedExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Super" => {
                Ok(Self::Super(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "YieldExpression" => Ok(Self::YieldExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ObjectExpression" => Ok(Self::ObjectExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TaggedTemplateExpression" => Ok(Self::TaggedTemplateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "UpdateExpression" => Ok(Self::UpdateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "NewExpression" => Ok(Self::NewExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::NullLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::RegExpLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "TSInterfaceDeclaration" => Ok(Self::TSInterfaceDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BinaryExpression" => {
                let operator = json.get("operator").and_then(|v| v.as_str());
                let left_type =
                    json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if operator == Some("in") && left_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateInExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BinaryExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "ConditionalExpression" => Ok(Self::ConditionalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentExpression" => Ok(Self::AssignmentExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SequenceExpression" => Ok(Self::SequenceExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXFragment" => Ok(Self::JSXFragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ChainExpression" => Ok(Self::ChainExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MetaProperty" => Ok(Self::MetaProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "V8IntrinsicExpression" => Ok(Self::V8IntrinsicExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInstantiationExpression" => Ok(Self::TSInstantiationExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Function" => Ok(Self::FunctionDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportExpression" => Ok(Self::ImportExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "LogicalExpression" => Ok(Self::LogicalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::JSXElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AwaitExpression" => Ok(Self::AwaitExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for ModuleExportName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Literal" => Ok(Self::StringLiteral(FromESTree::from_estree(json, allocator)?)),
            "Identifier" => Ok(Self::IdentifierName(FromESTree::from_estree(json, allocator)?)),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for V8IntrinsicExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        let arguments = FromESTree::from_estree(json.estree_field("arguments")?, allocator)?;
        Ok(V8IntrinsicExpression { span, name, arguments })
    }
}

impl<'a> FromESTree<'a> for BooleanLiteral {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        Ok(BooleanLiteral { span, value })
    }
}

impl<'a> FromESTree<'a> for NullLiteral {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(NullLiteral { span })
    }
}

impl<'a> FromESTree<'a> for NumericLiteral<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let raw = match json.estree_field_opt("raw") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let base = oxc_syntax::number::NumberBase::Decimal;
        Ok(NumericLiteral { span, value, raw, base })
    }
}

impl<'a> FromESTree<'a> for StringLiteral<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let raw = match json.estree_field_opt("raw") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let lone_surrogates = false;
        Ok(StringLiteral { span, value, raw, lone_surrogates })
    }
}

impl<'a> FromESTree<'a> for BigIntLiteral<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let raw = match json.estree_field_opt("raw") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let base = oxc_syntax::number::BigintBase::Decimal;
        Ok(BigIntLiteral { span, value, raw, base })
    }
}

impl<'a> FromESTree<'a> for RegExpLiteral<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let regex = FromESTree::from_estree(json.estree_field("regex")?, allocator)?;
        let raw = match json.estree_field_opt("raw") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(RegExpLiteral { span, regex, raw })
    }
}

impl<'a> FromESTree<'a> for RegExp<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let pattern = FromESTree::from_estree(json.estree_field("pattern")?, allocator)?;
        let flags = FromESTree::from_estree(json.estree_field("flags")?, allocator)?;
        Ok(RegExp { pattern, flags })
    }
}

impl<'a> FromESTree<'a> for RegExpPattern<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let text = FromESTree::from_estree(json.estree_field("pattern")?, allocator)?;
        let pattern = None;
        Ok(RegExpPattern { text, pattern })
    }
}

impl<'a> FromESTree<'a> for RegExpFlags {
    fn from_estree(_json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        Ok(Self::empty())
    }
}

impl<'a> FromESTree<'a> for JSXElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let opening_element =
            FromESTree::from_estree(json.estree_field("openingElement")?, allocator)?;
        let children = FromESTree::from_estree(json.estree_field("children")?, allocator)?;
        let closing_element = match json.estree_field_opt("closingElement") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(JSXElement { span, opening_element, children, closing_element })
    }
}

impl<'a> FromESTree<'a> for JSXOpeningElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let attributes = FromESTree::from_estree(json.estree_field("attributes")?, allocator)?;
        Ok(JSXOpeningElement { span, name, type_arguments, attributes })
    }
}

impl<'a> FromESTree<'a> for JSXClosingElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        Ok(JSXClosingElement { span, name })
    }
}

impl<'a> FromESTree<'a> for JSXFragment<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let opening_fragment =
            FromESTree::from_estree(json.estree_field("openingFragment")?, allocator)?;
        let children = FromESTree::from_estree(json.estree_field("children")?, allocator)?;
        let closing_fragment =
            FromESTree::from_estree(json.estree_field("closingFragment")?, allocator)?;
        Ok(JSXFragment { span, opening_fragment, children, closing_fragment })
    }
}

impl<'a> FromESTree<'a> for JSXOpeningFragment {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(JSXOpeningFragment { span })
    }
}

impl<'a> FromESTree<'a> for JSXClosingFragment {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(JSXClosingFragment { span })
    }
}

impl<'a> FromESTree<'a> for JSXElementName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "JSXIdentifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXMemberExpression" => Ok(Self::MemberExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXNamespacedName" => Ok(Self::NamespacedName(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::IdentifierReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSXNamespacedName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let namespace = FromESTree::from_estree(json.estree_field("namespace")?, allocator)?;
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        Ok(JSXNamespacedName { span, namespace, name })
    }
}

impl<'a> FromESTree<'a> for JSXMemberExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let object = FromESTree::from_estree(json.estree_field("object")?, allocator)?;
        let property = FromESTree::from_estree(json.estree_field("property")?, allocator)?;
        Ok(JSXMemberExpression { span, object, property })
    }
}

impl<'a> FromESTree<'a> for JSXMemberExpressionObject<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXMemberExpression" => Ok(Self::MemberExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::IdentifierReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSXExpressionContainer<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(JSXExpressionContainer { span, expression })
    }
}

impl<'a> FromESTree<'a> for JSXExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ObjectExpression" => Ok(Self::ObjectExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TaggedTemplateExpression" => Ok(Self::TaggedTemplateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrayExpression" => Ok(Self::ArrayExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInstantiationExpression" => Ok(Self::TSInstantiationExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "BinaryExpression" => {
                let operator = json.get("operator").and_then(|v| v.as_str());
                let left_type =
                    json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if operator == Some("in") && left_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateInExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BinaryExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "Function" => Ok(Self::FunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ConditionalExpression" => Ok(Self::ConditionalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "NewExpression" => Ok(Self::NewExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "V8IntrinsicExpression" => Ok(Self::V8IntrinsicExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "YieldExpression" => Ok(Self::YieldExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAsExpression" => Ok(Self::TSAsExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MetaProperty" => Ok(Self::MetaProperty(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "CallExpression" => Ok(Self::CallExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ChainExpression" => Ok(Self::ChainExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ImportExpression" => Ok(Self::ImportExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "MemberExpression" => {
                let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
                let property_type =
                    json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());
                if property_type == Some("PrivateIdentifier") {
                    Ok(Self::PrivateFieldExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if is_computed {
                    Ok(Self::ComputedMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::StaticMemberExpression(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "TSTypeAssertion" => Ok(Self::TSTypeAssertion(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNonNullExpression" => Ok(Self::TSNonNullExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "SequenceExpression" => Ok(Self::SequenceExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ArrowFunctionExpression" => Ok(Self::ArrowFunctionExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Class" => Ok(Self::ClassExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Super" => {
                Ok(Self::Super(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::JSXElement(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXFragment" => Ok(Self::JSXFragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "LogicalExpression" => Ok(Self::LogicalExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ParenthesizedExpression" => Ok(Self::ParenthesizedExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::NullLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::RegExpLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "AwaitExpression" => Ok(Self::AwaitExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSatisfiesExpression" => Ok(Self::TSSatisfiesExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXEmptyExpression" => {
                Ok(Self::EmptyExpression(FromESTree::from_estree(json, allocator)?))
            }
            "UpdateExpression" => Ok(Self::UpdateExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "AssignmentExpression" => Ok(Self::AssignmentExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSXEmptyExpression {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(JSXEmptyExpression { span })
    }
}

impl<'a> FromESTree<'a> for JSXAttributeItem<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "JSXAttribute" => Ok(Self::Attribute(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXSpreadAttribute" => Ok(Self::SpreadAttribute(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSXAttribute<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        let value = match json.estree_field_opt("value") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(JSXAttribute { span, name, value })
    }
}

impl<'a> FromESTree<'a> for JSXSpreadAttribute<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let argument = FromESTree::from_estree(json.estree_field("argument")?, allocator)?;
        Ok(JSXSpreadAttribute { span, argument })
    }
}

impl<'a> FromESTree<'a> for JSXAttributeName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "JSXIdentifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXNamespacedName" => Ok(Self::NamespacedName(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSXAttributeValue<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "JSXFragment" => Ok(Self::Fragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Literal" => Ok(Self::StringLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXExpressionContainer" => Ok(Self::ExpressionContainer(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXElement" => Ok(Self::Element(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSXIdentifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        Ok(JSXIdentifier { span, name })
    }
}

impl<'a> FromESTree<'a> for JSXChild<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "JSXElement" => Ok(Self::Element(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXSpreadChild" => {
                Ok(Self::Spread(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            "JSXFragment" => Ok(Self::Fragment(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXExpressionContainer" => Ok(Self::ExpressionContainer(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "JSXText" => {
                Ok(Self::Text(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
            }
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSXSpreadChild<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(JSXSpreadChild { span, expression })
    }
}

impl<'a> FromESTree<'a> for JSXText<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let value = FromESTree::from_estree(json.estree_field("value")?, allocator)?;
        let raw = match json.estree_field_opt("raw") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(JSXText { span, value, raw })
    }
}

impl<'a> FromESTree<'a> for TSThisParameter<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let this_span = Default::default();
        let type_annotation = match json.estree_field_opt("typeAnnotation") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSThisParameter { span, this_span, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSEnumDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let r#const = FromESTree::from_estree(json.estree_field("const")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSEnumDeclaration { span, id, body, r#const, declare, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSEnumBody<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let members = FromESTree::from_estree(json.estree_field("members")?, allocator)?;
        Ok(TSEnumBody { span, members })
    }
}

impl<'a> FromESTree<'a> for TSEnumMember<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        let initializer = match json.estree_field_opt("initializer") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSEnumMember { span, id, initializer })
    }
}

impl<'a> FromESTree<'a> for TSEnumMemberName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::String(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::String(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::String(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::String(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::String(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::ComputedString(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::String(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            "TemplateLiteral" => Ok(Self::ComputedTemplateString(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSTypeAnnotation<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSTypeAnnotation { span, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSLiteralType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let literal = FromESTree::from_estree(json.estree_field("literal")?, allocator)?;
        Ok(TSLiteralType { span, literal })
    }
}

impl<'a> FromESTree<'a> for TSLiteral<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "UnaryExpression" => Ok(Self::UnaryExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TemplateLiteral" => Ok(Self::TemplateLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Literal" => {
                if json.get("value").is_some_and(|v| v.is_null()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_boolean()) {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_number()) {
                    Ok(Self::NumericLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("bigint").is_some() {
                    Ok(Self::BigIntLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("regex").is_some() {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else if json.get("value").is_some_and(|v| v.is_string()) {
                    Ok(Self::StringLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                } else {
                    Ok(Self::BooleanLiteral(ABox::new_in(
                        FromESTree::from_estree(json, allocator)?,
                        allocator,
                    )))
                }
            }
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSVoidKeyword" => Ok(Self::TSVoidKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSArrayType" => Ok(Self::TSArrayType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAnyKeyword" => Ok(Self::TSAnyKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSParenthesizedType" => Ok(Self::TSParenthesizedType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSStringKeyword" => Ok(Self::TSStringKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSMappedType" => Ok(Self::TSMappedType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNullKeyword" => Ok(Self::TSNullKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeOperator" => Ok(Self::TSTypeOperatorType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSJSDocNullableType" => Ok(Self::JSDocNullableType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSJSDocUnknownType" => Ok(Self::JSDocUnknownType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNamedTupleMember" => Ok(Self::TSNamedTupleMember(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNumberKeyword" => Ok(Self::TSNumberKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNeverKeyword" => Ok(Self::TSNeverKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSImportType" => Ok(Self::TSImportType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSThisType" => Ok(Self::TSThisType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeReference" => Ok(Self::TSTypeReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSObjectKeyword" => Ok(Self::TSObjectKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeQuery" => Ok(Self::TSTypeQuery(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSymbolKeyword" => Ok(Self::TSSymbolKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSUnknownKeyword" => Ok(Self::TSUnknownKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInferType" => Ok(Self::TSInferType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTupleType" => Ok(Self::TSTupleType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIntersectionType" => Ok(Self::TSIntersectionType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSLiteralType" => Ok(Self::TSLiteralType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeLiteral" => Ok(Self::TSTypeLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSBigIntKeyword" => Ok(Self::TSBigIntKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSUndefinedKeyword" => Ok(Self::TSUndefinedKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSUnionType" => Ok(Self::TSUnionType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIndexedAccessType" => Ok(Self::TSIndexedAccessType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTemplateLiteralType" => Ok(Self::TSTemplateLiteralType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSFunctionType" => Ok(Self::TSFunctionType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSConditionalType" => Ok(Self::TSConditionalType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSConstructorType" => Ok(Self::TSConstructorType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSBooleanKeyword" => Ok(Self::TSBooleanKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIntrinsicKeyword" => Ok(Self::TSIntrinsicKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypePredicate" => Ok(Self::TSTypePredicate(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSJSDocNonNullableType" => Ok(Self::JSDocNonNullableType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSConditionalType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let check_type = FromESTree::from_estree(json.estree_field("checkType")?, allocator)?;
        let extends_type = FromESTree::from_estree(json.estree_field("extendsType")?, allocator)?;
        let true_type = FromESTree::from_estree(json.estree_field("trueType")?, allocator)?;
        let false_type = FromESTree::from_estree(json.estree_field("falseType")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSConditionalType { span, check_type, extends_type, true_type, false_type, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSUnionType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let types = FromESTree::from_estree(json.estree_field("types")?, allocator)?;
        Ok(TSUnionType { span, types })
    }
}

impl<'a> FromESTree<'a> for TSIntersectionType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let types = FromESTree::from_estree(json.estree_field("types")?, allocator)?;
        Ok(TSIntersectionType { span, types })
    }
}

impl<'a> FromESTree<'a> for TSParenthesizedType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSParenthesizedType { span, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSTypeOperator<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let operator = FromESTree::from_estree(json.estree_field("operator")?, allocator)?;
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSTypeOperator { span, operator, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSTypeOperatorOperator {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "keyof" => Ok(Self::Keyof),
            "unique" => Ok(Self::Unique),
            "readonly" => Ok(Self::Readonly),
            other => {
                Err(DeserError::InvalidFieldValue("TSTypeOperatorOperator", other.to_string()))
            }
        }
    }
}

impl<'a> FromESTree<'a> for TSArrayType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let element_type = FromESTree::from_estree(json.estree_field("elementType")?, allocator)?;
        Ok(TSArrayType { span, element_type })
    }
}

impl<'a> FromESTree<'a> for TSIndexedAccessType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let object_type = FromESTree::from_estree(json.estree_field("objectType")?, allocator)?;
        let index_type = FromESTree::from_estree(json.estree_field("indexType")?, allocator)?;
        Ok(TSIndexedAccessType { span, object_type, index_type })
    }
}

impl<'a> FromESTree<'a> for TSTupleType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let element_types = FromESTree::from_estree(json.estree_field("elementTypes")?, allocator)?;
        Ok(TSTupleType { span, element_types })
    }
}

impl<'a> FromESTree<'a> for TSNamedTupleMember<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let label = FromESTree::from_estree(json.estree_field("label")?, allocator)?;
        let element_type = FromESTree::from_estree(json.estree_field("elementType")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        Ok(TSNamedTupleMember { span, label, element_type, optional })
    }
}

impl<'a> FromESTree<'a> for TSOptionalType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSOptionalType { span, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSRestType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSRestType { span, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSTupleElement<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSNumberKeyword" => Ok(Self::TSNumberKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSThisType" => Ok(Self::TSThisType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSBigIntKeyword" => Ok(Self::TSBigIntKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSJSDocNullableType" => Ok(Self::JSDocNullableType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSBooleanKeyword" => Ok(Self::TSBooleanKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNamedTupleMember" => Ok(Self::TSNamedTupleMember(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypePredicate" => Ok(Self::TSTypePredicate(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSFunctionType" => Ok(Self::TSFunctionType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIntrinsicKeyword" => Ok(Self::TSIntrinsicKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSMappedType" => Ok(Self::TSMappedType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSImportType" => Ok(Self::TSImportType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSJSDocNonNullableType" => Ok(Self::JSDocNonNullableType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSLiteralType" => Ok(Self::TSLiteralType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTemplateLiteralType" => Ok(Self::TSTemplateLiteralType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSOptionalType" => Ok(Self::TSOptionalType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeOperator" => Ok(Self::TSTypeOperatorType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSStringKeyword" => Ok(Self::TSStringKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSUndefinedKeyword" => Ok(Self::TSUndefinedKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSUnionType" => Ok(Self::TSUnionType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSArrayType" => Ok(Self::TSArrayType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNeverKeyword" => Ok(Self::TSNeverKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSConstructorType" => Ok(Self::TSConstructorType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTupleType" => Ok(Self::TSTupleType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSVoidKeyword" => Ok(Self::TSVoidKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIntersectionType" => Ok(Self::TSIntersectionType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSSymbolKeyword" => Ok(Self::TSSymbolKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSObjectKeyword" => Ok(Self::TSObjectKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSInferType" => Ok(Self::TSInferType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSUnknownKeyword" => Ok(Self::TSUnknownKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeQuery" => Ok(Self::TSTypeQuery(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeReference" => Ok(Self::TSTypeReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSAnyKeyword" => Ok(Self::TSAnyKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSConditionalType" => Ok(Self::TSConditionalType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIndexedAccessType" => Ok(Self::TSIndexedAccessType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSRestType" => Ok(Self::TSRestType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSTypeLiteral" => Ok(Self::TSTypeLiteral(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSNullKeyword" => Ok(Self::TSNullKeyword(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSParenthesizedType" => Ok(Self::TSParenthesizedType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSJSDocUnknownType" => Ok(Self::JSDocUnknownType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSAnyKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSAnyKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSStringKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSStringKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSBooleanKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSBooleanKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSNumberKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSNumberKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSNeverKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSNeverKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSIntrinsicKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSIntrinsicKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSUnknownKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSUnknownKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSNullKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSNullKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSUndefinedKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSUndefinedKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSVoidKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSVoidKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSSymbolKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSSymbolKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSThisType {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSThisType { span })
    }
}

impl<'a> FromESTree<'a> for TSObjectKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSObjectKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSBigIntKeyword {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(TSBigIntKeyword { span })
    }
}

impl<'a> FromESTree<'a> for TSTypeReference<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_name = FromESTree::from_estree(json.estree_field("typeName")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSTypeReference { span, type_name, type_arguments })
    }
}

impl<'a> FromESTree<'a> for TSTypeName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Identifier" => Ok(Self::IdentifierReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSQualifiedName" => Ok(Self::QualifiedName(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSQualifiedName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(TSQualifiedName { span, left, right })
    }
}

impl<'a> FromESTree<'a> for TSTypeParameterInstantiation<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        Ok(TSTypeParameterInstantiation { span, params })
    }
}

impl<'a> FromESTree<'a> for TSTypeParameter<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        let constraint = match json.estree_field_opt("constraint") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let default = match json.estree_field_opt("default") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let r#in = FromESTree::from_estree(json.estree_field("in")?, allocator)?;
        let out = FromESTree::from_estree(json.estree_field("out")?, allocator)?;
        let r#const = FromESTree::from_estree(json.estree_field("const")?, allocator)?;
        Ok(TSTypeParameter { span, name, constraint, default, r#in, out, r#const })
    }
}

impl<'a> FromESTree<'a> for TSTypeParameterDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        Ok(TSTypeParameterDeclaration { span, params })
    }
}

impl<'a> FromESTree<'a> for TSTypeAliasDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSTypeAliasDeclaration { span, id, type_parameters, type_annotation, declare, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSAccessibility {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "private" => Ok(Self::Private),
            "protected" => Ok(Self::Protected),
            "public" => Ok(Self::Public),
            other => Err(DeserError::InvalidFieldValue("TSAccessibility", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSClassImplements<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSClassImplements { span, expression, type_arguments })
    }
}

impl<'a> FromESTree<'a> for TSInterfaceDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let extends = FromESTree::from_estree(json.estree_field("extends")?, allocator)?;
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSInterfaceDeclaration { span, id, type_parameters, extends, body, declare, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSInterfaceBody<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        Ok(TSInterfaceBody { span, body })
    }
}

impl<'a> FromESTree<'a> for TSPropertySignature<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        let readonly = FromESTree::from_estree(json.estree_field("readonly")?, allocator)?;
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let type_annotation = match json.estree_field_opt("typeAnnotation") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSPropertySignature { span, computed, optional, readonly, key, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSSignature<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSPropertySignature" => Ok(Self::TSPropertySignature(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSIndexSignature" => Ok(Self::TSIndexSignature(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSCallSignatureDeclaration" => Ok(Self::TSCallSignatureDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSConstructSignatureDeclaration" => Ok(Self::TSConstructSignatureDeclaration(
                ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator),
            )),
            "TSMethodSignature" => Ok(Self::TSMethodSignature(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSIndexSignature<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let parameters = FromESTree::from_estree(json.estree_field("parameters")?, allocator)?;
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        let readonly = FromESTree::from_estree(json.estree_field("readonly")?, allocator)?;
        let r#static = FromESTree::from_estree(json.estree_field("static")?, allocator)?;
        Ok(TSIndexSignature { span, parameters, type_annotation, readonly, r#static })
    }
}

impl<'a> FromESTree<'a> for TSCallSignatureDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let this_param = None;
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        let return_type = match json.estree_field_opt("returnType") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let scope_id = std::cell::Cell::default();
        Ok(TSCallSignatureDeclaration {
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
        })
    }
}

impl<'a> FromESTree<'a> for TSMethodSignatureKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "method" => Ok(Self::Method),
            "get" => Ok(Self::Get),
            "set" => Ok(Self::Set),
            other => Err(DeserError::InvalidFieldValue("TSMethodSignatureKind", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSMethodSignature<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let key = FromESTree::from_estree(json.estree_field("key")?, allocator)?;
        let computed = FromESTree::from_estree(json.estree_field("computed")?, allocator)?;
        let optional = FromESTree::from_estree(json.estree_field("optional")?, allocator)?;
        let kind = FromESTree::from_estree(json.estree_field("kind")?, allocator)?;
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let this_param = None;
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        let return_type = match json.estree_field_opt("returnType") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let scope_id = std::cell::Cell::default();
        Ok(TSMethodSignature {
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
        })
    }
}

impl<'a> FromESTree<'a> for TSConstructSignatureDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        let return_type = match json.estree_field_opt("returnType") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let scope_id = std::cell::Cell::default();
        Ok(TSConstructSignatureDeclaration { span, type_parameters, params, return_type, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSIndexSignatureName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let name = FromESTree::from_estree(json.estree_field("name")?, allocator)?;
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSIndexSignatureName { span, name, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSInterfaceHeritage<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSInterfaceHeritage { span, expression, type_arguments })
    }
}

impl<'a> FromESTree<'a> for TSTypePredicate<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let parameter_name =
            FromESTree::from_estree(json.estree_field("parameterName")?, allocator)?;
        let asserts = FromESTree::from_estree(json.estree_field("asserts")?, allocator)?;
        let type_annotation = match json.estree_field_opt("typeAnnotation") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSTypePredicate { span, parameter_name, asserts, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSTypePredicateName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSThisType" => Ok(Self::This(FromESTree::from_estree(json, allocator)?)),
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSModuleDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        let body = match json.estree_field_opt("body") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let kind = FromESTree::from_estree(json.estree_field("kind")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSModuleDeclaration { span, id, body, kind, declare, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSModuleDeclarationKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "module" => Ok(Self::Module),
            "namespace" => Ok(Self::Namespace),
            other => {
                Err(DeserError::InvalidFieldValue("TSModuleDeclarationKind", other.to_string()))
            }
        }
    }
}

impl<'a> FromESTree<'a> for TSModuleDeclarationName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Identifier" => Ok(Self::Identifier(FromESTree::from_estree(json, allocator)?)),
            "Literal" => Ok(Self::StringLiteral(FromESTree::from_estree(json, allocator)?)),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSModuleDeclarationBody<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSModuleBlock" => Ok(Self::TSModuleBlock(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSModuleDeclaration" => Ok(Self::TSModuleDeclaration(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSGlobalDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let global_span = Default::default();
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        let declare = FromESTree::from_estree(json.estree_field("declare")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSGlobalDeclaration { span, global_span, body, declare, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSModuleBlock<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let directives = AVec::new_in(allocator);
        let body = FromESTree::from_estree(json.estree_field("body")?, allocator)?;
        Ok(TSModuleBlock { span, directives, body })
    }
}

impl<'a> FromESTree<'a> for TSTypeLiteral<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let members = FromESTree::from_estree(json.estree_field("members")?, allocator)?;
        Ok(TSTypeLiteral { span, members })
    }
}

impl<'a> FromESTree<'a> for TSInferType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_parameter =
            FromESTree::from_estree(json.estree_field("typeParameter")?, allocator)?;
        Ok(TSInferType { span, type_parameter })
    }
}

impl<'a> FromESTree<'a> for TSTypeQuery<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expr_name = FromESTree::from_estree(json.estree_field("exprName")?, allocator)?;
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSTypeQuery { span, expr_name, type_arguments })
    }
}

impl<'a> FromESTree<'a> for TSTypeQueryExprName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSImportType" => Ok(Self::TSImportType(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::IdentifierReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSQualifiedName" => Ok(Self::QualifiedName(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSImportType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let source = FromESTree::from_estree(json.estree_field("source")?, allocator)?;
        let options = match json.estree_field_opt("options") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let qualifier = match json.estree_field_opt("qualifier") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let type_arguments = match json.estree_field_opt("typeArguments") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        Ok(TSImportType { span, source, options, qualifier, type_arguments })
    }
}

impl<'a> FromESTree<'a> for TSImportTypeQualifier<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "Identifier" => Ok(Self::Identifier(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSQualifiedName" => Ok(Self::QualifiedName(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSImportTypeQualifiedName<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let left = FromESTree::from_estree(json.estree_field("left")?, allocator)?;
        let right = FromESTree::from_estree(json.estree_field("right")?, allocator)?;
        Ok(TSImportTypeQualifiedName { span, left, right })
    }
}

impl<'a> FromESTree<'a> for TSFunctionType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let this_param = None;
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        let return_type = FromESTree::from_estree(json.estree_field("returnType")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSFunctionType { span, type_parameters, this_param, params, return_type, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSConstructorType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let r#abstract = FromESTree::from_estree(json.estree_field("abstract")?, allocator)?;
        let type_parameters = match json.estree_field_opt("typeParameters") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let params = FromESTree::from_estree(json.estree_field("params")?, allocator)?;
        let return_type = FromESTree::from_estree(json.estree_field("returnType")?, allocator)?;
        let scope_id = std::cell::Cell::default();
        Ok(TSConstructorType { span, r#abstract, type_parameters, params, return_type, scope_id })
    }
}

impl<'a> FromESTree<'a> for TSMappedType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_parameter =
            panic!("Cannot default Box<{}> - field should not be skipped", "TSTypeParameter");
        let name_type = match json.estree_field_opt("nameType") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let type_annotation = match json.estree_field_opt("typeAnnotation") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let optional = match json.estree_field_opt("optional") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let readonly = match json.estree_field_opt("readonly") {
            Some(field_json) if !field_json.is_null() => {
                Some(FromESTree::from_estree(field_json, allocator)?)
            }
            _ => None,
        };
        let scope_id = std::cell::Cell::default();
        Ok(TSMappedType {
            span,
            type_parameter,
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id,
        })
    }
}

impl<'a> FromESTree<'a> for TSMappedTypeModifierOperator {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "true" => Ok(Self::True),
            "+" => Ok(Self::Plus),
            "-" => Ok(Self::Minus),
            other => Err(DeserError::InvalidFieldValue(
                "TSMappedTypeModifierOperator",
                other.to_string(),
            )),
        }
    }
}

impl<'a> FromESTree<'a> for TSTemplateLiteralType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let quasis = FromESTree::from_estree(json.estree_field("quasis")?, allocator)?;
        let types = FromESTree::from_estree(json.estree_field("types")?, allocator)?;
        Ok(TSTemplateLiteralType { span, quasis, types })
    }
}

impl<'a> FromESTree<'a> for TSAsExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSAsExpression { span, expression, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSSatisfiesExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        Ok(TSSatisfiesExpression { span, expression, type_annotation })
    }
}

impl<'a> FromESTree<'a> for TSTypeAssertion<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(TSTypeAssertion { span, type_annotation, expression })
    }
}

impl<'a> FromESTree<'a> for TSImportEqualsDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        let module_reference =
            FromESTree::from_estree(json.estree_field("moduleReference")?, allocator)?;
        let import_kind = FromESTree::from_estree(json.estree_field("importKind")?, allocator)?;
        Ok(TSImportEqualsDeclaration { span, id, module_reference, import_kind })
    }
}

impl<'a> FromESTree<'a> for TSModuleReference<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let type_name = json.estree_type()?;
        match type_name {
            "TSQualifiedName" => Ok(Self::QualifiedName(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "TSExternalModuleReference" => Ok(Self::ExternalModuleReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "Identifier" => Ok(Self::IdentifierReference(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            "ThisExpression" => Ok(Self::ThisExpression(ABox::new_in(
                FromESTree::from_estree(json, allocator)?,
                allocator,
            ))),
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for TSExternalModuleReference<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(TSExternalModuleReference { span, expression })
    }
}

impl<'a> FromESTree<'a> for TSNonNullExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(TSNonNullExpression { span, expression })
    }
}

impl<'a> FromESTree<'a> for Decorator<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(Decorator { span, expression })
    }
}

impl<'a> FromESTree<'a> for TSExportAssignment<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        Ok(TSExportAssignment { span, expression })
    }
}

impl<'a> FromESTree<'a> for TSNamespaceExportDeclaration<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let id = FromESTree::from_estree(json.estree_field("id")?, allocator)?;
        Ok(TSNamespaceExportDeclaration { span, id })
    }
}

impl<'a> FromESTree<'a> for TSInstantiationExpression<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let expression = FromESTree::from_estree(json.estree_field("expression")?, allocator)?;
        let type_arguments =
            FromESTree::from_estree(json.estree_field("typeArguments")?, allocator)?;
        Ok(TSInstantiationExpression { span, expression, type_arguments })
    }
}

impl<'a> FromESTree<'a> for ImportOrExportKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "value" => Ok(Self::Value),
            "type" => Ok(Self::Type),
            other => Err(DeserError::InvalidFieldValue("ImportOrExportKind", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for JSDocNullableType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        let postfix = FromESTree::from_estree(json.estree_field("postfix")?, allocator)?;
        Ok(JSDocNullableType { span, type_annotation, postfix })
    }
}

impl<'a> FromESTree<'a> for JSDocNonNullableType<'a> {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let type_annotation =
            FromESTree::from_estree(json.estree_field("typeAnnotation")?, allocator)?;
        let postfix = FromESTree::from_estree(json.estree_field("postfix")?, allocator)?;
        Ok(JSDocNonNullableType { span, type_annotation, postfix })
    }
}

impl<'a> FromESTree<'a> for JSDocUnknownType {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        Ok(JSDocUnknownType { span })
    }
}

impl<'a> FromESTree<'a> for CommentKind {
    fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let s = json.as_str().ok_or(DeserError::ExpectedString)?;
        match s {
            "Line" => Ok(Self::Line),
            "Block" => Ok(Self::SingleLineBlock),
            "Block" => Ok(Self::MultiLineBlock),
            other => Err(DeserError::InvalidFieldValue("CommentKind", other.to_string())),
        }
    }
}

impl<'a> FromESTree<'a> for Comment {
    fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(json);
        let attached_to = 0;
        let kind = FromESTree::from_estree(json.estree_field("type")?, allocator)?;
        let position = Default::default();
        let newlines = Default::default();
        let content = Default::default();
        Ok(Comment { span, attached_to, kind, position, newlines, content })
    }
}
