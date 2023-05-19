use oxc_index::Idx;
use oxc_span::Atom;

use crate::symbol::SymbolId;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ReferenceId(usize);

impl Idx for ReferenceId {
    fn new(idx: usize) -> Self {
        Self(idx)
    }

    fn index(self) -> usize {
        self.0
    }
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
