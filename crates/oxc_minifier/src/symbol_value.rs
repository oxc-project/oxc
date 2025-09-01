use rustc_hash::FxHashMap;

use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

#[derive(Debug, Default)]
pub enum SymbolValue<'a> {
    /// Initialized primitive constant value evaluated from expressions.
    Primitive(ConstantValue<'a>),
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

    #[expect(unused)]
    pub scope_id: ScopeId,
}

#[derive(Debug, Default)]
pub struct SymbolInformationMap<'a> {
    values: FxHashMap<SymbolId, SymbolInformation<'a>>,
}

impl<'a> SymbolInformationMap<'a> {
    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn init_value(&mut self, symbol_id: SymbolId, symbol_value: SymbolInformation<'a>) {
        self.values.insert(symbol_id, symbol_value);
    }

    pub fn set_constant_value(
        &mut self,
        symbol_id: SymbolId,
        symbol_value: Option<ConstantValue<'a>>,
    ) {
        let info = self.values.get_mut(&symbol_id).expect("symbol value must exist");
        if let Some(constant) = symbol_value {
            info.value = SymbolValue::Primitive(constant);
        }
    }

    pub fn get_symbol_value(&self, symbol_id: SymbolId) -> Option<&SymbolInformation<'a>> {
        self.values.get(&symbol_id)
    }
}
