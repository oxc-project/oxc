use crate::symbol::SymbolId;

#[derive(Debug, Clone)]
pub struct Reference {
    symbol_id: SymbolId,
}

impl Reference {
    pub fn new_read(symbol_id: SymbolId) -> Self {
        Self { symbol_id }
    }

    pub fn symbol_id(&self) -> SymbolId {
        self.symbol_id
    }
}
