use std::hash::BuildHasherDefault;

use indexmap::IndexMap;
use oxc_index::IndexVec;
use oxc_span::CompactStr;
use oxc_syntax::reference::{ReferenceFlag, ReferenceId};
pub use oxc_syntax::scope::{ScopeFlags, ScopeId};
use rustc_hash::{FxHashMap, FxHasher};

use crate::{symbol::SymbolId, AstNodeId};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

type Bindings = FxIndexMap<CompactStr, SymbolId>;
pub(crate) type UnresolvedReference = (ReferenceId, ReferenceFlag);
pub(crate) type UnresolvedReferences = FxHashMap<CompactStr, Vec<UnresolvedReference>>;

/// Scope Tree
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
pub struct ScopeTree {
    /// Maps a scope to the parent scope it belongs in.
    parent_ids: IndexVec<ScopeId, Option<ScopeId>>,
    /// Maps a scope to direct children scopes.
    child_ids: IndexVec<ScopeId, Vec<ScopeId>>,
    /// Maps a scope to its node id.
    node_ids: IndexVec<ScopeId, AstNodeId>,
    flags: IndexVec<ScopeId, ScopeFlags>,
    bindings: IndexVec<ScopeId, Bindings>,
    pub(crate) root_unresolved_references: UnresolvedReferences,
}

impl ScopeTree {
    pub fn len(&self) -> usize {
        self.parent_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn ancestors(&self, scope_id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        std::iter::successors(Some(scope_id), |scope_id| self.parent_ids[*scope_id])
    }

    pub fn descendants(&self, scope_id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        // Has to be a `fn` and pass arguments because we can't
        // have recursive closures
        fn add_to_list(
            parent_id: ScopeId,
            child_ids: &IndexVec<ScopeId, Vec<ScopeId>>,
            items: &mut Vec<ScopeId>,
        ) {
            if let Some(children) = child_ids.get(parent_id) {
                for child_id in children {
                    items.push(*child_id);
                    add_to_list(*child_id, child_ids, items);
                }
            }
        }

        let mut list = vec![];

        add_to_list(scope_id, &self.child_ids, &mut list);

        list.into_iter()
    }

    pub fn get_child_ids(&self, scope_id: ScopeId) -> Option<&Vec<ScopeId>> {
        self.child_ids.get(scope_id)
    }

    pub fn get_child_ids_mut(&mut self, scope_id: ScopeId) -> Option<&mut Vec<ScopeId>> {
        self.child_ids.get_mut(scope_id)
    }

    pub fn descendants_from_root(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.parent_ids.iter_enumerated().map(|(scope_id, _)| scope_id)
    }

    #[inline]
    pub fn root_scope_id(&self) -> ScopeId {
        ScopeId::new(0)
    }

    pub fn root_flags(&self) -> ScopeFlags {
        self.flags[self.root_scope_id()]
    }

    pub fn root_unresolved_references(&self) -> &UnresolvedReferences {
        &self.root_unresolved_references
    }

    pub fn root_unresolved_references_ids(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = ReferenceId> + '_> + '_ {
        self.root_unresolved_references.values().map(|v| v.iter().map(|(id, _)| *id))
    }

    pub fn get_flags(&self, scope_id: ScopeId) -> ScopeFlags {
        self.flags[scope_id]
    }

    pub fn get_flags_mut(&mut self, scope_id: ScopeId) -> &mut ScopeFlags {
        &mut self.flags[scope_id]
    }

    pub fn get_new_scope_flags(&self, flags: ScopeFlags, parent_scope_id: ScopeId) -> ScopeFlags {
        let mut strict_mode = self.root_flags().is_strict_mode();
        let parent_scope_flags = self.get_flags(parent_scope_id);

        // Inherit strict mode for functions
        // https://tc39.es/ecma262/#sec-strict-mode-code
        if !strict_mode
            && (parent_scope_flags.is_function() || parent_scope_flags.is_ts_module_block())
            && parent_scope_flags.is_strict_mode()
        {
            strict_mode = true;
        }

        // inherit flags for non-function scopes
        let mut flags = flags;
        if !flags.contains(ScopeFlags::Function) {
            flags |= parent_scope_flags & ScopeFlags::Modifiers;
        };

        if strict_mode {
            flags |= ScopeFlags::StrictMode;
        }

        flags
    }

    pub fn get_parent_id(&self, scope_id: ScopeId) -> Option<ScopeId> {
        self.parent_ids[scope_id]
    }

    pub fn set_parent_id(&mut self, scope_id: ScopeId, parent_id: Option<ScopeId>) {
        self.parent_ids[scope_id] = parent_id;
        if let Some(parent_id) = parent_id {
            self.child_ids[parent_id].push(scope_id);
        }
    }

    /// Get a variable binding by name that was declared in the top-level scope
    pub fn get_root_binding(&self, name: &str) -> Option<SymbolId> {
        self.get_binding(self.root_scope_id(), name)
    }

    pub fn add_root_unresolved_reference(
        &mut self,
        name: CompactStr,
        reference: UnresolvedReference,
    ) {
        self.root_unresolved_references.entry(name).or_default().push(reference);
    }

    pub fn has_binding(&self, scope_id: ScopeId, name: &str) -> bool {
        self.bindings[scope_id].get(name).is_some()
    }

    pub fn get_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        self.bindings[scope_id].get(name).copied()
    }

    pub fn find_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        for scope_id in self.ancestors(scope_id) {
            if let Some(symbol_id) = self.bindings[scope_id].get(name) {
                return Some(*symbol_id);
            }
        }
        None
    }

