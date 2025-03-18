use std::{fmt, mem};

use rustc_hash::FxHashSet;

use oxc_allocator::{Allocator, FromIn, HashMap as ArenaHashMap, Vec as ArenaVec};
use oxc_index::{Idx, IndexVec};
use oxc_span::{Atom, Span};
use oxc_syntax::{
    node::NodeId,
    reference::{Reference, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{RedeclarationId, SymbolFlags, SymbolId},
};

pub type Bindings<'a> = ArenaHashMap<'a, &'a str, SymbolId>;
pub type UnresolvedReferences<'a> = ArenaHashMap<'a, &'a str, ArenaVec<'a, ReferenceId>>;

/// # Symbol Table and Scope Tree
///
/// ## Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
///
/// Most symbols won't have redeclarations, so instead of storing `Vec<Span>` directly in
/// `redeclare_variables` (32 bytes per symbol), store `Option<RedeclarationId>` (4 bytes).
/// That ID indexes into `redeclarations` where the actual `Vec<Span>` is stored.
///
/// ## Scope Tree
///
/// The scope tree stores lexical scopes created by a program, and all the
/// variable bindings each scope creates.
///
/// - All scopes have a parent scope, except the root scope.
/// - Scopes can have 0 or more child scopes.
/// - Nodes that create a scope store the [`ScopeId`] of the scope they create.
pub struct Scoping {
    /* Symbol Table Fields */
    pub(crate) symbol_spans: IndexVec<SymbolId, Span>,
    pub(crate) symbol_flags: IndexVec<SymbolId, SymbolFlags>,
    pub(crate) symbol_scope_ids: IndexVec<SymbolId, ScopeId>,
    /// Pointer to the AST Node where this symbol is declared
    pub(crate) symbol_declarations: IndexVec<SymbolId, NodeId>,
    symbol_redeclarations: IndexVec<SymbolId, Option<RedeclarationId>>,

    pub(crate) references: IndexVec<ReferenceId, Reference>,

    /// Symbols that are used as the name property of a function.
    function_names: FxHashSet<SymbolId>,
    /// Symbols that are used as the name property of a class.
    class_names: FxHashSet<SymbolId>,

    /// Function or Variable Symbol IDs that are marked with `@__NO_SIDE_EFFECTS__`.
    pub(crate) no_side_effects: FxHashSet<SymbolId>,

    /* Scope Tree Fields */
    /// Maps a scope to the parent scope it belongs in.
    scope_parent_ids: IndexVec<ScopeId, Option<ScopeId>>,

    /// Runtime flag for constructing child_ids.
    pub(crate) scope_build_child_ids: bool,

    /// Maps a scope to its node id.
    scope_node_ids: IndexVec<ScopeId, NodeId>,

    scope_flags: IndexVec<ScopeId, ScopeFlags>,

    pub(crate) cell: ScopingCell,
}

impl fmt::Debug for Scoping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Scoping").finish()
    }
}

impl Default for Scoping {
    fn default() -> Self {
        Self {
            symbol_spans: IndexVec::new(),
            symbol_flags: IndexVec::new(),
            symbol_scope_ids: IndexVec::new(),
            symbol_declarations: IndexVec::new(),
            symbol_redeclarations: IndexVec::new(),
            references: IndexVec::new(),
            function_names: FxHashSet::default(),
            class_names: FxHashSet::default(),
            no_side_effects: FxHashSet::default(),
            scope_parent_ids: IndexVec::new(),
            scope_build_child_ids: false,
            scope_node_ids: IndexVec::new(),
            scope_flags: IndexVec::new(),
            cell: ScopingCell::new(Allocator::default(), |allocator| ScopingInner {
                symbol_names: ArenaVec::new_in(allocator),
                resolved_references: ArenaVec::new_in(allocator),
                redeclaration_spans: ArenaVec::new_in(allocator),
                bindings: IndexVec::new(),
                scope_child_ids: ArenaVec::new_in(allocator),
                root_unresolved_references: UnresolvedReferences::new_in(allocator),
            }),
        }
    }
}

