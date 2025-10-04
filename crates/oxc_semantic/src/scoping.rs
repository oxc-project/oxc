use std::{collections::hash_map::Entry, fmt, mem};

use rustc_hash::{FxHashMap, FxHashSet};
use self_cell::self_cell;

use oxc_allocator::{Allocator, CloneIn, FromIn, HashMap as ArenaHashMap, Vec as ArenaVec};
use oxc_index::IndexVec;
use oxc_span::{Atom, Span};
use oxc_syntax::{
    node::NodeId,
    reference::{Reference, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

pub type Bindings<'a> = ArenaHashMap<'a, &'a str, SymbolId>;
pub type UnresolvedReferences<'a> = ArenaHashMap<'a, &'a str, ArenaVec<'a, ReferenceId>>;

#[derive(Clone, Debug)]
pub struct Redeclaration {
    pub span: Span,
    pub declaration: NodeId,
    pub flags: SymbolFlags,
}

impl CloneIn<'_> for Redeclaration {
    type Cloned = Self;

    #[inline]
    fn clone_in(&self, _allocator: &Allocator) -> Self::Cloned {
        Self { span: self.span, declaration: NodeId::DUMMY, flags: self.flags }
    }

    #[inline]
    fn clone_in_with_semantic_ids(&self, _allocator: &Allocator) -> Self::Cloned {
        self.clone()
    }
}

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

    pub(crate) references: IndexVec<ReferenceId, Reference>,

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
            references: IndexVec::new(),
            no_side_effects: FxHashSet::default(),
            scope_parent_ids: IndexVec::new(),
            scope_build_child_ids: false,
            scope_node_ids: IndexVec::new(),
            scope_flags: IndexVec::new(),
            cell: ScopingCell::new(Allocator::default(), |allocator| ScopingInner {
                symbol_names: ArenaVec::new_in(allocator),
                resolved_references: ArenaVec::new_in(allocator),
                symbol_redeclarations: FxHashMap::default(),
                bindings: IndexVec::new(),
                scope_child_ids: ArenaVec::new_in(allocator),
                root_unresolved_references: UnresolvedReferences::new_in(allocator),
            }),
        }
    }
}

/// [`ScopingCell`] contains parts of [`Scoping`] which are 2-dimensional structures
/// e.g. `Vec<Vec<T>>`, `HashMap<Vec<T>>`, `Vec<HashMap<T>>`.
///
/// These structures are very expensive to construct and drop, due to the large number of allocations /
/// deallocations involved. Therefore, we store them in an arena allocator to:
/// 1. Avoid costly heap allocations.
/// 2. Be able to drop all the data cheaply in one go.
///
/// We use [`self_cell!`] to be able to store the `Allocator` alongside `Vec`s and `HashMap`s which store
/// their data in that `Allocator` (self-referential struct).
///
/// Conceptually, these structures own their data, and so `ScopingCell` (and therefore also `Scoping`)
/// should be `Send` and `Sync`, exactly as it would be if `ScopingCell` contained standard heap-allocated
/// `Vec`s and `HashMap`s.
///
/// However, the use of an arena allocator complicates matters, because `Allocator` is not `Sync`,
/// and `oxc_allocator::Vec` is not `Send`.
///
/// ### `Sync`
///
/// For it to be safe for `&ScopingCell` to be sent across threads, we must make it impossible to obtain
/// multiple `&Allocator` references from them on different threads, because those references could be
/// used to allocate into the same arena simultaneously. `Allocator` is not thread-safe, and this would
/// likely be undefined behavior.
///
/// We prevent this by wrapping the struct created by `self_cell!` in a further wrapper.
/// That outer wrapper prevents access to `with_dependent` and `borrow_owner` methods of `ScopingCellInner`,
/// which allow obtaining `&Allocator` from a `&ScopingCell`.
///
/// The only method which *does* allow access to `&Allocator` is `with_dependent_mut`.
/// It takes `&mut self`, which guarantees exclusive access to `ScopingCell`. Therefore, no other code
/// (on any thread) can simultaneously have access to the `Allocator` during a call to `with_dependent_mut`.
///
/// `allocator_used_bytes` obtains an `&Allocator` reference internally, without taking `&mut self`.
/// But it doesn't mutate the `Allocator` in any way, and it doesn't expose the `&Allocator` to user.
/// By taking `&self`, it guarantees that `with_dependent_mut` cannot be called at the same time.
///
/// ### `Send`
///
/// `Allocator` is `Send`. `oxc_allocator::Vec` is not, but that restriction is purely to prevent a `Vec`
/// being moved to different thread from the `Allocator`, which would allow multiple threads making
/// allocations in that arena simultaneously.
///
/// Here, the `Allocator` and the `Vec`s are contained in the same struct, and moving them to another
/// thread *together* does not cause a problem.
///
/// This is all enclosed in a module, to prevent access to `ScopingCellInner` directly.
mod scoping_cell {
    use super::*;

