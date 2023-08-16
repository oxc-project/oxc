use bitflags::bitflags;
use oxc_index::define_index_type;
use oxc_span::{Atom, Span};

use crate::{symbol::SymbolId, AstNodeId};

define_index_type! {
    pub struct ReferenceId = u32;
}

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

bitflags! {
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct ReferenceFlag: u8 {
        const None = 0;
        const Read = 1 << 0;
        const Write = 1 << 1;
        const ReadWrite = Self::Read.bits() | Self::Write.bits();
    }
}

impl ReferenceFlag {
    pub const fn read() -> Self {
        Self::Read
    }

    pub const fn write() -> Self {
        Self::Write
    }

    pub const fn read_write() -> Self {
        Self::ReadWrite
    }

    /// The identifier is read from. It may also be written to.
    pub const fn is_read(&self) -> bool {
        self.intersects(Self::Read)
    }

    /// The identifier is only read from.
    pub const fn is_read_only(&self) -> bool {
        self.contains(Self::Read)
    }

    /// The identifier is written to. It may also be read from.
    pub const fn is_write(&self) -> bool {
        self.intersects(Self::Write)
    }

    /// The identifier is only written to. It is not read from in this reference.
    pub const fn is_write_only(&self) -> bool {
        self.contains(Self::Write)
    }

    /// The identifier is both read from and written to, e.g `a += 1`.
    pub const fn is_read_write(&self) -> bool {
        self.contains(Self::ReadWrite)
    }
}
