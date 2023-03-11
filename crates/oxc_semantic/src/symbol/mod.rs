//! Symbol and Symbol Table for tracking of semantics of variables
#![allow(non_upper_case_globals)]

mod id;
mod reference;
mod table;

use bitflags::bitflags;
use oxc_ast::{Atom, Span};

pub use self::{
    id::SymbolId,
    reference::{Reference, ReferenceFlag},
    table::SymbolTable,
};

#[derive(Debug)]
pub struct Symbol {
    id: SymbolId,
    name: Atom,
    span: Span,
    flags: SymbolFlags,
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[test]
fn symbol_size() {
    use std::mem::size_of;
    assert_eq!(size_of::<Symbol>(), 40);
}

bitflags! {
    #[derive(Default)]
    pub struct SymbolFlags: u16 {
        const None                    = 0;
        const Class                   = 1 << 5;

        const Value = Self::Class.bits;

        const ClassExcludes = Self::Value.bits;
    }
}

impl Symbol {
    #[must_use]
    pub fn new(id: SymbolId, name: Atom, span: Span, flags: SymbolFlags) -> Self {
        Self { id, name, span, flags }
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
    pub fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    #[allow(unused)]
    pub fn flags(&self) -> SymbolFlags {
        self.flags
    }
}
