use std::{fmt, mem};

use oxc_allocator::{Allocator, HashMap as ArenaHashMap, Vec as ArenaVec};
use oxc_index::{Idx, IndexVec};
use oxc_syntax::{
    node::NodeId,
    reference::ReferenceId,
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use crate::SymbolTable;

pub(crate) type Bindings<'a> = ArenaHashMap<'a, &'a str, SymbolId>;
pub type UnresolvedReferences<'a> = ArenaHashMap<'a, &'a str, ArenaVec<'a, ReferenceId>>;

/// Scope Tree
///
/// The scope tree stores lexical scopes created by a program, and all the
/// variable bindings each scope creates.
///
/// - All scopes have a parent scope, except the root scope.
/// - Scopes can have 0 or more child scopes.
/// - Nodes that create a scope store the [`ScopeId`] of the scope they create.
///
/// `SoA` (Struct of Arrays) for memory efficiency.
pub struct ScopeTree {
    /// Maps a scope to the parent scope it belongs in.
    parent_ids: IndexVec<ScopeId, Option<ScopeId>>,

    /// Runtime flag for constructing child_ids.
    pub(crate) build_child_ids: bool,

    /// Maps a scope to its node id.
    node_ids: IndexVec<ScopeId, NodeId>,

    flags: IndexVec<ScopeId, ScopeFlags>,

    pub(crate) cell: ScopeTreeCell,
}

impl fmt::Debug for ScopeTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("ScopeTree").finish()
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self {
            parent_ids: IndexVec::new(),
            build_child_ids: false,
            node_ids: IndexVec::new(),
            flags: IndexVec::new(),
            cell: ScopeTreeCell::new(Allocator::default(), |allocator| ScopeTreeInner {
                bindings: IndexVec::new(),
                child_ids: ArenaVec::new_in(allocator),
                root_unresolved_references: UnresolvedReferences::new_in(allocator),
            }),
        }
    }
}

self_cell::self_cell!(
    pub(crate) struct ScopeTreeCell {
        owner: Allocator,
        #[covariant]
        dependent: ScopeTreeInner,
    }
);

pub(crate) struct ScopeTreeInner<'cell> {
    /// Symbol bindings in a scope.
    ///
    /// A binding is a mapping from an identifier name to its [`SymbolId`]
    pub(crate) bindings: IndexVec<ScopeId, Bindings<'cell>>,

    /// Maps a scope to direct children scopes.
    child_ids: ArenaVec<'cell, ArenaVec<'cell, ScopeId>>,

    pub(crate) root_unresolved_references: UnresolvedReferences<'cell>,
}

impl ScopeTree {
    const ROOT_SCOPE_ID: ScopeId = ScopeId::new(0);

    /// Returns the number of scopes found in the program. Includes the root
    /// program scope.
    #[inline]
    pub fn len(&self) -> usize {
        self.parent_ids.len()
    }

