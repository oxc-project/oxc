#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use oxc_index::Idx;
use oxc_span::Span;

use super::SymbolId;
use crate::node::AstNodeId;

#[derive(Debug, Clone)]
pub struct Reference {
    pub ast_node_id: AstNodeId,
    pub span: Span,

    flag: ReferenceFlag,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ReferenceFlag: u8 {
        const Read = 1 << 0;
        const Write = 1 << 1;
        const ReadWrite = Self::Read.bits() | Self::Write.bits();
    }
}

impl Reference {
    pub fn new(ast_node_id: AstNodeId, span: Span, flag: ReferenceFlag) -> Self {
        Self { ast_node_id, span, flag }
    }

    pub const fn is_read(&self) -> bool {
        self.flag.contains(ReferenceFlag::Read)
    }

    pub const fn is_write(&self) -> bool {
        self.flag.contains(ReferenceFlag::Write)
    }

    pub const fn is_read_write(&self) -> bool {
        self.flag.intersects(ReferenceFlag::ReadWrite)
    }

    pub fn resolve_to(self, symbol: SymbolId) -> ResolvedReference {
        ResolvedReference::new(self, symbol)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedReference {
    pub reference: Reference,
    // The Symbol the reference refers to.
    pub resolved_symbol_id: SymbolId,
}

impl ResolvedReference {
    pub fn new(reference: Reference, resolved_symbol_id: SymbolId) -> Self {
        Self { reference, resolved_symbol_id }
    }

    pub const fn is_read(&self) -> bool {
        self.reference.is_read()
    }

    pub const fn is_write(&self) -> bool {
        self.reference.is_write()
    }

    pub const fn is_read_write(&self) -> bool {
        self.reference.is_read_write()
    }

    pub fn span(&self) -> Span {
        self.reference.span
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ResolvedReferenceId(usize);

impl Idx for ResolvedReferenceId {
    fn new(idx: usize) -> Self {
        Self(idx)
    }

    fn index(self) -> usize {
        self.0
    }
}
