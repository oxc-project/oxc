use rustc_hash::FxHashMap;

use oxc_syntax::{constant_value::ConstantValue, scope::ScopeId, symbol::SymbolId};

/// Pre-computed enum member values and declaration-to-scope mappings.
///
/// Populated during semantic analysis for all enums (const and regular).
/// Used by the transformer to inline const enum member accesses.
#[derive(Clone, Default)]
pub struct EnumData {
    /// Computed constant values for enum members, keyed by member `SymbolId`.
    member_values: FxHashMap<SymbolId, ConstantValue>,
    /// Maps enum declaration `SymbolId` → body `ScopeId`s (one per declaration).
    body_scopes: FxHashMap<SymbolId, Vec<ScopeId>>,
}

impl EnumData {
    pub fn get_member_value(&self, symbol_id: SymbolId) -> Option<&ConstantValue> {
        self.member_values.get(&symbol_id)
    }

    pub(crate) fn set_member_value(&mut self, symbol_id: SymbolId, value: ConstantValue) {
        self.member_values.insert(symbol_id, value);
    }

    pub fn get_body_scopes(&self, symbol_id: SymbolId) -> Option<&[ScopeId]> {
        self.body_scopes.get(&symbol_id).map(Vec::as_slice)
    }

    pub(crate) fn add_body_scope(&mut self, symbol_id: SymbolId, scope_id: ScopeId) {
        self.body_scopes.entry(symbol_id).or_default().push(scope_id);
    }
}