    // Inner self-referential struct containing `Allocator` and `ScopingInner`,
    // where `ScopingInner` contains `Vec`s and `HashMap`s which store their data in the `Allocator`.
    self_cell!(
        pub struct ScopingCellInner {
            owner: Allocator,
            #[covariant]
            dependent: ScopingInner,
        }
    );

    /// Wrapper around [`ScopingCellInner`], which only provides methods that give access to an
    /// `&Allocator` reference if provided with `&mut ScopingCell`. See comments above.
    #[repr(transparent)]
    pub struct ScopingCell(ScopingCellInner);

    #[expect(clippy::inline_always)] // All methods just delegate
    impl ScopingCell {
        /// Construct a new [`ScopingCell`] with an [`Allocator`] and `dependent_builder` function.
        #[inline(always)]
        pub fn new(
            allocator: Allocator,
            dependent_builder: impl for<'_q> FnOnce(&'_q Allocator) -> ScopingInner<'_q>,
        ) -> Self {
            Self(ScopingCellInner::new(allocator, dependent_builder))
        }

        /// Borrow [`ScopingInner`].
        #[inline(always)]
        pub fn borrow_dependent(&self) -> &ScopingInner<'_> {
            self.0.borrow_dependent()
        }

        /// Call given closure `func` with an unique reference to [`ScopingInner`].
        #[inline(always)]
        pub fn with_dependent_mut<'outer_fn, Ret>(
            &'outer_fn mut self,
            func: impl for<'_q> FnOnce(&'_q Allocator, &'outer_fn mut ScopingInner<'_q>) -> Ret,
        ) -> Ret {
            self.0.with_dependent_mut(func)
        }

        /// Calculate the total size of data used in the [`Allocator`], in bytes.
        ///
        /// See [`Allocator::used_bytes`] for more info.
        #[expect(clippy::unnecessary_safety_comment)]
        #[inline(always)]
        pub fn allocator_used_bytes(&self) -> usize {
            // SAFETY:
            // `with_dependent_mut` is the only method which gives access to the `Allocator`, and it
            // takes `&mut self`. This method takes `&self`, which means it can't be called at the same
            // time as `with_dependent_mut` (or within `with_dependent_mut`'s callback closure).
            //
            // Therefore, the only other references to `&Allocator` which can be held at this point
            // are in other calls to this method on other threads.
            // `used_bytes` does not perform allocations, or mutate the `Allocator` in any way.
            // So it's fine if 2 threads are calling this method simultaneously, because they're
            // both performing read-only actions.
            //
            // Another thread could simultaneously hold a reference to `&ScopingInner` via `borrow_dependent`,
            // but the `Vec`s and `HashMap`s in `ScopingInner` don't allow making allocations in the arena
            // without a `&mut` reference (e.g. `Vec::push` takes `&mut self`). Such mutable references
            // cannot be obtained from an immutable `&ScopingInner` reference.
            // So there's no way for simultaneous usage of `borrow_dependent` on another thread to break
            // the guarantee that no mutation of the `Allocator` can occur during this method.
            self.0.borrow_owner().used_bytes()
        }

        /// Consume [`ScopingCell`] and return the [`Allocator`] it contains.
        #[expect(dead_code)]
        #[inline(always)]
        pub fn into_owner(self) -> Allocator {
            self.0.into_owner()
        }
    }

    /// SAFETY: `ScopingCell` can be `Send` because both the `Allocator` and `Vec`s / `HashMap`s
    /// storing their data in that `Allocator` are moved to another thread together.
    unsafe impl Send for ScopingCell {}

    /// SAFETY: `ScopingCell` can be `Sync` if `ScopingInner` is `Sync`, because `ScopingCell` provides
    /// no methods which give access to an `&Allocator` reference, except when taking `&mut self`,
    /// which guarantees exclusive access. See further explanation above.
    unsafe impl<'cell> Sync for ScopingCell where ScopingInner<'cell>: Sync {}
}
use scoping_cell::ScopingCell;

