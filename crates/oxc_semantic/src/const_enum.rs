//! Const enum value evaluation and storage
//!
//! This module provides functionality for evaluating and storing const enum values
//! during semantic analysis. Const enums are compiled away and their members are
//! inlined as literal values.

use num_bigint::BigInt;
use oxc_ast::{
    AstBuilder,
    ast::{Expression, IdentifierReference, TSEnumDeclaration, TSEnumMemberName},
};
use oxc_ecmascript::{
    GlobalContext,
    constant_evaluation::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue},
    side_effects::{MayHaveSideEffectsContext, PropertyReadSideEffects},
};
use oxc_syntax::{reference::ReferenceId, symbol::SymbolId};
use rustc_hash::FxHashMap;

use crate::Scoping;

/// Owned version of ConstantValue that doesn't require arena lifetime
#[derive(Debug, Clone, PartialEq)]
pub enum NormalizedConstantValue {
    Number(f64),
    BigInt(BigInt),
    String(String),
    Boolean(bool),
    Computed,
}

impl std::fmt::Display for NormalizedConstantValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::BigInt(n) => write!(f, "{n}n"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Computed => write!(f, "Computed"),
        }
    }
}

impl<'a> From<ConstantValue<'a>> for NormalizedConstantValue {
    fn from(value: ConstantValue<'a>) -> Self {
        match value {
            ConstantValue::Number(n) => Self::Number(n),
            ConstantValue::BigInt(n) => Self::BigInt(n),
            ConstantValue::String(s) => Self::String(s.into_owned()),
            ConstantValue::Boolean(b) => Self::Boolean(b),
            ConstantValue::Undefined | ConstantValue::Null => Self::Computed,
        }
    }
}

/// Normalized const enum info without arena lifetime
#[derive(Debug, Clone)]
pub struct NormalizedConstEnumInfo {
    /// Symbol ID of the const enum
    pub symbol_id: SymbolId,
    /// Members of the const enum
    pub members: FxHashMap<SymbolId, NormalizedConstantValue>,

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

pub struct ConstantEnumCtx<'b, 'a: 'b> {
    const_enum_members: &'b FxHashMap<SymbolId, ConstantValue<'a>>,
    scoping: &'a Scoping,
    builder: oxc_ast::AstBuilder<'a>,
}

impl<'b, 'a> ConstantEnumCtx<'b, 'a> {
    pub fn new(
        const_enum_members: &'b FxHashMap<SymbolId, ConstantValue<'a>>,
        scoping: &'a Scoping,
        builder: oxc_ast::AstBuilder<'a>,
    ) -> Self {
        Self { const_enum_members, scoping, builder }
    }
}

impl<'a> GlobalContext<'a> for ConstantEnumCtx<'_, 'a> {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        ident
            .reference_id
            .get()
            .and_then(|reference_id| {
                let reference = self.scoping.references.get(reference_id)?;
                let symbol_id = reference.symbol_id()?;
                Some(!self.const_enum_members.contains_key(&symbol_id))
            })
            .unwrap_or(true)
    }

    fn get_constant_value_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        let reference = self.scoping.references.get(reference_id)?;
        let symbol_id = reference.symbol_id()?;
        self.const_enum_members.get(&symbol_id).cloned()
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for ConstantEnumCtx<'_, 'a> {
    fn annotations(&self) -> bool {
        true
    }

    fn manual_pure_functions(&self, _callee: &Expression) -> bool {
        false
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        PropertyReadSideEffects::All
    }

    fn unknown_global_side_effects(&self) -> bool {
        true
    }
}

impl<'a> ConstantEvaluationCtx<'a> for ConstantEnumCtx<'_, 'a> {
    fn ast(&self) -> oxc_ast::AstBuilder<'a> {
        self.builder
    }
}

/// Process a const enum declaration and evaluate its members
pub fn process_const_enum(
    enum_declaration: &TSEnumDeclaration<'_>,
    scoping: &Scoping,
    const_enum_table: &mut ConstEnumTable,
) {
    let symbol_id = enum_declaration.id.symbol_id();
    let current_scope = enum_declaration.scope_id();
    let allocator = oxc_allocator::Allocator::default();
    let ast_builder = AstBuilder::new(&allocator);
    let mut members = FxHashMap::default();
    let mut member_name_to_symbol_id = FxHashMap::default();
    let mut next_index: Option<f64> = Some(-1.0); // Start at -1, first auto-increment will make it 0

    for member in &enum_declaration.body.members {
        let member_name = match &member.id {
            TSEnumMemberName::Identifier(ident) => ident.name.as_str(),
            TSEnumMemberName::String(string) | TSEnumMemberName::ComputedString(string) => {
                string.value.as_str()
            }
            TSEnumMemberName::ComputedTemplateString(template) => {
                if template.expressions.is_empty() {
                    if let Some(quasi) = template.quasis.first() {
                        quasi.value.raw.as_str()
                    } else {
                        continue;
                    }
                } else {
                    // Skip template literals with expressions for now
                    continue;
                }
            }
        };
        let Some(member_symbol_id) = scoping.get_binding(current_scope, member_name) else {
            continue;
        };
        let value = if let Some(initializer) = &member.initializer {
            let ctx = ConstantEnumCtx::new(&members, scoping, ast_builder);
            let ret = initializer.evaluate_value(&ctx).unwrap_or(ConstantValue::Undefined);
            match &ret {
                ConstantValue::Number(n) => {
                    next_index = Some(*n);
                }
                _ => {
                    next_index = None;
                }
            }
            ret
        } else {
            match next_index.as_mut() {
                Some(n) => {
                    *n += 1.0;
                    ConstantValue::Number(*n)
                }
                None => ConstantValue::Undefined,
            }
        };

        member_name_to_symbol_id.insert(member_name.to_string(), member_symbol_id);

        members.insert(member_symbol_id, value);
    }

    let members = members
        .into_iter()
        .map(|(symbol_id, value)| (symbol_id, value.into()))
        .collect::<FxHashMap<SymbolId, NormalizedConstantValue>>();
    let enum_info = NormalizedConstEnumInfo { symbol_id, members, member_name_to_symbol_id };

    const_enum_table.add_enum(symbol_id, enum_info);
}