self_cell::self_cell!(
    pub struct ScopingCell {
        owner: Allocator,
        #[covariant]
        dependent: ScopingInner,
    }
);

pub struct ScopingInner<'cell> {
    /* Symbol Table Fields */
    symbol_names: ArenaVec<'cell, Atom<'cell>>,
    resolved_references: ArenaVec<'cell, ArenaVec<'cell, ReferenceId>>,
    redeclaration_spans: ArenaVec<'cell, ArenaVec<'cell, Span>>,
    /* Scope Tree Fields */
    /// Symbol bindings in a scope.
    ///
    /// A binding is a mapping from an identifier name to its [`SymbolId`]
    pub(crate) bindings: IndexVec<ScopeId, Bindings<'cell>>,

    /// Maps a scope to direct children scopes.
    scope_child_ids: ArenaVec<'cell, ArenaVec<'cell, ScopeId>>,

    pub(crate) root_unresolved_references: UnresolvedReferences<'cell>,
}

// Symbol Table Methods
impl Scoping {
    /// Returns the number of symbols in this table.
    #[inline]
    pub fn symbols_len(&self) -> usize {
        self.symbol_spans.len()
    }

    /// Returns `true` if this table contains no symbols.
    #[inline]
    pub fn symbols_is_empty(&self) -> bool {
        self.symbol_spans.is_empty()
    }

