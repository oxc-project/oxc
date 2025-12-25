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

use crate::ast::js::*;
use crate::ast::literal::*;

impl<'a> FromESTree<'a> for IdentifierName<'a> {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(value);
        let name = FromESTree::from_estree(value.estree_field("name")?, allocator)?;
        Ok(IdentifierName { span, name })
    }
}

impl<'a> FromESTree<'a> for IdentifierReference<'a> {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(value);
        let name = FromESTree::from_estree(value.estree_field("name")?, allocator)?;
        let reference_id = std::cell::Cell::default();
        Ok(IdentifierReference { span, name, reference_id })
    }
}

impl<'a> FromESTree<'a> for BooleanLiteral {
    fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(value);
        let value = FromESTree::from_estree(value.estree_field("value")?, allocator)?;
        Ok(BooleanLiteral { span, value })
    }
}

impl<'a> FromESTree<'a> for NullLiteral {
    fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
        let span = parse_span_or_empty(value);
        Ok(NullLiteral { span })
    }
}
