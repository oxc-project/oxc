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

    #[expect(unused)]
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
