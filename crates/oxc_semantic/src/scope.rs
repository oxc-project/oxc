use std::hash::BuildHasherDefault;

use indexmap::IndexMap;

use oxc_index::IndexVec;
use oxc_span::CompactStr;
pub use oxc_syntax::scope::{ScopeFlags, ScopeId};
use rustc_hash::{FxHashMap, FxHasher};

use crate::{reference::ReferenceId, symbol::SymbolId, AstNodeId};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

type Bindings = FxIndexMap<CompactStr, SymbolId>;
type UnresolvedReferences = FxHashMap<CompactStr, Vec<ReferenceId>>;

/// Scope Tree
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
pub struct ScopeTree {
    /// Maps a scope to the parent scope it belongs in
    parent_ids: IndexVec<ScopeId, Option<ScopeId>>,

    /// Maps a scope to direct children scopes
    child_ids: FxHashMap<ScopeId, Vec<ScopeId>>,
    // Maps a scope to its node id
    node_ids: FxHashMap<ScopeId, AstNodeId>,
    flags: IndexVec<ScopeId, ScopeFlags>,
    bindings: IndexVec<ScopeId, Bindings>,
    unresolved_references: IndexVec<ScopeId, UnresolvedReferences>,
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
            child_ids: &FxHashMap<ScopeId, Vec<ScopeId>>,
            items: &mut Vec<ScopeId>,
        ) {
            if let Some(children) = child_ids.get(&parent_id) {
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
        self.child_ids.get(&scope_id)
    }

    pub fn descendants_from_root(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.parent_ids.iter_enumerated().map(|(scope_id, _)| scope_id)
    }

    pub fn root_scope_id(&self) -> ScopeId {
        ScopeId::new(0)
    }

    pub fn root_flags(&self) -> ScopeFlags {
        self.flags[self.root_scope_id()]
    }

    pub fn root_unresolved_references(&self) -> &UnresolvedReferences {
        &self.unresolved_references[self.root_scope_id()]
    }

    pub fn get_flags(&self, scope_id: ScopeId) -> ScopeFlags {
        self.flags[scope_id]
    }

    pub fn get_flags_mut(&mut self, scope_id: ScopeId) -> &mut ScopeFlags {
        &mut self.flags[scope_id]
    }

    pub fn get_parent_id(&self, scope_id: ScopeId) -> Option<ScopeId> {
        self.parent_ids[scope_id]
    }

    /// Get a variable binding by name that was declared in the top-level scope
    pub fn get_root_binding(&self, name: &str) -> Option<SymbolId> {
        self.get_binding(self.root_scope_id(), name)
    }

    pub fn add_root_unresolved_reference(&mut self, name: CompactStr, reference_id: ReferenceId) {
        self.add_unresolved_reference(self.root_scope_id(), name, reference_id);
    }

    pub fn has_binding(&self, scope_id: ScopeId, name: &str) -> bool {
        self.bindings[scope_id].get(name).is_some()
    }

    pub fn get_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        self.bindings[scope_id].get(name).copied()
    }

    pub fn find_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        // TODO: Calculate hash of `name` only once rather than repeatedly on each turn of the loop
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
        self.node_ids[&scope_id]
    }

    pub fn iter_bindings(&self) -> impl Iterator<Item = (ScopeId, SymbolId, &'_ CompactStr)> + '_ {
        self.bindings.iter_enumerated().flat_map(|(scope_id, bindings)| {
            bindings.iter().map(move |(name, symbol_id)| (scope_id, *symbol_id, name))
        })
    }

    pub(crate) fn get_bindings_mut(&mut self, scope_id: ScopeId) -> &mut Bindings {
        &mut self.bindings[scope_id]
    }

    pub(crate) fn add_scope(&mut self, parent_id: Option<ScopeId>, flags: ScopeFlags) -> ScopeId {
        let scope_id = self.parent_ids.push(parent_id);
        _ = self.flags.push(flags);
        _ = self.bindings.push(Bindings::default());
        _ = self.unresolved_references.push(UnresolvedReferences::default());

        if let Some(parent_id) = parent_id {
            self.child_ids.entry(parent_id).or_default().push(scope_id);
        }

        scope_id
    }

    pub(crate) fn add_node_id(&mut self, scope_id: ScopeId, node_id: AstNodeId) {
        self.node_ids.insert(scope_id, node_id);
    }

    pub fn add_binding(&mut self, scope_id: ScopeId, name: CompactStr, symbol_id: SymbolId) {
        self.bindings[scope_id].insert(name, symbol_id);
    }

    pub fn remove_binding(&mut self, scope_id: ScopeId, name: &CompactStr) {
        self.bindings[scope_id].shift_remove(name);
    }

    pub(crate) fn add_unresolved_reference(
        &mut self,
        scope_id: ScopeId,
        name: CompactStr,
        reference_id: ReferenceId,
    ) {
        self.unresolved_references[scope_id].entry(name).or_default().push(reference_id);
    }

    pub(crate) fn extend_unresolved_reference(
        &mut self,
        scope_id: ScopeId,
        name: CompactStr,
        reference_ids: Vec<ReferenceId>,
    ) {
        self.unresolved_references[scope_id].entry(name).or_default().extend(reference_ids);
    }

    pub(crate) fn unresolved_references_mut(
        &mut self,
        scope_id: ScopeId,
    ) -> &mut UnresolvedReferences {
        &mut self.unresolved_references[scope_id]
    }
}
