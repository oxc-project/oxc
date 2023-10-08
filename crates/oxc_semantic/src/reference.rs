use oxc_span::{Atom, Span};

use crate::{symbol::SymbolId, AstNodeId};

pub use oxc_syntax::reference::{ReferenceFlag, ReferenceId};

#[derive(Debug, Clone)]
pub struct Reference {
    span: Span,
    /// The name of the identifier that was referred to
    name: Atom,
    node_id: AstNodeId,
    symbol_id: Option<SymbolId>,
    /// Describes how this referenced is used by other AST nodes. References can
    /// be reads, writes, or both.
    flag: ReferenceFlag,
}

impl Reference {
    pub fn new(span: Span, name: Atom, node_id: AstNodeId, flag: ReferenceFlag) -> Self {
        Self { span, name, node_id, symbol_id: None, flag }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Atom {
        &self.name
    }

    pub fn node_id(&self) -> AstNodeId {
        self.node_id
    }

    pub fn symbol_id(&self) -> Option<SymbolId> {
        self.symbol_id
    }

    pub(crate) fn set_symbol_id(&mut self, symbol_id: SymbolId) {
        self.symbol_id = Some(symbol_id);
    }

    /// Returns `true` if the identifier value was read. This is not mutually
    /// exclusive with [`#is_write`]
    pub fn is_read(&self) -> bool {
        self.flag.is_read()
    }

    /// Returns `true` if the identifier was written to. This is not mutually
    /// exclusive with [`#is_read`]
    pub fn is_write(&self) -> bool {
        self.flag.is_write()
    }
}