    pub fn symbol_names(&self) -> impl Iterator<Item = &str> + '_ {
        self.cell.borrow_dependent().symbol_names.iter().map(Atom::as_str)
    }

    pub fn resolved_references(&self) -> impl Iterator<Item = &ArenaVec<'_, ReferenceId>> + '_ {
        self.cell.borrow_dependent().resolved_references.iter()
    }

    /// Iterate over all symbol IDs in this table.
    ///
    /// Use [`ScopeTree::iter_bindings_in`] to only iterate over symbols declared in a specific
    /// scope.
    ///
    /// [`ScopeTree::iter_bindings_in`]: crate::scoping::Scoping::iter_bindings_in
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
    ///         let flags = semantic.scoping().get_flags(symbol_id);
    ///         flags.is_class()
    ///      })
    ///      .collect::<Vec<_>>();
    /// ```
    pub fn symbol_ids(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.symbol_spans.iter_enumerated().map(|(symbol_id, _)| symbol_id)
    }

    /// Get the [`Span`] of the [`AstNode`] declaring a symbol.
    ///
    /// [`AstNode`]: crate::node::AstNode
    #[inline]
    pub fn symbol_span(&self, symbol_id: SymbolId) -> Span {
        self.symbol_spans[symbol_id]
    }

    /// Get the identifier name a symbol is bound to.
    #[inline]
    pub fn symbol_name(&self, symbol_id: SymbolId) -> &str {
        &self.cell.borrow_dependent().symbol_names[symbol_id.index()]
    }

    /// Rename a symbol.
    ///
    /// Returns the old name.
    #[inline]
    pub fn set_symbol_name(&mut self, symbol_id: SymbolId, name: &str) -> &str {
        self.cell
            .with_dependent_mut(|allocator, cell| {
                mem::replace(
                    &mut cell.symbol_names[symbol_id.index()],
                    Atom::from_in(name, allocator),
                )
            })
            .as_str()
    }

    /// Get the [`SymbolFlags`] for a symbol, which describe how the symbol is declared.
    ///
    /// To find how a symbol is used, use [`Scoping::get_resolved_references`].
    #[inline]
    pub fn symbol_flags(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.symbol_flags[symbol_id]
    }

    /// Get a mutable reference to a symbol's [flags](SymbolFlags).
    #[inline]
    pub fn symbol_flags_mut(&mut self, symbol_id: SymbolId) -> &mut SymbolFlags {
        &mut self.symbol_flags[symbol_id]
    }

    #[inline]
    pub fn symbol_redeclarations(&self, symbol_id: SymbolId) -> &[Span] {
        if let Some(redeclaration_id) = self.symbol_redeclarations[symbol_id] {
            &self.cell.borrow_dependent().redeclaration_spans[redeclaration_id.index()]
        } else {
            static EMPTY: &[Span] = &[];
            EMPTY
        }
    }

    #[inline]
    pub fn union_symbol_flag(&mut self, symbol_id: SymbolId, includes: SymbolFlags) {
        self.symbol_flags[symbol_id] |= includes;
    }

    #[inline]
    pub fn set_symbol_scope_id(&mut self, symbol_id: SymbolId, scope_id: ScopeId) {
        self.symbol_scope_ids[symbol_id] = scope_id;
    }

    #[inline]
    pub fn symbol_scope_id(&self, symbol_id: SymbolId) -> ScopeId {
        self.symbol_scope_ids[symbol_id]
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
    pub fn symbol_declaration(&self, symbol_id: SymbolId) -> NodeId {
        self.symbol_declarations[symbol_id]
    }

    pub fn create_symbol(
        &mut self,
        span: Span,
        name: &str,
        flags: SymbolFlags,
        scope_id: ScopeId,
        node_id: NodeId,
    ) -> SymbolId {
        self.symbol_spans.push(span);
        self.symbol_flags.push(flags);
        self.symbol_scope_ids.push(scope_id);
        self.symbol_declarations.push(node_id);
        self.cell.with_dependent_mut(|allocator, cell| {
            cell.symbol_names.push(Atom::from_in(name, allocator));
            cell.resolved_references.push(ArenaVec::new_in(allocator));
        });
        self.symbol_redeclarations.push(None)
    }

    pub fn add_symbol_redeclaration(&mut self, symbol_id: SymbolId, span: Span) {
        if let Some(redeclaration_id) = self.symbol_redeclarations[symbol_id] {
            self.cell.with_dependent_mut(|_, cell| {
                cell.redeclaration_spans[redeclaration_id.index()].push(span);
            });
        } else {
            self.cell.with_dependent_mut(|allocator, cell| {
                let v = ArenaVec::from_array_in([span], allocator);
                let redeclaration_id = cell.redeclaration_spans.len();
                cell.redeclaration_spans.push(v);
                self.symbol_redeclarations[symbol_id] =
                    Some(RedeclarationId::from_usize(redeclaration_id));
            });
        };
    }

    pub fn create_reference(&mut self, reference: Reference) -> ReferenceId {
        self.references.push(reference)
    }

    /// Get a resolved or unresolved reference.
    ///
    /// [`ReferenceId`]s can be found in [oxc_ast::ast::IdentifierReference] and similar nodes.
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
        self.symbol_name(self.references[reference_id].symbol_id()?).into()
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
    /// If you want direct access to a symbol's [`Reference`]s, use [`Scoping::get_resolved_references`].
    #[inline]
    pub fn get_resolved_reference_ids(&self, symbol_id: SymbolId) -> &ArenaVec<'_, ReferenceId> {
        &self.cell.borrow_dependent().resolved_references[symbol_id.index()]
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
        if self.symbol_flags[symbol_id].contains(SymbolFlags::ConstVariable) {
            false
        } else {
            self.get_resolved_references(symbol_id).any(Reference::is_write)
        }
    }

    /// Get whether a symbol is used (i.e. read or written after declaration).
    pub fn symbol_is_used(&self, symbol_id: SymbolId) -> bool {
        self.get_resolved_references(symbol_id).count() > 0
    }

    /// Add a reference to a symbol.
    pub fn add_resolved_reference(&mut self, symbol_id: SymbolId, reference_id: ReferenceId) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            cell.resolved_references[symbol_id.index()].push(reference_id);
        });
    }

    /// Delete a reference to a symbol.
    ///
    /// # Panics
    /// Panics if provided `reference_id` is not a resolved reference for `symbol_id`.
    pub fn delete_resolved_reference(&mut self, symbol_id: SymbolId, reference_id: ReferenceId) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            let reference_ids = &mut cell.resolved_references[symbol_id.index()];
            let index = reference_ids.iter().position(|&id| id == reference_id).unwrap();
            reference_ids.swap_remove(index);
        });
    }

    pub fn reserve(
        &mut self,
        additional_symbols: usize,
        additional_references: usize,
        additional_scopes: usize,
    ) {
        self.symbol_spans.reserve(additional_symbols);
        self.symbol_flags.reserve(additional_symbols);
        self.symbol_scope_ids.reserve(additional_symbols);
        self.symbol_declarations.reserve(additional_symbols);
        self.cell.with_dependent_mut(|_allocator, cell| {
            cell.symbol_names.reserve(additional_symbols);
            cell.resolved_references.reserve(additional_symbols);
        });
        self.references.reserve(additional_references);

        self.scope_parent_ids.reserve(additional_scopes);
        self.scope_flags.reserve(additional_scopes);
        self.cell.with_dependent_mut(|_allocator, cell| {
            cell.bindings.reserve(additional_scopes);
        });
        self.scope_node_ids.reserve(additional_scopes);
        if self.scope_build_child_ids {
            self.cell.with_dependent_mut(|_allocator, cell| {
                cell.scope_child_ids.reserve(additional_scopes);
            });
        }
    }

    pub fn no_side_effects(&self) -> &FxHashSet<SymbolId> {
        &self.no_side_effects
    }
}

