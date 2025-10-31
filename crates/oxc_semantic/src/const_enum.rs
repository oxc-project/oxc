//! Const enum value evaluation and storage
//!
//! This module provides functionality for evaluating and storing const enum values
//! during semantic analysis. Const enums are compiled away and their members are
//! inlined as literal values.

use std::collections::HashMap;
use num_bigint::BigInt;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

/// Represents a computed const enum member value
#[derive(Debug, Clone)]
pub enum ConstEnumMemberValue<'a> {
    /// String literal value
    String(&'a str),
    /// Numeric literal value (f64 to handle both integers and floats)
    Number(f64),
    /// BigInt value
    BigInt(BigInt),
    /// Boolean value
    Boolean(bool),
    /// Computed value from other enum members (not stored for now)
    Computed,
}

impl<'a> PartialEq for ConstEnumMemberValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::BigInt(l), Self::BigInt(r)) => l == r,
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            (Self::Computed, Self::Computed) => true,
            _ => false,
        }
    }
}

/// Information about a const enum member
#[derive(Debug, Clone)]
pub struct ConstEnumMemberInfo<'a> {
    /// Name of the enum member
    pub name: &'a str,
    /// Computed value of the member
    pub value: ConstEnumMemberValue<'a>,
    /// Span of the member declaration
    pub span: Span,
    /// Whether this member has an explicit initializer
    pub has_initializer: bool,
}

/// Information about a const enum
#[derive(Debug, Clone)]
pub struct ConstEnumInfo<'a> {
    /// Symbol ID of the const enum
    pub symbol_id: SymbolId,
    /// Members of the const enum
    pub members: HashMap<&'a str, ConstEnumMemberInfo<'a>>,
    /// Span of the enum declaration
    pub span: Span,
}

/// Storage for all const enum information in a program
#[derive(Debug, Default, Clone)]
pub struct ConstEnumTable<'a> {
    /// Map of const enum symbol IDs to their information
    pub enums: HashMap<SymbolId, ConstEnumInfo<'a>>,
}

impl<'a> ConstEnumTable<'a> {
    /// Create a new const enum table
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a const enum to the table
    pub fn add_enum(&mut self, symbol_id: SymbolId, enum_info: ConstEnumInfo<'a>) {
        self.enums.insert(symbol_id, enum_info);
    }

    /// Get const enum information by symbol ID
    pub fn get_enum(&self, symbol_id: SymbolId) -> Option<&ConstEnumInfo<'a>> {
        self.enums.get(&symbol_id)
    }

    /// Get a const enum member value
    pub fn get_member_value(
        &self,
        symbol_id: SymbolId,
        member_name: &str,
    ) -> Option<&ConstEnumMemberValue<'a>> {
        self.enums
            .get(&symbol_id)
            .and_then(|enum_info| enum_info.members.get(member_name))
            .map(|member| &member.value)
    }

    /// Check if a symbol represents a const enum
    pub fn is_const_enum(&self, symbol_id: SymbolId) -> bool {
        self.enums.contains_key(&symbol_id)
    }

    /// Get all const enums
    pub fn enums(&self) -> impl Iterator<Item = (&SymbolId, &ConstEnumInfo<'a>)> {
        self.enums.iter()
    }
}


