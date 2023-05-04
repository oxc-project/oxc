#![allow(non_upper_case_globals)]

use std::num::NonZeroUsize;

use bitflags::bitflags;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolvedReferenceId(NonZeroUsize);

impl Default for ResolvedReferenceId {
    fn default() -> Self {
        Self::new(1)
    }
}

impl ResolvedReferenceId {
    #[must_use]
    pub fn new(n: usize) -> Self {
        unsafe { Self(NonZeroUsize::new_unchecked(n)) }
    }

    #[must_use]
    pub(crate) fn index0(self) -> usize {
        self.0.get() - 1
    }
}
