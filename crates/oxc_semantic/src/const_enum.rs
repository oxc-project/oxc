//! Const enum value evaluation and storage
//!
//! This module provides functionality for evaluating and storing const enum values
//! during semantic analysis. Const enums are compiled away and their members are
//! inlined as literal values.
//!
//! Uses the enum evaluation logic from oxc_ecmascript::enum_evaluation, which is
//! based on TypeScript's enum implementation and shared with the transformer.

use oxc_ast::{AstBuilder, ast::TSEnumDeclaration};
use oxc_ecmascript::enum_evaluation::{ConstantValue, EnumEvaluator};
use oxc_span::Atom;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::FxHashMap;

use crate::Scoping;

/// Owned version of ConstantValue that doesn't require arena lifetime.
/// TypeScript only allows number and string as enum member values.
#[derive(Debug, Clone, PartialEq)]
pub enum NormalizedConstantValue {
    Number(f64),
    String(String),
}

impl std::fmt::Display for NormalizedConstantValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{s}\""),
        }
    }
}

impl<'a> From<ConstantValue<'a>> for NormalizedConstantValue {
    fn from(value: ConstantValue<'a>) -> Self {
        match value {
            ConstantValue::Number(n) => Self::Number(n),
            ConstantValue::String(s) => Self::String(s.to_string()),
        }
    }
}

/// Normalized const enum info without arena lifetime
#[derive(Debug, Clone)]
pub struct NormalizedConstEnumInfo {
    /// Members of the const enum
    pub members: FxHashMap<SymbolId, NormalizedConstantValue>,
    /// Member name to symbol ID mapping for cross-module const enum inlining
    pub member_name_to_symbol_id: FxHashMap<String, SymbolId>,
}

/// Storage for all const enum information in a program
#[derive(Debug, Default, Clone)]
pub struct ConstEnumTable {
    /// Map of const enum symbol IDs to their information
    pub enums: FxHashMap<SymbolId, NormalizedConstEnumInfo>,
}

impl ConstEnumTable {
    /// Create a new const enum table
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a const enum to the table
    pub fn add_enum(&mut self, symbol_id: SymbolId, enum_info: NormalizedConstEnumInfo) {
        self.enums.insert(symbol_id, enum_info);
    }

    /// Get all const enums
    pub fn enums(&self) -> impl Iterator<Item = (&SymbolId, &NormalizedConstEnumInfo)> {
        self.enums.iter()
    }
}

/// Process a const enum declaration and evaluate its members
/// using the TypeScript-based enum evaluation logic
pub fn process_const_enum(
    enum_declaration: &TSEnumDeclaration<'_>,
    scoping: &Scoping,
    const_enum_table: &mut ConstEnumTable,
) {
    let symbol_id = enum_declaration.id.symbol_id();
    let current_scope = enum_declaration.scope_id();
    let allocator = oxc_allocator::Allocator::default();
    let ast_builder = AstBuilder::new(&allocator);
    let evaluator = EnumEvaluator::new(ast_builder);

    // Track previous members for constant propagation within the same enum
    let mut prev_members: FxHashMap<Atom, Option<ConstantValue>> = FxHashMap::default();
    let mut members = FxHashMap::default();
    let mut member_name_to_symbol_id = FxHashMap::default();
    let mut next_index: Option<f64> = Some(-1.0); // Start at -1, first auto-increment will make it 0

    for member in &enum_declaration.body.members {
        let member_name = member.id.static_name();

        let Some(member_symbol_id) = scoping.get_binding(current_scope, member_name.as_str())
        else {
            continue;
        };

        let member_atom = ast_builder.atom(&member_name);

        // Evaluate the member value
        let value = if let Some(initializer) = &member.initializer {
            let evaluated = evaluator.computed_constant_value(initializer, &prev_members);
            match evaluated {
                Some(ConstantValue::Number(n)) => {
                    next_index = Some(n);
                    evaluated
                }
                Some(ConstantValue::String(_)) => {
                    // After a string member, auto-increment is no longer possible
                    next_index = None;
                    evaluated
                }
                None => {
                    next_index = None;
                    None
                }
            }
        } else {
            // Auto-increment based on previous numeric member
            match next_index.as_mut() {
                Some(n) => {
                    *n += 1.0;
                    Some(ConstantValue::Number(*n))
                }
                None => None,
            }
        };

        // Store the member for reference by later members
        prev_members.insert(member_atom, value);
        member_name_to_symbol_id.insert(member_name.to_string(), member_symbol_id);

        // Only store successfully evaluated values
        if let Some(const_value) = value {
            members.insert(member_symbol_id, const_value.into());
        }
    }

    let enum_info = NormalizedConstEnumInfo { members, member_name_to_symbol_id };
    const_enum_table.add_enum(symbol_id, enum_info);
}
