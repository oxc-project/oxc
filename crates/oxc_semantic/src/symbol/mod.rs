//! Symbol and Symbol Table for tracking of semantics of variables

mod builder;
mod reference;
mod table;

use oxc_span::{Atom, Span};
pub use oxc_syntax::symbol::{SymbolFlags, SymbolId};

use self::reference::ResolvedReferenceId;
pub use self::{
    builder::SymbolTableBuilder,
    reference::{Reference, ReferenceFlag, ResolvedReference},
    table::SymbolTable,
};
use crate::node::AstNodeId;

#[derive(Debug)]
pub struct Symbol {
    id: SymbolId,
    /// Pointer to the AST Node where this symbol is declared
    declaration: AstNodeId,
    name: Atom,
    span: Span,
    flags: SymbolFlags,
    /// Pointers to the AST Nodes that reference this symbol
    references: Vec<ResolvedReferenceId>,
}

#[cfg(target_pointer_width = "64")]
mod size_asserts {
    use oxc_index::assert_eq_size;
    assert_eq_size!(super::Symbol, [u8; 80]);
}

impl Symbol {
    pub fn new(
        id: SymbolId,
        declaration: AstNodeId,
        name: Atom,
        span: Span,
        flags: SymbolFlags,
    ) -> Self {
        Self { id, declaration, name, span, flags, references: vec![] }
    }

    pub fn id(&self) -> SymbolId {
        self.id
    }

    pub fn name(&self) -> &Atom {
        &self.name
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn flags(&self) -> SymbolFlags {
        self.flags
    }

    pub fn is_const(&self) -> bool {
        self.flags.contains(SymbolFlags::ConstVariable)
    }

    pub fn is_class(&self) -> bool {
        self.flags.contains(SymbolFlags::Class)
    }

    pub fn is_export(&self) -> bool {
        self.flags.contains(SymbolFlags::Export)
    }

    pub fn references(&self) -> &[ResolvedReferenceId] {
        &self.references
    }

    pub fn declaration(&self) -> AstNodeId {
        self.declaration
    }
}
