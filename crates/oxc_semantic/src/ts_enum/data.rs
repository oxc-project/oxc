use std::hash::{Hash, Hasher};

use hashbrown::{HashTable, hash_table::Entry};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

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
    /// Names with lone surrogates have no lexical binding. Keep their values separately.
    /// Hash and compare decoded units directly, without allocating a lookup key.
    utf16_member_values: HashTable<Utf16MemberValue>,
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

    pub(crate) fn get_utf16_member_value(
        &self,
        scope_id: ScopeId,
        name: JSStr<'_>,
    ) -> Option<&ConstantValue> {
        self.utf16_member_values
            .find(utf16_member_hash(scope_id, name.encode_utf16()), |entry| {
                entry.matches(scope_id, name)
            })
            .map(|entry| &entry.value)
    }

    pub(crate) fn set_utf16_member_value(
        &mut self,
        scope_id: ScopeId,
        name: JSStr<'_>,
        value: ConstantValue,
    ) {
        match self.utf16_member_values.entry(
            utf16_member_hash(scope_id, name.encode_utf16()),
            |entry| entry.matches(scope_id, name),
            |entry| utf16_member_hash(entry.scope_id, entry.name.iter().copied()),
        ) {
            Entry::Occupied(mut entry) => entry.get_mut().value = value,
            Entry::Vacant(entry) => {
                entry.insert(Utf16MemberValue {
                    scope_id,
                    name: name.encode_utf16().collect(),
                    value,
                });
            }
        }
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

#[derive(Clone)]
struct Utf16MemberValue {
    scope_id: ScopeId,
    name: Box<[u16]>,
    value: ConstantValue,
}

impl Utf16MemberValue {
    fn matches(&self, scope_id: ScopeId, name: JSStr<'_>) -> bool {
        self.scope_id == scope_id && name.encode_utf16().eq(self.name.iter().copied())
    }
}

/// Use the same hash for borrowed WTF-8 lookups and stored UTF-16 names, including on rehash.
fn utf16_member_hash(scope_id: ScopeId, units: impl Iterator<Item = u16>) -> u64 {
    let mut hasher = FxHasher::default();
    scope_id.hash(&mut hasher);
    let mut len = 0usize;
    for unit in units {
        unit.hash(&mut hasher);
        len += 1;
    }
    len.hash(&mut hasher);
    hasher.finish()
}