    /// Returns `true` if there are no scopes in the program.
    ///
    /// This will always return `false` when semantic analysis has completed
    /// since there is a root scope.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.parent_ids.is_empty()
    }

    /// Iterate over the scopes that contain a scope.
    ///
    /// The first element of this iterator will be the scope itself. This
    /// guarantees the iterator will have at least 1 element.
    pub fn ancestors(&self, scope_id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        std::iter::successors(Some(scope_id), |&scope_id| self.parent_ids[scope_id])
    }

    pub fn descendants_from_root(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.parent_ids.iter_enumerated().map(|(scope_id, _)| scope_id)
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
    pub fn root_flags(&self) -> ScopeFlags {
        self.flags[self.root_scope_id()]
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
        self.cell.with_dependent_mut(|allocator, inner| {
            for (k, v) in entries {
                let k = allocator.alloc_str(k);
                let v = ArenaVec::from_iter_in(v, allocator);
                inner.root_unresolved_references.insert(k, v);
            }
            // =
            // .extend_from(entries.map(|(k, v)| (allocator.alloc(k),)))
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
        self.cell.with_dependent_mut(|_allocator, inner| {
            let reference_ids = inner.root_unresolved_references.get_mut(name).unwrap();
            if reference_ids.len() == 1 {
                assert_eq!(reference_ids[0], reference_id);
                inner.root_unresolved_references.remove(name);
            } else {
                let index = reference_ids.iter().position(|&id| id == reference_id).unwrap();
                reference_ids.swap_remove(index);
            }
        });
    }

    #[inline]
    pub fn get_flags(&self, scope_id: ScopeId) -> ScopeFlags {
        self.flags[scope_id]
    }

    #[inline]
    pub fn get_flags_mut(&mut self, scope_id: ScopeId) -> &mut ScopeFlags {
        &mut self.flags[scope_id]
    }

    /// Get [`ScopeFlags`] for a new child scope under `parent_scope_id`.
    pub fn get_new_scope_flags(&self, flags: ScopeFlags, parent_scope_id: ScopeId) -> ScopeFlags {
        // https://tc39.es/ecma262/#sec-strict-mode-code
        flags | self.get_flags(parent_scope_id) & ScopeFlags::StrictMode
    }

    #[inline]
    pub fn get_parent_id(&self, scope_id: ScopeId) -> Option<ScopeId> {
        self.parent_ids[scope_id]
    }

    pub fn set_parent_id(&mut self, scope_id: ScopeId, parent_id: Option<ScopeId>) {
        self.parent_ids[scope_id] = parent_id;
        if self.build_child_ids {
            // Set this scope as child of parent scope
            if let Some(parent_id) = parent_id {
                self.cell.with_dependent_mut(|_allocator, inner| {
                    inner.child_ids[parent_id.index()].push(scope_id);
                });
            }
        }
    }

    /// Change the parent scope of a scope.
    ///
    /// This will also remove the scope from the child list of the old parent and add it to the new parent.
    pub fn change_parent_id(&mut self, scope_id: ScopeId, new_parent_id: Option<ScopeId>) {
        let old_parent_id = mem::replace(&mut self.parent_ids[scope_id], new_parent_id);
        if self.build_child_ids {
            self.cell.with_dependent_mut(|_allocator, inner| {
                // Remove this scope from old parent scope
                if let Some(old_parent_id) = old_parent_id {
                    inner.child_ids[old_parent_id.index()].retain(|&child_id| child_id != scope_id);
                }
                // And add it to new parent scope
                if let Some(parent_id) = new_parent_id {
                    inner.child_ids[parent_id.index()].push(scope_id);
                }
            });
        }
    }

    /// Delete a scope.
    pub fn delete_scope(&mut self, scope_id: ScopeId) {
        if self.build_child_ids {
            self.cell.with_dependent_mut(|_allocator, inner| {
                inner.child_ids[scope_id.index()].clear();
                let parent_id = self.parent_ids[scope_id];
                if let Some(parent_id) = parent_id {
                    inner.child_ids[parent_id.index()].retain(|&child_id| child_id != scope_id);
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
        self.cell.with_dependent_mut(|allocator, inner| {
            let name = allocator.alloc_str(name);
            inner
                .root_unresolved_references
                .entry(name)
                .or_insert_with(|| ArenaVec::new_in(allocator))
                .push(reference_id);
        });
    }

    /// Check if a symbol is declared in a certain scope.
    pub fn has_binding(&self, scope_id: ScopeId, name: &str) -> bool {
        self.cell.borrow_dependent().bindings[scope_id].contains_key(name)
    }

    /// Get the symbol bound to an identifier name in a scope.
    ///
    /// Returns [`None`] if that name is not bound in the scope. This could be
    /// because the symbol is not declared within this tree, but it could also
    /// be because its declaration is in a parent scope. If you want to find a
    /// binding that might be declared in a parent scope, use [`find_binding`].
    ///
    /// [`find_binding`]: ScopeTree::find_binding
    pub fn get_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        self.cell.borrow_dependent().bindings[scope_id].get(name).copied()
    }

    /// Find a binding by name in a scope or its ancestors.
    ///
    /// Bindings are resolved by walking up the scope tree until a binding is
    /// found. If no binding is found, [`None`] is returned.
    pub fn find_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        for scope_id in self.ancestors(scope_id) {
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
        self.node_ids[scope_id]
    }

    /// Iterate over all bindings declared in the entire program.
    ///
    /// If you only want bindings in a specific scope, use [`iter_bindings_in`].
    ///
    /// [`iter_bindings_in`]: ScopeTree::iter_bindings_in
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
        self.cell.with_dependent_mut(|allocator, inner| {
            let name = allocator.alloc_str(name);
            inner.bindings[scope_id].insert(name, symbol_id);
        });
    }

    /// Return whether this `ScopeTree` has child IDs recorded
    #[inline]
    pub fn has_child_ids(&self) -> bool {
        self.build_child_ids
    }

    /// Get the child scopes of a scope
    #[inline]
    pub fn get_child_ids(&self, scope_id: ScopeId) -> &[ScopeId] {
        &self.cell.borrow_dependent().child_ids[scope_id.index()]
    }

    pub fn iter_all_child_ids(&self, scope_id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        let mut stack = self.cell.borrow_dependent().child_ids[scope_id.index()]
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let child_ids = &self.cell.borrow_dependent().child_ids;
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
        self.cell.with_dependent_mut(|_allocator, inner| {
            inner.child_ids[scope_id.index()]
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
        let scope_id = self.parent_ids.push(parent_id);
        self.flags.push(flags);
        self.cell.with_dependent_mut(|allocator, inner| {
            inner.bindings.push(Bindings::new_in(allocator));
        });
        self.node_ids.push(node_id);
        if self.build_child_ids {
            self.cell.with_dependent_mut(|allocator, inner| {
                inner.child_ids.push(ArenaVec::new_in(allocator));
                if let Some(parent_id) = parent_id {
                    inner.child_ids[parent_id.index()].push(scope_id);
                }
            });
        }
        scope_id
    }

    /// Add a binding to a scope.
    ///
    /// [`binding`]: Bindings
    pub fn add_binding(&mut self, scope_id: ScopeId, name: &str, symbol_id: SymbolId) {
        self.cell.with_dependent_mut(|allocator, inner| {
            let name = allocator.alloc_str(name);
            inner.bindings[scope_id].insert(name, symbol_id);
        });
    }

    /// Remove an existing binding from a scope.
    pub fn remove_binding(&mut self, scope_id: ScopeId, name: &str) {
        self.cell.with_dependent_mut(|_allocator, inner| {
            inner.bindings[scope_id].remove(name);
        });
    }

    /// Move a binding from one scope to another.
    pub fn move_binding(&mut self, from: ScopeId, to: ScopeId, name: &str) {
        self.cell.with_dependent_mut(|_allocator, inner| {
            let from_map = &mut inner.bindings[from];
            if let Some((name, symbol_id)) = from_map.remove_entry(name) {
                inner.bindings[to].insert(name, symbol_id);
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
        self.cell.with_dependent_mut(|allocator, inner| {
            let bindings = &mut inner.bindings[scope_id];
            let old_symbol_id = bindings.remove(old_name);
            debug_assert_eq!(old_symbol_id, Some(symbol_id));
            let new_name = allocator.alloc_str(new_name);
            let existing_symbol_id = bindings.insert(new_name, symbol_id);
            debug_assert!(existing_symbol_id.is_none());
        });
    }

    /// Reserve memory for an `additional` number of scopes.
    pub fn reserve(&mut self, additional: usize) {
        self.parent_ids.reserve(additional);
        self.flags.reserve(additional);
        self.cell.with_dependent_mut(|_allocator, inner| {
            inner.bindings.reserve(additional);
        });
        self.node_ids.reserve(additional);
        if self.build_child_ids {
            self.cell.with_dependent_mut(|_allocator, inner| {
                inner.child_ids.reserve(additional);
            });
        }
    }

    pub fn delete_typescript_bindings(&mut self, symbol_table: &SymbolTable) {
        self.cell.with_dependent_mut(|_allocator, inner| {
            for bindings in &mut inner.bindings {
                bindings.retain(|_name, symbol_id| {
                    let flags = symbol_table.get_flags(*symbol_id);
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