/// Scope Tree Methods
impl Scoping {
    const ROOT_SCOPE_ID: ScopeId = ScopeId::new(0);

    /// Returns the number of scopes found in the program. Includes the root
    /// program scope.
    #[inline]
    pub fn scopes_len(&self) -> usize {
        self.scope_parent_ids.len()
    }

    /// Returns `true` if there are no scopes in the program.
    ///
    /// This will always return `false` when semantic analysis has completed
    /// since there is a root scope.
    #[inline]
    pub fn scopes_is_empty(&self) -> bool {
        self.scope_parent_ids.is_empty()
    }

    /// Iterate over the scopes that contain a scope.
    ///
    /// The first element of this iterator will be the scope itself. This
    /// guarantees the iterator will have at least 1 element.
    pub fn scope_ancestors(&self, scope_id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        std::iter::successors(Some(scope_id), |&scope_id| self.scope_parent_ids[scope_id])
    }

    pub fn scope_descendants_from_root(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.scope_parent_ids.iter_enumerated().map(|(scope_id, _)| scope_id)
    }

    /// Get the root [`Program`] scope id.
    ///
    /// [`Program`]: oxc_ast::ast::Program
    #[inline]
    pub const fn root_scope_id(&self) -> ScopeId {
        Self::ROOT_SCOPE_ID
    }

    /// Get the flags for the root scope.
    ///
    /// This is a shorthand for `scope.get_flags(scope.root_scope_id())`.
    #[inline]
    pub fn root_scope_flags(&self) -> ScopeFlags {
        self.scope_flags[self.root_scope_id()]
    }

    #[inline]
    pub fn root_unresolved_references(&self) -> &UnresolvedReferences {
        &self.cell.borrow_dependent().root_unresolved_references
    }

