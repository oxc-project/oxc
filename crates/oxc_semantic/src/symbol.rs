#![allow(non_snake_case)] // Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`

use std::{fmt, hash};

use oxc_ast::ast::Expression;
use oxc_index::IndexVec;
use oxc_span::{CompactStr, Span};
pub use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use crate::{
    node::{AstNode, AstNodeId},
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
#[derive(Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify), serde(rename_all = "camelCase"))]
pub struct SymbolTable {
    pub spans: IndexVec<SymbolId, Span>,
    pub names: IndexVec<SymbolId, CompactStr>,
    pub flags: IndexVec<SymbolId, SymbolFlags>,
    pub scope_ids: IndexVec<SymbolId, ScopeId>,
    /// Pointer to the AST Node where this symbol is declared
    pub declarations: IndexVec<SymbolId, AstNodeId>,
    pub resolved_references: IndexVec<SymbolId, Vec<ReferenceId>>,
    pub references: IndexVec<ReferenceId, Reference>,
    pub redeclare_variables: IndexVec<SymbolId, Vec<Span>>,
}

impl SymbolTable {
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.spans.iter_enumerated().map(|(symbol_id, _)| symbol_id)
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.spans.iter_enumerated().rev().map(|(symbol_id, _)| symbol_id)
    }

    pub fn get_symbol_id_from_span(&self, span: &Span) -> Option<SymbolId> {
        self.spans
            .iter_enumerated()
            .find_map(|(symbol, inner_span)| if inner_span == span { Some(symbol) } else { None })
    }

    pub fn get_symbol_id_from_name(&self, name: &str) -> Option<SymbolId> {
        self.names.iter_enumerated().find_map(|(symbol, inner_name)| {
            if inner_name.as_str() == name {
                Some(symbol)
            } else {
                None
            }
        })
    }

    pub fn get_span(&self, symbol_id: SymbolId) -> Span {
        self.spans[symbol_id]
    }

    pub fn get_name(&self, symbol_id: SymbolId) -> &str {
        &self.names[symbol_id]
    }

    pub fn set_name(&mut self, symbol_id: SymbolId, name: CompactStr) {
        self.names[symbol_id] = name;
    }

    pub fn get_flag(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.flags[symbol_id]
    }

    pub fn get_redeclare_variables(&self, symbol_id: SymbolId) -> &Vec<Span> {
        &self.redeclare_variables[symbol_id]
    }

    pub fn union_flag(&mut self, symbol_id: SymbolId, includes: SymbolFlags) {
        self.flags[symbol_id] |= includes;
    }

    pub fn get_scope_id(&self, symbol_id: SymbolId) -> ScopeId {
        self.scope_ids[symbol_id]
    }

    pub fn get_scope_id_from_span(&self, span: &Span) -> Option<ScopeId> {
        self.get_symbol_id_from_span(span).map(|symbol_id| self.get_scope_id(symbol_id))
    }

    pub fn get_scope_id_from_name(&self, name: &str) -> Option<ScopeId> {
        self.get_symbol_id_from_name(name).map(|symbol_id| self.get_scope_id(symbol_id))
    }

    pub fn get_declaration(&self, symbol_id: SymbolId) -> AstNodeId {
        self.declarations[symbol_id]
    }

    pub fn create_symbol(
        &mut self,
        span: Span,
        name: CompactStr,
        flag: SymbolFlags,
        scope_id: ScopeId,
    ) -> SymbolId {
        self.spans.push(span);
        self.names.push(name);
        self.flags.push(flag);
        self.scope_ids.push(scope_id);
        self.resolved_references.push(vec![]);
        self.redeclare_variables.push(vec![])
    }

    pub fn add_declaration(&mut self, node_id: AstNodeId) {
        self.declarations.push(node_id);
    }

    pub fn add_redeclare_variable(&mut self, symbol_id: SymbolId, span: Span) {
        self.redeclare_variables[symbol_id].push(span);
    }

    pub fn create_reference(&mut self, reference: Reference) -> ReferenceId {
        self.references.push(reference)
    }

    pub fn get_reference(&self, reference_id: ReferenceId) -> &Reference {
        &self.references[reference_id]
    }

    pub fn get_reference_mut(&mut self, reference_id: ReferenceId) -> &mut Reference {
        &mut self.references[reference_id]
    }

    pub fn has_binding(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id().is_some()
    }

    pub fn is_global_reference(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id().is_none()
    }

    pub fn get_resolved_reference_ids(&self, symbol_id: SymbolId) -> &Vec<ReferenceId> {
        &self.resolved_references[symbol_id]
    }

    pub fn get_resolved_references(
        &self,
        symbol_id: SymbolId,
    ) -> impl Iterator<Item = &Reference> + '_ {
        self.resolved_references[symbol_id]
            .iter()
            .map(|reference_id| &self.references[*reference_id])
    }

    /// Determine whether evaluating the specific input `node` is a consequenceless reference. ie.
    /// evaluating it won't result in potentially arbitrary code from being ran. The following are
    /// allowed and determined not to cause side effects:
    ///
    ///  - `this` expressions
    ///  - `super` expressions
    ///  - Bound identifiers
    ///
    /// Reference:
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L557>
    pub fn is_static(&self, expr: &Expression) -> bool {
        match expr {
            Expression::ThisExpression(_) | Expression::Super(_) => true,
            Expression::Identifier(ident) => {
                ident.reference_id.get().map_or(false, |reference_id| {
                    self.get_reference(reference_id).symbol_id().map_or_else(
                        || self.has_binding(reference_id),
                        |symbol_id| self.get_resolved_references(symbol_id).all(|r| !r.is_write()),
                    )
                })
            }
            _ => false,
        }
    }
}

