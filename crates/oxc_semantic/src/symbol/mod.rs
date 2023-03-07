//! Symbol and Symbol Table for tracking of semantics of variables

pub mod id;
pub mod table;

use oxc_ast::{Atom, Span};

pub use self::{id::SymbolId, table::SymbolTable};

#[derive(Debug)]
pub struct Symbol {
    id: SymbolId,
    name: Atom,
    span: Span,
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[test]
fn symbol_size() {
    use std::mem::size_of;
    assert_eq!(size_of::<Symbol>(), 40);
}

impl Symbol {
    #[must_use]
    pub fn new(id: SymbolId, name: Atom, span: Span) -> Self {
        Self { id, name, span }
    }

    #[must_use]
    #[allow(unused)]
    pub fn id(&self) -> SymbolId {
        self.id
    }

    #[must_use]
    #[allow(unused)]
    pub fn name(&self) -> &Atom {
        &self.name
    }

    #[must_use]
    #[allow(unused)]
    pub fn span(&self) -> &Span {
        &self.span
    }
}
