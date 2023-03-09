#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use oxc_ast::Span;

use super::SymbolId;
use crate::node::AstNodeId;

#[derive(Debug, Clone)]
pub struct Reference {
    pub ast_node_id: AstNodeId,
    pub span: Span,

    pub resolved_symbol_id: Option<SymbolId>,

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
    pub const fn new(ast_node_id: AstNodeId, span: Span, flag: ReferenceFlag) -> Self {
        Self { ast_node_id, span, resolved_symbol_id: None, flag }
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
}
