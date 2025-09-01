use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

#[derive(Debug, Default)]
pub enum SymbolValue<'a> {
    /// Initialized primitive constant value.
    Primitive(ConstantValue<'a>),
    /// Initialized primitive value.
    /// This can be inlined within the same scope after the variable is declared.
    ScopedPrimitive(ConstantValue<'a>),
    /// Initialized scoped literal value.
    /// This can be inlined within the same scope after the variable is declared if it's only used once.
    ScopedLiteral,
    #[default]
    Unknown,
}

impl<'a> SymbolValue<'a> {
    pub fn to_constant_value(&self) -> Option<ConstantValue<'a>> {
        if let SymbolValue::Primitive(cv) = self { Some(cv.clone()) } else { None }
    }
}

#[derive(Debug)]
pub struct SymbolInformation<'a> {
    pub value: SymbolValue<'a>,

    /// Symbol is exported.
    pub exported: bool,

    pub read_references_count: u32,
    pub write_references_count: u32,

    pub scope_id: Option<ScopeId>,
}

#[derive(Debug, Default)]
pub struct SymbolInformationMap<'a> {
    values: FxHashMap<SymbolId, SymbolInformation<'a>>,
    inlineable_symbols: FxHashSet<SymbolId>,
}

impl<'a> SymbolInformationMap<'a> {
    pub fn clear(&mut self) {
        self.values.clear();
        self.inlineable_symbols.clear();
    }

    pub fn init_value(&mut self, symbol_id: SymbolId, symbol_value: SymbolInformation<'a>) {
        self.values.insert(symbol_id, symbol_value);
    }

    pub fn set_value(
        &mut self,
        symbol_id: SymbolId,
        symbol_value: SymbolValue<'a>,
        scope_id: ScopeId,
    ) {
        let info = self.values.get_mut(&symbol_id).expect("symbol value must exist");
        info.value = symbol_value;
        info.scope_id = Some(scope_id);
    }

    pub fn get_symbol_value(&self, symbol_id: SymbolId) -> Option<&SymbolInformation<'a>> {
        self.values.get(&symbol_id)
    }

    pub fn mark_symbol_inlineable(&mut self, symbol_id: SymbolId) {
        self.inlineable_symbols.insert(symbol_id);
    }

    pub fn get_inlineable_symbols(&self) -> &FxHashSet<SymbolId> {
        &self.inlineable_symbols
    }
}
