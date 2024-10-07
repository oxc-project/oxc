#![allow(non_snake_case)] // Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`

#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_index::IndexVec;
use oxc_span::{CompactStr, Span};
pub use oxc_syntax::{
    scope::ScopeId,
    symbol::{RedeclarationId, SymbolFlags, SymbolId},
};

use crate::{
    node::NodeId,
    reference::{Reference, ReferenceId},
};

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type IndexVec<I, T> = Array<T>;
"#;

/// Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
///
/// Most symbols won't have redeclarations, so instead of storing `Vec<Span>` directly in
/// `redeclare_variables` (32 bytes per symbol), store `Option<RedeclarationId>` (4 bytes).
/// That ID indexes into `redeclarations` where the actual `Vec<Span>` is stored.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify), serde(rename_all = "camelCase"))]
pub struct SymbolTable {
    pub spans: IndexVec<SymbolId, Span>,
    pub names: IndexVec<SymbolId, CompactStr>,
    pub flags: IndexVec<SymbolId, SymbolFlags>,
    pub scope_ids: IndexVec<SymbolId, ScopeId>,
    /// Pointer to the AST Node where this symbol is declared
    pub declarations: IndexVec<SymbolId, NodeId>,
    pub resolved_references: IndexVec<SymbolId, Vec<ReferenceId>>,
    redeclarations: IndexVec<SymbolId, Option<RedeclarationId>>,

    redeclaration_spans: IndexVec<RedeclarationId, Vec<Span>>,

    pub references: IndexVec<ReferenceId, Reference>,
}

impl SymbolTable {
    /// Returns the number of symbols in this table.
    #[inline]
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    /// Returns `true` if this table contains no symbols.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Iterate over all symbol IDs in this table.
    ///
    /// Use [`ScopeTree::iter_bindings_in`] to only iterate over symbols declared in a specific
    /// scope.
    ///
    /// [`ScopeTree::iter_bindings_in`]: crate::scope::ScopeTree::iter_bindings_in
    ///
    /// ## Example
    ///
    /// ```
    /// use oxc_semantic::Semantic;
    /// let semantic: Semantic<'_> = parse_and_analyze("./foo.js");
    ///
    /// let classes = semantic
    ///     .scopes()
    ///     .symbol_ids()
    ///     .filter(|&symbol_id| {
    ///         let flags = semantic.symbols().get_flags(symbol_id);
    ///         flags.is_class()
    ///      })
    ///      .collect::<Vec<_>>();
    /// ```
    pub fn symbol_ids(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.spans.iter_enumerated().map(|(symbol_id, _)| symbol_id)
    }

    pub fn get_symbol_id_from_span(&self, span: Span) -> Option<SymbolId> {
        self.spans
            .iter_enumerated()
            .find_map(|(symbol, &inner_span)| if inner_span == span { Some(symbol) } else { None })
    }

    /// Get the [`Span`] of the [`AstNode`] declaring a symbol.
    ///
    /// [`AstNode`]: crate::node::AstNode
    #[inline]
    pub fn get_span(&self, symbol_id: SymbolId) -> Span {
        self.spans[symbol_id]
    }

    /// Get the identifier name a symbol is bound to.
    #[inline]
    pub fn get_name(&self, symbol_id: SymbolId) -> &str {
        &self.names[symbol_id]
    }

    #[inline]
    pub fn set_name(&mut self, symbol_id: SymbolId, name: CompactStr) {
        self.names[symbol_id] = name;
    }

    /// Get the [`SymbolFlags`] for a symbol, which describe how the symbol is declared.
    ///
    /// To find how a symbol is used, use [`SymbolTable::get_resolved_references`].
    #[inline]
    pub fn get_flags(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.flags[symbol_id]
    }

    /// Get a mutable reference to a symbol's [flags](SymbolFlags).
    #[inline]
    pub fn get_flags_mut(&mut self, symbol_id: SymbolId) -> &mut SymbolFlags {
        &mut self.flags[symbol_id]
    }

    #[inline]
    pub fn get_redeclarations(&self, symbol_id: SymbolId) -> &[Span] {
        if let Some(redeclaration_id) = self.redeclarations[symbol_id] {
            &self.redeclaration_spans[redeclaration_id]
        } else {
            static EMPTY: &[Span] = &[];
            EMPTY
        }
    }

    #[inline]
    pub fn union_flag(&mut self, symbol_id: SymbolId, includes: SymbolFlags) {
        self.flags[symbol_id] |= includes;
    }

    #[inline]
    pub fn set_scope_id(&mut self, symbol_id: SymbolId, scope_id: ScopeId) {
        self.scope_ids[symbol_id] = scope_id;
    }

    #[inline]
    pub fn get_scope_id(&self, symbol_id: SymbolId) -> ScopeId {
        self.scope_ids[symbol_id]
    }

    pub fn get_scope_id_from_span(&self, span: Span) -> Option<ScopeId> {
        self.get_symbol_id_from_span(span).map(|symbol_id| self.get_scope_id(symbol_id))
    }

