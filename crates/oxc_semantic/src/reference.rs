// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_span::{CompactStr, Span};
pub use oxc_syntax::reference::{ReferenceFlag, ReferenceId};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use crate::{symbol::SymbolId, AstNodeId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct Reference {
    span: Span,
    /// The name of the identifier that was referred to
    name: CompactStr,
    node_id: AstNodeId,
    symbol_id: Option<SymbolId>,
    /// Describes how this referenced is used by other AST nodes. References can
    /// be reads, writes, or both.
    flag: ReferenceFlag,
}

impl Reference {
    pub fn new(span: Span, name: CompactStr, node_id: AstNodeId, flag: ReferenceFlag) -> Self {
        Self { span, name, node_id, symbol_id: None, flag }
    }

    pub fn new_with_symbol_id(
        span: Span,
        name: CompactStr,
        node_id: AstNodeId,
        symbol_id: SymbolId,
        flag: ReferenceFlag,
    ) -> Self {
        Self { span, name, node_id, symbol_id: Some(symbol_id), flag }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &CompactStr {
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

    pub fn flag(&self) -> &ReferenceFlag {
        &self.flag
    }

    pub fn flag_mut(&mut self) -> &mut ReferenceFlag {
        &mut self.flag
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

    pub fn is_type(&self) -> bool {
        self.flag.is_type()
    }
}
