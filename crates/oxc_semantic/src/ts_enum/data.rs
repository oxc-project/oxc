use rustc_hash::{FxHashMap, FxHashSet};

use oxc_str::JSStr;

use oxc_syntax::{constant_value::ConstantValue, scope::ScopeId, symbol::SymbolId};

/// Pre-computed enum member values and declaration-to-scope mappings.
///
/// Populated during semantic analysis for all enums (const and regular).
/// Used by the transformer to inline const enum member accesses.
#[derive(Clone, Default)]
pub struct EnumData {
    /// Computed constant values for enum members, keyed by member `SymbolId`.
    member_values: FxHashMap<SymbolId, ConstantValue>,
    /// String-valued members whose name or value cannot be stored in the UTF-8 constant table.
    /// Keys use canonical WTF-8 so arbitrary JavaScript names retain their identity.
    string_members: FxHashMap<ScopeId, FxHashSet<Box<[u8]>>>,
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

    pub(crate) fn is_string_member(&self, scope_id: ScopeId, name: JSStr<'_>) -> bool {
        self.string_members.get(&scope_id).is_some_and(|names| names.contains(name.as_bytes()))
    }

    pub(crate) fn add_string_member(&mut self, scope_id: ScopeId, name: JSStr<'_>) {
        self.string_members.entry(scope_id).or_default().insert(name.as_bytes().into());
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