/// A unique symbol found in a parsed program.
///
/// A [`Symbol`] is a value bound to some name, called an identifier. In source
/// code, this identifier is a string, but because of scoping, [`Symbol`]s use a
/// clearer [`SymbolId`].
///
/// ```ts
/// let x = 1;         // <- 1
/// function foo() {   // <- 2
///     let x = 2;     // <- 3, shadowed
/// }
/// ```
///
/// ## Values vs Types
///
/// TypeScript interfaces, type parameters, type aliases, and so on are also
/// symbols and are stored in the [`SymbolTable`], alongside JavaScript
/// value-like symbols. You can tell which one a [`Symbol`] is by inspecting its [`SymbolFlags`].
///
/// ## Symbols in the AST
///
/// The AST makes it easy to differentiate which nodes declare a [`Symbol`] and
/// which reference one by having different kinds of nodes for each. Symbol
/// bindings are represented by [`BindingIdentifier`] nodes, which are usually
/// contained inside [`Declaration`]s
///
/// ## Representation
///
/// [`Symbol`]s are effectively just fat pointers. This has several implications
/// for their usage. For one, [`Clone`]s are extremely cheap. Despite this, they
/// intentionally do not implement [`Copy`] so that passing them around is more
/// obvious in consuming code. Secondly, they do not own any of their own data.
/// Most data is stored in the [`SymbolTable`], but some pieces are in
/// [`crate::AstNodes`] and [`crate::ScopeTree`]. All of these are tied to the
/// memory arena used during semantic analysis, making symbols neither [`Send`]
/// nor [`Sync`].
///
/// [`BindingIdentifier`]: oxc_ast::ast::BindingIdentifier
/// [`Declaration`]: oxc_ast::ast::Declaration
#[derive(Clone)]
#[non_exhaustive]
#[must_use]
pub struct Symbol<'s, 'a> {
    id: SymbolId,
    flags: SymbolFlags, // can be added w/o cost since id is 4 bytes
    semantic: &'s crate::Semantic<'a>,
}

impl<'s, 'a> Symbol<'s, 'a> {
    pub(crate) fn new(semantic: &'s crate::Semantic<'a>, symbol_id: SymbolId) -> Self {
        let flags = semantic.symbols().get_flag(symbol_id);
        Self { id: symbol_id, flags, semantic }
    }

