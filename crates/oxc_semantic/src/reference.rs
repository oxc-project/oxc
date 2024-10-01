// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

pub use oxc_syntax::reference::{ReferenceFlags, ReferenceId};

use crate::{symbol::SymbolId, NodeId};

/// Describes where and how a Symbol is used in the AST.
///
/// References indicate how they are being used using [`ReferenceFlags`]. Refer
/// to the documentation for [`ReferenceFlags`] for more information.
///
/// ## Resolution
/// References to symbols that could be resolved have their `symbol_id` field
/// populated. [`None`] indicates that either a global variable or a
/// non-existent symbol is being referenced.
///
/// The node identified by `node_id` will be an [`IdentifierReference`].
/// Note that declarations do not count as references, even if the declaration
/// is being used in an expression.
///
/// ```ts
/// const arr = [1, 2, 3].map(function mapper(x) { return x + 1; });
/// //      Not considered a reference ^^^^^^
/// ```
///
/// [`IdentifierReference`]: oxc_ast::ast::IdentifierReference
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct Reference {
    /// The AST node making the reference.
    node_id: NodeId,
    /// The symbol being referenced.
    ///
    /// This will be [`None`] if no symbol could be found within
    /// the reference's scope tree. Usually this indicates a global variable or
    /// a reference to a non-existent symbol.
    symbol_id: Option<SymbolId>,
    /// Describes how this referenced is used by other AST nodes. References can
    /// be reads, writes, or both.
    flags: ReferenceFlags,
}

impl Reference {
    /// Create a new unresolved reference.
    #[inline]
    pub fn new(node_id: NodeId, flags: ReferenceFlags) -> Self {
        Self { node_id, symbol_id: None, flags }
    }

    /// Create a new resolved reference on a symbol.
    #[inline]
    pub fn new_with_symbol_id(node_id: NodeId, symbol_id: SymbolId, flags: ReferenceFlags) -> Self {
        Self { node_id, symbol_id: Some(symbol_id), flags }
    }

    /// Get the id of the node that is referencing the symbol.
    ///
    /// This will usually point to an [`IdentifierReference`] node, but it could
    /// be some specialized reference type like a [`JSXIdentifier`].
    ///
    /// [`IdentifierReference`]: oxc_ast::ast::IdentifierReference
    /// [`JSXIdentifier`]: oxc_ast::ast::JSXIdentifier
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get the id of the symbol being referenced.
    ///
    /// Will return [`None`] if the symbol could not be resolved.
    #[inline]
    pub fn symbol_id(&self) -> Option<SymbolId> {
        self.symbol_id
    }

    #[inline]
    pub(crate) fn set_symbol_id(&mut self, symbol_id: SymbolId) {
        self.symbol_id = Some(symbol_id);
    }

    #[inline]
    pub fn flags(&self) -> ReferenceFlags {
        self.flags
    }

    #[inline]
    pub fn flags_mut(&mut self) -> &mut ReferenceFlags {
        &mut self.flags
    }

    /// Returns `true` if the identifier value was read.
    ///
    /// This is not mutually exclusive with [`Reference::is_write`].
    #[inline]
    pub fn is_read(&self) -> bool {
        self.flags.is_read()
    }

    /// Returns `true` if the identifier was written to.
    ///
    /// This is not mutually exclusive with [`Reference::is_read`].
    #[inline]
    pub fn is_write(&self) -> bool {
        self.flags.is_write()
    }

    /// Returns `true` if this reference is used in a value context.
    pub fn is_value(&self) -> bool {
        self.flags.is_value()
    }

    /// Returns `true` if this reference is used in a type context.
    #[inline]
    pub fn is_type(&self) -> bool {
        self.flags.is_type()
    }
}
