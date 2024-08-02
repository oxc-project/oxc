// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

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
    node_id: AstNodeId,
    symbol_id: Option<SymbolId>,
    /// Describes how this referenced is used by other AST nodes. References can
    /// be reads, writes, or both.
    flag: ReferenceFlag,
}

impl Reference {
    #[inline]
    pub fn new(node_id: AstNodeId, flag: ReferenceFlag) -> Self {
        Self { node_id, symbol_id: None, flag }
    }

    #[inline]
    pub fn new_with_symbol_id(
        node_id: AstNodeId,
        symbol_id: SymbolId,
        flag: ReferenceFlag,
    ) -> Self {
        Self { node_id, symbol_id: Some(symbol_id), flag }
    }

    #[inline]
    pub fn node_id(&self) -> AstNodeId {
        self.node_id
    }

    #[inline]
    pub fn symbol_id(&self) -> Option<SymbolId> {
        self.symbol_id
    }

    #[inline]
    pub(crate) fn set_symbol_id(&mut self, symbol_id: SymbolId) {
        self.symbol_id = Some(symbol_id);
    }

    #[inline]
    pub fn flag(&self) -> &ReferenceFlag {
        &self.flag
    }

    #[inline]
    pub fn flag_mut(&mut self) -> &mut ReferenceFlag {
        &mut self.flag
    }

    /// Returns `true` if the identifier value was read. This is not mutually
    /// exclusive with [`#is_write`]
    #[inline]
    pub fn is_read(&self) -> bool {
        self.flag.is_read()
    }

    /// Returns `true` if the identifier was written to. This is not mutually
    /// exclusive with [`#is_read`]
    #[inline]
    pub fn is_write(&self) -> bool {
        self.flag.is_write()
    }

    #[inline]
    pub fn is_type(&self) -> bool {
        self.flag.is_type() || self.flag.is_ts_type_query()
    }
}
