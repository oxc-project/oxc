use std::{fmt, mem};

use oxc_allocator::{Allocator, FromIn, Vec as ArenaVec};
use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_index::{Idx, IndexVec};
use oxc_span::{Atom, Span};
use oxc_syntax::{
    node::NodeId,
    reference::{Reference, ReferenceId},
    scope::ScopeId,
    symbol::{RedeclarationId, SymbolFlags, SymbolId},
};

/// Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
///
/// Most symbols won't have redeclarations, so instead of storing `Vec<Span>` directly in
/// `redeclare_variables` (32 bytes per symbol), store `Option<RedeclarationId>` (4 bytes).
/// That ID indexes into `redeclarations` where the actual `Vec<Span>` is stored.
pub struct SymbolTable {
    pub(crate) spans: IndexVec<SymbolId, Span>,
    pub(crate) flags: IndexVec<SymbolId, SymbolFlags>,
    pub(crate) scope_ids: IndexVec<SymbolId, ScopeId>,
    /// Pointer to the AST Node where this symbol is declared
    pub(crate) declarations: IndexVec<SymbolId, NodeId>,
    redeclarations: IndexVec<SymbolId, Option<RedeclarationId>>,

    pub references: IndexVec<ReferenceId, Reference>,

    inner: SymbolTableCell,
}

impl fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("SymbolTable").finish()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        let allocator = Allocator::default();
        Self {
            spans: IndexVec::new(),
            flags: IndexVec::new(),
            scope_ids: IndexVec::new(),
            declarations: IndexVec::new(),
            redeclarations: IndexVec::new(),
            references: IndexVec::new(),
            inner: SymbolTableCell::new(allocator, |allocator| SymbolTableInner {
                names: ArenaVec::new_in(allocator),
                resolved_references: ArenaVec::new_in(allocator),
                redeclaration_spans: ArenaVec::new_in(allocator),
            }),
        }
    }
}

self_cell::self_cell!(
    struct SymbolTableCell {
        owner: Allocator,
        #[covariant]
        dependent: SymbolTableInner,
    }
);

struct SymbolTableInner<'cell> {
    names: ArenaVec<'cell, Atom<'cell>>,
    resolved_references: ArenaVec<'cell, ArenaVec<'cell, ReferenceId>>,
    redeclaration_spans: ArenaVec<'cell, ArenaVec<'cell, Span>>,
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

    pub fn names(&self) -> impl Iterator<Item = &str> + '_ {
        self.inner.borrow_dependent().names.iter().map(Atom::as_str)
    }

    pub fn resolved_references(&self) -> impl Iterator<Item = &ArenaVec<'_, ReferenceId>> + '_ {
        self.inner.borrow_dependent().resolved_references.iter()
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
        &self.inner.borrow_dependent().names[symbol_id.index()]
    }

    /// Rename a symbol.
    ///
    /// Returns the old name.
    #[inline]
    pub fn set_name(&mut self, symbol_id: SymbolId, name: &str) -> &str {
        self.inner
            .with_dependent_mut(|allocator, inner| {
                mem::replace(&mut inner.names[symbol_id.index()], Atom::from_in(name, allocator))
            })
            .as_str()
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
            &self.inner.borrow_dependent().redeclaration_spans[redeclaration_id.index()]
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
        name: &str,
        flags: SymbolFlags,
        scope_id: ScopeId,
        node_id: NodeId,
    ) -> SymbolId {
        self.spans.push(span);
        self.flags.push(flags);
        self.scope_ids.push(scope_id);
        self.declarations.push(node_id);
        self.inner.with_dependent_mut(|allocator, inner| {
            inner.names.push(Atom::from_in(name, allocator));
            inner.resolved_references.push(ArenaVec::new_in(allocator));
        });
        self.redeclarations.push(None)
    }

    pub fn add_redeclaration(&mut self, symbol_id: SymbolId, span: Span) {
        if let Some(redeclaration_id) = self.redeclarations[symbol_id] {
            self.inner.with_dependent_mut(|_, inner| {
                inner.redeclaration_spans[redeclaration_id.index()].push(span);
            });
        } else {
            self.inner.with_dependent_mut(|allocator, inner| {
                let mut v = ArenaVec::new_in(allocator);
                v.push(span);
                let redeclaration_id = inner.redeclaration_spans.len();
                inner.redeclaration_spans.push(v);
                self.redeclarations[symbol_id] =
                    Some(RedeclarationId::from_usize(redeclaration_id));
            });
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

    /// Get the name of the symbol a reference is resolved to. Returns `None` if the reference is
    /// not resolved.
    #[inline]
    pub fn get_reference_name(&self, reference_id: ReferenceId) -> Option<&str> {
        self.get_name(self.references[reference_id].symbol_id()?).into()
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
    pub fn get_resolved_reference_ids(&self, symbol_id: SymbolId) -> &ArenaVec<'_, ReferenceId> {
        &self.inner.borrow_dependent().resolved_references[symbol_id.index()]
    }

    /// Find [`Reference`]s resolved to a symbol.
    pub fn get_resolved_references(
        &self,
        symbol_id: SymbolId,
    ) -> impl DoubleEndedIterator<Item = &Reference> + '_ {
        self.get_resolved_reference_ids(symbol_id)
            .iter()
            .map(|&reference_id| &self.references[reference_id])
    }

    /// Get whether a symbol is mutated (i.e. assigned to).
    ///
    /// If symbol is `const`, always returns `false`.
    /// Otherwise, returns `true` if the symbol is assigned to somewhere in AST.
    pub fn symbol_is_mutated(&self, symbol_id: SymbolId) -> bool {
        if self.flags[symbol_id].contains(SymbolFlags::ConstVariable) {
            false
        } else {
            self.get_resolved_references(symbol_id).any(Reference::is_write)
        }
    }

    /// Add a reference to a symbol.
    pub fn add_resolved_reference(&mut self, symbol_id: SymbolId, reference_id: ReferenceId) {
        self.inner.with_dependent_mut(|_allocator, inner| {
            inner.resolved_references[symbol_id.index()].push(reference_id);
        });
    }

    /// Delete a reference to a symbol.
    ///
    /// # Panics
    /// Panics if provided `reference_id` is not a resolved reference for `symbol_id`.
    pub fn delete_resolved_reference(&mut self, symbol_id: SymbolId, reference_id: ReferenceId) {
        self.inner.with_dependent_mut(|_allocator, inner| {
            let reference_ids = &mut inner.resolved_references[symbol_id.index()];
            let index = reference_ids.iter().position(|&id| id == reference_id).unwrap();
            reference_ids.swap_remove(index);
        });
    }

    pub fn reserve(&mut self, additional_symbols: usize, additional_references: usize) {
        self.spans.reserve(additional_symbols);
        self.flags.reserve(additional_symbols);
        self.scope_ids.reserve(additional_symbols);
        self.declarations.reserve(additional_symbols);
        self.inner.with_dependent_mut(|_allocator, inner| {
            inner.names.reserve(additional_symbols);
            inner.resolved_references.reserve(additional_symbols);
        });
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

impl IsGlobalReference for IdentifierReference<'_> {
    fn is_global_reference(&self, symbols: &SymbolTable) -> bool {
        self.reference_id
            .get()
            .is_some_and(|reference_id| reference_id.is_global_reference(symbols))
    }

    fn is_global_reference_name(&self, name: &str, symbols: &SymbolTable) -> bool {
        self.name == name && self.is_global_reference(symbols)
    }
}

impl IsGlobalReference for Expression<'_> {
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
