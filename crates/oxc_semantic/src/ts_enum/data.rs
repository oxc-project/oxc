use rustc_hash::{FxHashMap, FxHashSet};

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
    /// `const enum` declaration symbols.
    ///
    /// Stored here rather than read from `SymbolFlags` so consumers can still tell
    /// const enums apart after the transformer has lowered them to `var`/`let`
    /// bindings and updated their symbol flags accordingly.
    const_enums: FxHashSet<SymbolId>,
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

    pub fn is_const_enum(&self, symbol_id: SymbolId) -> bool {
        self.const_enums.contains(&symbol_id)
    }

    pub(crate) fn add_const_enum(&mut self, symbol_id: SymbolId) {
        self.const_enums.insert(symbol_id);
    }
}
