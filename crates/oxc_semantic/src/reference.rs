use oxc_index::define_index_type;
use oxc_span::{Atom, Span};

use crate::symbol::SymbolId;

define_index_type! {
    pub struct ReferenceId = u32;
}

#[derive(Debug, Clone)]
pub struct Reference {
    span: Span,
    name: Atom,
    symbol_id: Option<SymbolId>,
    flag: ReferenceFlag,
}

impl Reference {
    pub fn new(span: Span, name: Atom, flag: ReferenceFlag) -> Self {
        Self { span, name, symbol_id: None, flag }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Atom {
        &self.name
    }

    pub fn symbol_id(&self) -> Option<SymbolId> {
        self.symbol_id
    }

    pub(crate) fn set_symbol_id(&mut self, symbol_id: SymbolId) {
        self.symbol_id = Some(symbol_id);
    }

    pub fn is_read(&self) -> bool {
        self.flag == ReferenceFlag::Read
    }

    pub fn is_write(&self) -> bool {
        self.flag == ReferenceFlag::Write
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReferenceFlag {
    None,
    Read,
    Write,
}

impl ReferenceFlag {
    pub fn read() -> Self {
        Self::Read
    }

    pub fn write() -> Self {
        Self::Write
    }
}