    pub fn root_unresolved_references_ids(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = ReferenceId> + '_> + '_ {
        self.cell.borrow_dependent().root_unresolved_references.values().map(|v| v.iter().copied())
    }

    pub(crate) fn set_root_unresolved_references<'a>(
        &mut self,
        entries: impl Iterator<Item = (&'a str, Vec<ReferenceId>)>,
    ) {
        self.cell.with_dependent_mut(|allocator, cell| {
            for (k, v) in entries {
                let k = allocator.alloc_str(k);
                let v = ArenaVec::from_iter_in(v, allocator);
                cell.root_unresolved_references.insert(k, v);
            }
        });
    }

    /// Delete an unresolved reference.
    ///
    /// If the `ReferenceId` provided is only reference remaining for this unresolved reference
    /// (i.e. this `x` was last `x` in the AST), deletes the key from `root_unresolved_references`.
    ///
    /// # Panics
    /// Panics if there is no unresolved reference for provided `name` and `reference_id`.
    #[inline]
    pub fn delete_root_unresolved_reference(&mut self, name: &str, reference_id: ReferenceId) {
        // It would be better to use `Entry` API to avoid 2 hash table lookups when deleting,
        // but `map.entry` requires an owned key to be provided. Currently we use `CompactStr`s as keys
        // which are not cheap to construct, so this is best we can do at present.
        // TODO: Switch to `Entry` API once we use `&str`s or `Atom`s as keys.
        self.cell.with_dependent_mut(|_allocator, cell| {
            let reference_ids = cell.root_unresolved_references.get_mut(name).unwrap();
            if reference_ids.len() == 1 {
                assert_eq!(reference_ids[0], reference_id);
                cell.root_unresolved_references.remove(name);
            } else {
                let index = reference_ids.iter().position(|&id| id == reference_id).unwrap();
                reference_ids.swap_remove(index);
            }
        });
    }

    #[inline]
    pub fn scope_flags(&self, scope_id: ScopeId) -> ScopeFlags {
        self.scope_flags[scope_id]
    }

    #[inline]
    pub fn scope_flags_mut(&mut self, scope_id: ScopeId) -> &mut ScopeFlags {
        &mut self.scope_flags[scope_id]
    }

    /// Get [`ScopeFlags`] for a new child scope under `parent_scope_id`.
    pub fn get_new_scope_flags(&self, flags: ScopeFlags, parent_scope_id: ScopeId) -> ScopeFlags {
        // https://tc39.es/ecma262/#sec-strict-mode-code
        flags | self.scope_flags(parent_scope_id) & ScopeFlags::StrictMode
    }

    #[inline]
    pub fn scope_parent_id(&self, scope_id: ScopeId) -> Option<ScopeId> {
        self.scope_parent_ids[scope_id]
    }

    pub fn set_scope_parent_id(&mut self, scope_id: ScopeId, parent_id: Option<ScopeId>) {
        self.scope_parent_ids[scope_id] = parent_id;
        if self.scope_build_child_ids {
            // Set this scope as child of parent scope
            if let Some(parent_id) = parent_id {
                self.cell.with_dependent_mut(|_allocator, cell| {
                    cell.scope_child_ids[parent_id.index()].push(scope_id);
                });
            }
        }
    }

    /// Change the parent scope of a scope.
    ///
    /// This will also remove the scope from the child list of the old parent and add it to the new parent.
    pub fn change_scope_parent_id(&mut self, scope_id: ScopeId, new_parent_id: Option<ScopeId>) {
        let old_parent_id = mem::replace(&mut self.scope_parent_ids[scope_id], new_parent_id);
        if self.scope_build_child_ids {
            self.cell.with_dependent_mut(|_allocator, cell| {
                // Remove this scope from old parent scope
                if let Some(old_parent_id) = old_parent_id {
                    cell.scope_child_ids[old_parent_id.index()]
                        .retain(|&child_id| child_id != scope_id);
                }
                // And add it to new parent scope
                if let Some(parent_id) = new_parent_id {
                    cell.scope_child_ids[parent_id.index()].push(scope_id);
                }
            });
        }
    }

    /// Delete a scope.
    pub fn delete_scope(&mut self, scope_id: ScopeId) {
        if self.scope_build_child_ids {
            self.cell.with_dependent_mut(|_allocator, cell| {
                cell.scope_child_ids[scope_id.index()].clear();
                let parent_id = self.scope_parent_ids[scope_id];
                if let Some(parent_id) = parent_id {
                    cell.scope_child_ids[parent_id.index()]
                        .retain(|&child_id| child_id != scope_id);
                }
            });
        }
    }

    /// Get a variable binding by name that was declared in the top-level scope
    #[inline]
    pub fn get_root_binding(&self, name: &str) -> Option<SymbolId> {
        self.get_binding(self.root_scope_id(), name)
    }

    pub fn add_root_unresolved_reference(&mut self, name: &str, reference_id: ReferenceId) {
        self.cell.with_dependent_mut(|allocator, cell| {
            let name = allocator.alloc_str(name);
            cell.root_unresolved_references
                .entry(name)
                .or_insert_with(|| ArenaVec::new_in(allocator))
                .push(reference_id);
        });
    }

    /// Check if a symbol is declared in a certain scope.
    pub fn scope_has_binding(&self, scope_id: ScopeId, name: &str) -> bool {
        self.cell.borrow_dependent().bindings[scope_id].contains_key(name)
    }

    /// Get the symbol bound to an identifier name in a scope.
    ///
    /// Returns [`None`] if that name is not bound in the scope. This could be
    /// because the symbol is not declared within this tree, but it could also
    /// be because its declaration is in a parent scope. If you want to find a
    /// binding that might be declared in a parent scope, use [`find_binding`].
    ///
    /// [`find_binding`]: Scoping::find_binding
    pub fn get_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        self.cell.borrow_dependent().bindings[scope_id].get(name).copied()
    }

