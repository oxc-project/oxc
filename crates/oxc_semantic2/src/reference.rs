use oxc_index::define_index_type;
use oxc_span::Atom;

use crate::symbol::SymbolId;

define_index_type! {
    pub struct ReferenceId = u32;
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub name: Atom,
    pub symbol_id: Option<SymbolId>,
}

impl Reference {
    pub fn new(name: Atom) -> Self {
        Self { name, symbol_id: None }
    }
}