    /// The unique identifier for this [`Symbol`].
    ///
    /// Although symbols are different across programs, their IDs are not.
    /// There are only so many IDs that can be fit in 4 bytes. Prefer
    /// [`PartialEq::eq`] for comparison if you're not certain that the same
    /// symbol table is being used.
    #[inline]
    pub fn id(&self) -> SymbolId {
        self.id
    }

    /// The [`ScopeId`] of the scope this [`Symbol`] was declared inside of.
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.semantic.symbols.get_scope_id(self.id)
    }

    /// Flags describing the scope this [`Symbol`] was declared in.
    #[inline]
    pub fn scope_flags(&self) -> crate::scope::ScopeFlags {
        self.semantic.scopes.get_flags(self.scope_id())
    }

    /// Flags describing what kind of value this [`Symbol`] is bound to and the
    /// nature of binding (an import, a const variable, etc.)
    #[inline]
    pub fn flags(&self) -> SymbolFlags {
        self.flags
    }

    /// The identifier name this [`Symbol`] is bound to.
    #[inline]
    pub fn name(&self) -> &str {
        self.semantic.symbols.get_name(self.id)
    }

    /// The [`Span`] for this [`Symbol`]'s declaration.
    #[inline]
    pub fn span(&self) -> oxc_span::Span {
        self.semantic.symbols.get_span(self.id)
    }

    /// The [`AstNodeId`] for the AST node that declared this [`Symbol`].
    ///
    /// This is usually a kind of [`Declaration`].
    ///
    /// [`Declaration`]: oxc_ast::ast::Declaration
    #[inline]
    pub fn declaration_id(&self) -> AstNodeId {
        self.semantic.symbols.get_declaration(self.id)
    }

    /// The [`AstNode`] that declared this [`Symbol`].
    ///
    /// This is usually a kind of [`Declaration`].
    ///
    /// [`Declaration`]: oxc_ast::ast::Declaration
    #[inline]
    pub fn declaration(&self) -> &AstNode<'a> {
        self.semantic.nodes.get_node(self.declaration_id())
    }

    /// Returns `true` if any references to this [`Symbol`] were found.
    #[inline]
    pub fn has_references(&self) -> bool {
        !self.semantic.symbols.resolved_references[self.id].is_empty()
    }

    /// Get IDs resolved [`Reference`]s to this [`Symbol`].
    ///
    /// This is a cheaper operation than [`Symbol::references`] and should be
    /// preferred wherever possible.
    #[inline]
    pub fn reference_ids(&self) -> &[ReferenceId] {
        self.semantic.symbols.get_resolved_reference_ids(self.id).as_slice()
    }

    /// Get resolved [`Reference`]s to this [`Symbol`].
    ///
    /// If you want to know how many references a symbol has, prefer
    /// [`Symbol::has_references`] or [`Symbol::reference_ids`].
    #[inline]
    pub fn references(&self) -> impl Iterator<Item = &Reference> + '_ {
        self.semantic.symbols.get_resolved_references(self.id)
    }
}

impl oxc_span::GetSpan for Symbol<'_, '_> {
    fn span(&self) -> Span {
        self.semantic.symbols.get_span(self.id)
    }
}

impl PartialEq for Symbol<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && (self.semantic as *const crate::Semantic<'_>)
                .eq(&(other.semantic as *const crate::Semantic<'_>))
    }
}

impl hash::Hash for Symbol<'_, '_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        let addr = self.semantic as *const _ as usize;
        state.write_usize(addr);
    }
}

impl fmt::Debug for Symbol<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Symbol")
            .field("id", &self.id)
            .field("name", &self.name())
            .field("symbol_flags", &self.flags())
            .field("declaration_node_id", &self.declaration_id())
            .field("scope_id", &self.scope_id())
            .field("references", &self.reference_ids())
            .finish()
    }
}

impl fmt::Display for Symbol<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}