    pub fn get_bindings(&self, scope_id: ScopeId) -> &Bindings {
        &self.bindings[scope_id]
    }

    pub fn get_node_id(&self, scope_id: ScopeId) -> AstNodeId {
        self.node_ids[scope_id]
    }

    pub fn iter_bindings(&self) -> impl Iterator<Item = (ScopeId, SymbolId, &'_ CompactStr)> + '_ {
        self.bindings.iter_enumerated().flat_map(|(scope_id, bindings)| {
            bindings.iter().map(move |(name, symbol_id)| (scope_id, *symbol_id, name))
        })
    }

    pub(crate) fn get_bindings_mut(&mut self, scope_id: ScopeId) -> &mut Bindings {
        &mut self.bindings[scope_id]
    }

    /// Create scope.
    /// For root (`Program`) scope, use `add_root_scope`.
    pub fn add_scope(
        &mut self,
        parent_id: ScopeId,
        node_id: AstNodeId,
        flags: ScopeFlags,
    ) -> ScopeId {
        let scope_id = self.add_scope_impl(Some(parent_id), node_id, flags);

        // Set this scope as child of parent scope
        self.child_ids[parent_id].push(scope_id);

        scope_id
    }

    /// Create root (`Program`) scope.
    pub fn add_root_scope(&mut self, node_id: AstNodeId, flags: ScopeFlags) -> ScopeId {
        self.add_scope_impl(None, node_id, flags)
    }

    // `#[inline]` because almost always called from `add_scope` and want to avoid
    // overhead of a function call there.
    #[inline]
    fn add_scope_impl(
        &mut self,
        parent_id: Option<ScopeId>,
        node_id: AstNodeId,
        flags: ScopeFlags,
    ) -> ScopeId {
        let scope_id = self.parent_ids.push(parent_id);
        self.child_ids.push(vec![]);
        self.flags.push(flags);
        self.bindings.push(Bindings::default());
        self.node_ids.push(node_id);
        scope_id
    }

    pub fn add_binding(&mut self, scope_id: ScopeId, name: CompactStr, symbol_id: SymbolId) {
        self.bindings[scope_id].insert(name, symbol_id);
    }

    pub fn remove_binding(&mut self, scope_id: ScopeId, name: &CompactStr) {
        self.bindings[scope_id].shift_remove(name);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.parent_ids.reserve(additional);
        self.child_ids.reserve(additional);
        self.flags.reserve(additional);
        self.bindings.reserve(additional);
        self.node_ids.reserve(additional);
    }
}