    /// Find a binding by name in a scope or its ancestors.
    ///
    /// Bindings are resolved by walking up the scope tree until a binding is
    /// found. If no binding is found, [`None`] is returned.
    pub fn find_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        for scope_id in self.scope_ancestors(scope_id) {
            if let Some(symbol_id) = self.get_binding(scope_id, name) {
                return Some(symbol_id);
            }
        }
        None
    }

    /// Get all bound identifiers in a scope.
    #[inline]
    pub fn get_bindings(&self, scope_id: ScopeId) -> &Bindings {
        &self.cell.borrow_dependent().bindings[scope_id]
    }

    /// Get the ID of the [`AstNode`] that created a scope.
    ///
    /// [`AstNode`]: crate::AstNode
    #[inline]
    pub fn get_node_id(&self, scope_id: ScopeId) -> NodeId {
        self.scope_node_ids[scope_id]
    }

    /// Iterate over all bindings declared in the entire program.
    ///
    /// If you only want bindings in a specific scope, use [`iter_bindings_in`].
    ///
    /// [`iter_bindings_in`]: Scoping::iter_bindings_in
    pub fn iter_bindings(&self) -> impl Iterator<Item = (ScopeId, &Bindings)> + '_ {
        self.cell.borrow_dependent().bindings.iter_enumerated()
    }

    /// Iterate over bindings declared inside a scope.
    #[inline]
    pub fn iter_bindings_in(&self, scope_id: ScopeId) -> impl Iterator<Item = SymbolId> + '_ {
        self.cell.borrow_dependent().bindings[scope_id].values().copied()
    }

    #[inline]
    pub(crate) fn insert_binding(&mut self, scope_id: ScopeId, name: &str, symbol_id: SymbolId) {
        self.cell.with_dependent_mut(|allocator, cell| {
            let name = allocator.alloc_str(name);
            cell.bindings[scope_id].insert(name, symbol_id);
        });
    }

    /// Return whether this `ScopeTree` has child IDs recorded
    #[inline]
    pub fn has_scope_child_ids(&self) -> bool {
        self.scope_build_child_ids
    }

    /// Get the child scopes of a scope
    #[inline]
    pub fn get_scope_child_ids(&self, scope_id: ScopeId) -> &[ScopeId] {
        &self.cell.borrow_dependent().scope_child_ids[scope_id.index()]
    }

    pub fn iter_all_scope_child_ids(
        &self,
        scope_id: ScopeId,
    ) -> impl Iterator<Item = ScopeId> + '_ {
        let mut stack = self.cell.borrow_dependent().scope_child_ids[scope_id.index()]
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let child_ids = &self.cell.borrow_dependent().scope_child_ids;
        std::iter::from_fn(move || {
            if let Some(scope_id) = stack.pop() {
                if let Some(children) = child_ids.get(scope_id.index()) {
                    stack.extend(children.iter().copied());
                }
                Some(scope_id)
            } else {
                None
            }
        })
    }

    pub fn remove_child_scopes(&mut self, scope_id: ScopeId, child_scope_ids: &[ScopeId]) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            cell.scope_child_ids[scope_id.index()]
                .retain(|scope_id| !child_scope_ids.contains(scope_id));
        });
    }

    /// Create a scope.
    #[inline]
    pub fn add_scope(
        &mut self,
        parent_id: Option<ScopeId>,
        node_id: NodeId,
        flags: ScopeFlags,
    ) -> ScopeId {
        let scope_id = self.scope_parent_ids.push(parent_id);
        self.scope_flags.push(flags);
        self.cell.with_dependent_mut(|allocator, cell| {
            cell.bindings.push(Bindings::new_in(allocator));
        });
        self.scope_node_ids.push(node_id);
        if self.scope_build_child_ids {
            self.cell.with_dependent_mut(|allocator, cell| {
                cell.scope_child_ids.push(ArenaVec::new_in(allocator));
                if let Some(parent_id) = parent_id {
                    cell.scope_child_ids[parent_id.index()].push(scope_id);
                }
            });
        }
        scope_id
    }

    /// Add a binding to a scope.
    ///
    /// [`binding`]: Bindings
    pub fn add_binding(&mut self, scope_id: ScopeId, name: &str, symbol_id: SymbolId) {
        self.cell.with_dependent_mut(|allocator, cell| {
            let name = allocator.alloc_str(name);
            cell.bindings[scope_id].insert(name, symbol_id);
        });
    }

    /// Remove an existing binding from a scope.
    pub fn remove_binding(&mut self, scope_id: ScopeId, name: &str) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            cell.bindings[scope_id].remove(name);
        });
    }

    /// Move a binding from one scope to another.
    pub fn move_binding(&mut self, from: ScopeId, to: ScopeId, name: &str) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            let from_map = &mut cell.bindings[from];
            if let Some((name, symbol_id)) = from_map.remove_entry(name) {
                cell.bindings[to].insert(name, symbol_id);
            }
        });
    }

    /// Rename a binding to a new name.
    ///
    /// The following must be true for successful operation:
    /// * Binding exists in specified scope for `old_name`.
    /// * Existing binding is for specified `symbol_id`.
    /// * No binding already exists for `new_name`.
    ///
    /// Panics in debug mode if any of the above are not true.
    pub fn rename_binding(
        &mut self,
        scope_id: ScopeId,
        symbol_id: SymbolId,
        old_name: &str,
        new_name: &str,
    ) {
        self.cell.with_dependent_mut(|allocator, cell| {
            let bindings = &mut cell.bindings[scope_id];
            let old_symbol_id = bindings.remove(old_name);
            debug_assert_eq!(old_symbol_id, Some(symbol_id));
            let new_name = allocator.alloc_str(new_name);
            let existing_symbol_id = bindings.insert(new_name, symbol_id);
            debug_assert!(existing_symbol_id.is_none());
        });
    }

    pub fn delete_typescript_bindings(&mut self) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            for bindings in &mut cell.bindings {
                bindings.retain(|_name, symbol_id| {
                    let flags = self.symbol_flags[*symbol_id];
                    !flags.intersects(
                        SymbolFlags::TypeAlias
                            | SymbolFlags::Interface
                            | SymbolFlags::TypeParameter,
                    )
                });
            }
        });
    }

    pub fn function_name_symbols(&self) -> &FxHashSet<SymbolId> {
        &self.function_names
    }

    pub fn class_name_symbols(&self) -> &FxHashSet<SymbolId> {
        &self.class_names
    }

    pub(crate) fn set_name_symbols(
        &mut self,
        function_symbols: FxHashSet<SymbolId>,
        function_references: FxHashSet<ReferenceId>,
        class_symbols: FxHashSet<SymbolId>,
        class_references: FxHashSet<ReferenceId>,
    ) {
        self.function_names = function_symbols
            .into_iter()
            .chain(
                function_references
                    .into_iter()
                    .filter_map(|ref_id| self.references[ref_id].symbol_id()),
            )
            .collect();
        self.class_names = class_symbols
            .into_iter()
            .chain(
                class_references
                    .into_iter()
                    .filter_map(|ref_id| self.references[ref_id].symbol_id()),
            )
            .collect();
    }
}