    /// Get the ID of the AST node declaring a symbol.
    ///
    /// This node will be a [`VariableDeclaration`], [`Function`], or some other AST node
    /// that _has_ a [`BindingIdentifier`] or a [`BindingPattern`]. It will not point to the
    /// binding pattern or identifier node itself.
    ///
    /// [`VariableDeclaration`]: oxc_ast::ast::VariableDeclaration
    /// [`Function`]: oxc_ast::ast::Function
    /// [`BindingIdentifier`]: oxc_ast::ast::BindingIdentifier
    /// [`BindingPattern`]: oxc_ast::ast::BindingPattern
    #[inline]
    pub fn get_declaration(&self, symbol_id: SymbolId) -> NodeId {
        self.declarations[symbol_id]
    }

    pub fn create_symbol(
        &mut self,
        span: Span,
        name: CompactStr,
        flags: SymbolFlags,
        scope_id: ScopeId,
        node_id: NodeId,
    ) -> SymbolId {
        self.spans.push(span);
        self.names.push(name);
        self.flags.push(flags);
        self.scope_ids.push(scope_id);
        self.declarations.push(node_id);
        self.resolved_references.push(vec![]);
        self.redeclarations.push(None)
    }

    pub fn add_redeclaration(&mut self, symbol_id: SymbolId, span: Span) {
        if let Some(redeclaration_id) = self.redeclarations[symbol_id] {
            self.redeclaration_spans[redeclaration_id].push(span);
        } else {
            let redeclaration_id = self.redeclaration_spans.push(vec![span]);
            self.redeclarations[symbol_id] = Some(redeclaration_id);
        };
    }

    pub fn create_reference(&mut self, reference: Reference) -> ReferenceId {
        self.references.push(reference)
    }

    /// Get a resolved or unresolved reference.
    ///
    /// [`ReferenceId`]s can be found in [`IdentifierReference`] and similar nodes.
    #[inline]
    pub fn get_reference(&self, reference_id: ReferenceId) -> &Reference {
        &self.references[reference_id]
    }

    #[inline]
    pub fn get_reference_mut(&mut self, reference_id: ReferenceId) -> &mut Reference {
        &mut self.references[reference_id]
    }

    /// Returns `true` if the corresponding [`Reference`] is resolved to a symbol.
    ///
    /// When `false`, this could either be a reference to a global value or an identifier that does
    /// not exist.
    #[inline]
    pub fn has_binding(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id().is_some()
    }

    /// Find [`Reference`] ids resolved to a symbol.
    ///
    /// If you want direct access to a symbol's [`Reference`]s, use
    /// [`SymbolTable::get_resolved_references`].
    #[inline]
    pub fn get_resolved_reference_ids(&self, symbol_id: SymbolId) -> &Vec<ReferenceId> {
        &self.resolved_references[symbol_id]
    }

    /// Find [`Reference`]s resolved to a symbol.
    pub fn get_resolved_references(
        &self,
        symbol_id: SymbolId,
    ) -> impl DoubleEndedIterator<Item = &Reference> + '_ {
        self.resolved_references[symbol_id]
            .iter()
            .map(|&reference_id| &self.references[reference_id])
    }

    /// Delete a reference to a symbol.
    ///
    /// # Panics
    /// Panics if provided `reference_id` is not a resolved reference for `symbol_id`.
    pub fn delete_resolved_reference(&mut self, symbol_id: SymbolId, reference_id: ReferenceId) {
        let reference_ids = &mut self.resolved_references[symbol_id];
        let index = reference_ids.iter().position(|&id| id == reference_id).unwrap();
        reference_ids.swap_remove(index);
    }

    pub fn reserve(&mut self, additional_symbols: usize, additional_references: usize) {
        self.spans.reserve(additional_symbols);
        self.names.reserve(additional_symbols);
        self.flags.reserve(additional_symbols);
        self.scope_ids.reserve(additional_symbols);
        self.declarations.reserve(additional_symbols);
        self.resolved_references.reserve(additional_symbols);
        self.redeclarations.reserve(additional_symbols);

        self.references.reserve(additional_references);
    }
}

/// Checks whether the a identifier reference is a global value or not.
pub trait IsGlobalReference {
    fn is_global_reference(&self, _symbols: &SymbolTable) -> bool;
    fn is_global_reference_name(&self, name: &str, _symbols: &SymbolTable) -> bool;
}

impl IsGlobalReference for ReferenceId {
    fn is_global_reference(&self, symbols: &SymbolTable) -> bool {
        symbols.references[*self].symbol_id().is_none()
    }

    fn is_global_reference_name(&self, _name: &str, _symbols: &SymbolTable) -> bool {
        panic!("This function is pointless to be called.");
    }
}

impl<'a> IsGlobalReference for IdentifierReference<'a> {
    fn is_global_reference(&self, symbols: &SymbolTable) -> bool {
        self.reference_id
            .get()
            .is_some_and(|reference_id| reference_id.is_global_reference(symbols))
    }

    fn is_global_reference_name(&self, name: &str, symbols: &SymbolTable) -> bool {
        self.name == name && self.is_global_reference(symbols)
    }
}

impl<'a> IsGlobalReference for Expression<'a> {
    fn is_global_reference(&self, symbols: &SymbolTable) -> bool {
        if let Expression::Identifier(ident) = self {
            return ident.is_global_reference(symbols);
        }
        false
    }

    fn is_global_reference_name(&self, name: &str, symbols: &SymbolTable) -> bool {
        if let Expression::Identifier(ident) = self {
            return ident.is_global_reference_name(name, symbols);
        }
        false
    }
}