pub struct ScopingInner<'cell> {
    /* Symbol Table Fields */
    symbol_names: ArenaVec<'cell, Atom<'cell>>,
    resolved_references: ArenaVec<'cell, ArenaVec<'cell, ReferenceId>>,
    /// Redeclarations of a symbol.
    ///
    /// NOTE:
    /// Once a symbol is redeclared, there are at least two entries here. The first
    /// entry is the original symbol information, and the rest are redeclarations.
    /// i.e. `symbol_redeclarations[symbol_id].len() >= 2` always.
    symbol_redeclarations: FxHashMap<SymbolId, ArenaVec<'cell, Redeclaration>>,
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
    pub fn set_symbol_name(&mut self, symbol_id: SymbolId, name: &str) {
        self.cell.with_dependent_mut(|allocator, cell| {
            cell.symbol_names[symbol_id.index()] = Atom::from_in(name, allocator);
        });
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
    pub fn symbol_redeclarations(&self, symbol_id: SymbolId) -> &[Redeclaration] {
        self.cell.borrow_dependent().symbol_redeclarations.get(&symbol_id).map_or_else(
            || {
                static EMPTY: &[Redeclaration] = &[];
                EMPTY
            },
            |v| v.as_slice(),
        )
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
        self.cell.with_dependent_mut(|allocator, cell| {
            cell.symbol_names.push(Atom::from_in(name, allocator));
            cell.resolved_references.push(ArenaVec::new_in(allocator));
        });
        self.symbol_spans.push(span);
        self.symbol_flags.push(flags);
        self.symbol_scope_ids.push(scope_id);
        self.symbol_declarations.push(node_id)
    }

    pub fn add_symbol_redeclaration(
        &mut self,
        symbol_id: SymbolId,
        flags: SymbolFlags,
        declaration: NodeId,
        span: Span,
    ) {
        let is_first_redeclared =
            !self.cell.borrow_dependent().symbol_redeclarations.contains_key(&symbol_id);
        // Borrow checker doesn't allow us to call `self.symbol_span` in `with_dependent_mut`,
        // so we need construct `Redeclaration` here.
        let first_declaration = is_first_redeclared.then(|| Redeclaration {
            span: self.symbol_span(symbol_id),
            declaration: self.symbol_declaration(symbol_id),
            flags: self.symbol_flags(symbol_id),
        });

        self.cell.with_dependent_mut(|allocator, cell| {
            let redeclaration = Redeclaration { span, declaration, flags };
            match cell.symbol_redeclarations.entry(symbol_id) {
                Entry::Occupied(occupied) => {
                    occupied.into_mut().push(redeclaration);
                }
                Entry::Vacant(vacant) => {
                    let first_declaration = first_declaration.unwrap_or_else(|| {
                        unreachable!(
                            "The above step has already been checked, and it was first declared."
                        )
                    });
                    let v = ArenaVec::from_array_in([first_declaration, redeclaration], allocator);
                    vacant.insert(v);
                }
            }
        });
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
    pub fn get_resolved_reference_ids(&self, symbol_id: SymbolId) -> &[ReferenceId] {
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

    /// Get whether a symbol is unused (i.e. not read or written after declaration).
    pub fn symbol_is_unused(&self, symbol_id: SymbolId) -> bool {
        self.get_resolved_reference_ids(symbol_id).is_empty()
    }

    /// Add a reference to a symbol.
    pub fn add_resolved_reference(&mut self, symbol_id: SymbolId, reference_id: ReferenceId) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            cell.resolved_references[symbol_id.index()].push(reference_id);
        });
    }

    /// Delete a reference.
    pub fn delete_reference(&mut self, reference_id: ReferenceId) {
        let Some(symbol_id) = self.get_reference(reference_id).symbol_id() else { return };
        self.delete_resolved_reference(symbol_id, reference_id);
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
            cell.symbol_names.reserve_exact(additional_symbols);
            cell.resolved_references.reserve_exact(additional_symbols);
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
                cell.scope_child_ids.reserve_exact(additional_scopes);
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
    #[expect(clippy::unused_self)]
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
    pub fn root_unresolved_references(&self) -> &UnresolvedReferences<'_> {
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
    pub fn get_bindings(&self, scope_id: ScopeId) -> &Bindings<'_> {
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
    pub fn iter_bindings(&self) -> impl Iterator<Item = (ScopeId, &Bindings<'_>)> + '_ {
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

    /// Remove a child scope from a parent scope.
    /// # Panics
    /// Panics if the child scope is not a child of the parent scope.
    pub fn remove_child_scope(&mut self, scope_id: ScopeId, child_scope_id: ScopeId) {
        self.cell.with_dependent_mut(|_allocator, cell| {
            let child_ids = &mut cell.scope_child_ids[scope_id.index()];
            let index = child_ids.iter().position(|&scope_id| scope_id == child_scope_id).unwrap();
            child_ids.swap_remove(index);
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

    /// Rename symbol.
    ///
    /// The following must be true for successful operation:
    /// * Binding exists in specified scope for `symbol_id`.
    /// * No binding already exists in scope for `new_name`.
    ///
    /// Panics in debug mode if either of the above are not satisfied.
    pub fn rename_symbol(&mut self, symbol_id: SymbolId, scope_id: ScopeId, new_name: &str) {
        self.cell.with_dependent_mut(|allocator, cell| {
            // Rename symbol
            let new_name = Atom::from_in(new_name, allocator);
            let old_name = mem::replace(&mut cell.symbol_names[symbol_id.index()], new_name);

            // Rename binding, same as `Self::rename_binding`, we cannot call it directly
            // because the `old_name` borrowed `cell`.
            let bindings = &mut cell.bindings[scope_id];
            let old_symbol_id = bindings.remove(old_name.as_str());
            debug_assert_eq!(old_symbol_id, Some(symbol_id));
            let existing_symbol_id = bindings.insert(new_name.as_str(), symbol_id);
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
}

impl Scoping {
    /// Clone all semantic data. Used in `Rolldown`.
    #[must_use]
    pub fn clone_in_with_semantic_ids_with_another_arena(&self) -> Self {
        let used_bytes = self.cell.allocator_used_bytes();
        let cell = self.cell.borrow_dependent();

        Self {
            symbol_spans: self.symbol_spans.clone(),
            symbol_flags: self.symbol_flags.clone(),
            symbol_scope_ids: self.symbol_scope_ids.clone(),
            symbol_declarations: self.symbol_declarations.clone(),
            references: self.references.clone(),
            no_side_effects: self.no_side_effects.clone(),
            scope_parent_ids: self.scope_parent_ids.clone(),
            scope_build_child_ids: self.scope_build_child_ids,
            scope_node_ids: self.scope_node_ids.clone(),
            scope_flags: self.scope_flags.clone(),
            cell: {
                let allocator = Allocator::with_capacity(used_bytes);
                ScopingCell::new(allocator, |allocator| ScopingInner {
                    symbol_names: cell.symbol_names.clone_in_with_semantic_ids(allocator),
                    resolved_references: cell
                        .resolved_references
                        .clone_in_with_semantic_ids(allocator),
                    symbol_redeclarations: cell
                        .symbol_redeclarations
                        .iter()
                        .map(|(k, v)| (*k, v.clone_in_with_semantic_ids(allocator)))
                        .collect(),
                    bindings: cell
                        .bindings
                        .iter()
                        .map(|map| map.clone_in_with_semantic_ids(allocator))
                        .collect(),
                    scope_child_ids: cell.scope_child_ids.clone_in_with_semantic_ids(allocator),
                    root_unresolved_references: cell
                        .root_unresolved_references
                        .clone_in_with_semantic_ids(allocator),
                })
            },
        }
    }
}
