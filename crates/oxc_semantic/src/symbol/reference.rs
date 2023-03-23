#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use oxc_ast::Span;

use super::SymbolId;
use crate::node::AstNodeId;

#[derive(Debug, Clone)]
pub struct Reference {
    pub ast_node_id: AstNodeId,
    pub span: Span,

    flag: ReferenceFlag,
}

bitflags! {
    #[derive(Default)]
    pub struct ReferenceFlag: u8 {
        const Read = 1 << 0;
        const Write = 1 << 1;
        const ReadWrite = Self::Read.bits | Self::Write.bits;
    }
}

impl Reference {
    #[must_use]
    pub fn new(ast_node_id: AstNodeId, span: Span, flag: ReferenceFlag) -> Self {
        Self { ast_node_id, span, flag }
    }

    #[must_use]
    pub const fn is_read(&self) -> bool {
        self.flag.contains(ReferenceFlag::Read)
    }

    #[must_use]
    pub const fn is_write(&self) -> bool {
        self.flag.contains(ReferenceFlag::Write)
    }

    #[must_use]
    pub const fn is_read_write(&self) -> bool {
        self.flag.intersects(ReferenceFlag::ReadWrite)
    }

    #[must_use]
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
    #[must_use]
    pub fn new(reference: Reference, resolved_symbol_id: SymbolId) -> Self {
        Self { reference, resolved_symbol_id }
    }

    #[must_use]
    pub const fn is_read(&self) -> bool {
        self.reference.is_read()
    }

    #[must_use]
    pub const fn is_write(&self) -> bool {
        self.reference.is_write()
    }

    #[must_use]
    pub const fn is_read_write(&self) -> bool {
        self.reference.is_read_write()
    }

    #[must_use]
    pub fn span(&self) -> Span {
        self.reference.span
    }
}
