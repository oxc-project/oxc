use rustc_hash::FxHashMap;

use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

#[derive(Debug)]
pub struct SymbolValue<'a> {
    /// Initialized constant value evaluated from expressions.
    /// `None` when the value is not a constant evaluated value.
    pub initialized_constant: Option<ConstantValue<'a>>,

    /// Symbol is exported.
    pub exported: bool,

    pub read_references_count: u32,
    pub write_references_count: u32,

    /// Number of read references that are member write targets (e.g. `a` in `a.foo = 1`).
    /// These reads exist only to access the object for a property write, not to use the value.
    /// Always <= `read_references_count`.
    pub member_write_target_read_count: u32,

    /// Whether the symbol's value is guaranteed fresh (cannot alias another binding).
    /// True for function/class declarations and variable declarations initialized
    /// with object/array/function/class literals.
    pub is_fresh_value: bool,

    pub scope_id: ScopeId,
}

#[derive(Debug, Default)]
pub struct SymbolValues<'a> {
    values: FxHashMap<SymbolId, SymbolValue<'a>>,
}

impl<'a> SymbolValues<'a> {
    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn init_value(&mut self, symbol_id: SymbolId, symbol_value: SymbolValue<'a>) {
        self.values.insert(symbol_id, symbol_value);
    }

    pub fn get_symbol_value(&self, symbol_id: SymbolId) -> Option<&SymbolValue<'a>> {
        self.values.get(&symbol_id)
    }
}
