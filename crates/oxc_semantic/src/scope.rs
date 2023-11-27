use std::hash::BuildHasherDefault;

use indexmap::IndexMap;
use oxc_ast::{ast::Expression, syntax_directed_operations::GatherNodeParts};
use oxc_index::IndexVec;
use oxc_span::Atom;
pub use oxc_syntax::scope::{ScopeFlags, ScopeId};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use crate::{reference::ReferenceId, symbol::SymbolId};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

type Bindings = FxIndexMap<Atom, SymbolId>;
type UnresolvedReferences = FxHashMap<Atom, Vec<ReferenceId>>;

/// Scope Tree
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
pub struct ScopeTree {
    parent_ids: IndexVec<ScopeId, Option<ScopeId>>,
    flags: IndexVec<ScopeId, ScopeFlags>,
    bindings: IndexVec<ScopeId, Bindings>,
    unresolved_references: IndexVec<ScopeId, UnresolvedReferences>,
    /// used for global de conflicting, used for generate a global unique identifier
    references: FxHashSet<Atom>,
    /// only enable in transformer, could remove overhead in oxc-linter
    pub(crate) global_deconflicting: bool,
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

    pub fn descendants(&self) -> impl Iterator<Item = ScopeId> + '_ {
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
    pub fn get_root_binding(&self, name: &Atom) -> Option<SymbolId> {
        self.get_binding(self.root_scope_id(), name)
    }

    pub fn has_binding(&self, scope_id: ScopeId, name: &Atom) -> bool {
        self.bindings[scope_id].get(name).is_some()
    }

    pub fn get_binding(&self, scope_id: ScopeId, name: &Atom) -> Option<SymbolId> {
        self.bindings[scope_id].get(name).copied()
    }

    pub fn get_bindings(&self, scope_id: ScopeId) -> &Bindings {
        &self.bindings[scope_id]
    }

    pub fn iter_bindings(&self) -> impl Iterator<Item = (ScopeId, SymbolId, Atom)> + '_ {
        self.bindings.iter_enumerated().flat_map(|(scope_id, bindings)| {
            bindings.iter().map(move |(name, symbol_id)| (scope_id, *symbol_id, name.clone()))
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
        scope_id
    }

    pub fn add_binding(&mut self, scope_id: ScopeId, name: Atom, symbol_id: SymbolId) {
        if self.global_deconflicting {
            self.references.insert(name.clone());
        }
        self.bindings[scope_id].insert(name, symbol_id);
    }

    pub(crate) fn add_unresolved_reference(
        &mut self,
        scope_id: ScopeId,
        name: Atom,
        reference_id: ReferenceId,
    ) {
        self.unresolved_references[scope_id].entry(name).or_default().push(reference_id);
    }

    pub(crate) fn extend_unresolved_reference(
        &mut self,
        scope_id: ScopeId,
        name: Atom,
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

    // TODO:
    // <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L543>
    pub fn generate_uid_based_on_node(&self, expr: &Expression) -> Atom {
        let mut parts = std::vec::Vec::with_capacity(1);
        expr.gather(&mut |part| parts.push(part));
        let name = parts.join("$");
        let name = name.trim_start_matches('_');
        for i in 0.. {
            let name = Self::generate_uid(name, i);
            if !self.has_binding(ScopeId::new(0), &name) && !self.references.contains(&name) {
                return name;
            }
        }
        unreachable!()
    }

    fn generate_uid(name: &str, i: i32) -> Atom {
        Atom::from(if i > 1 { format!("_{name}{i}") } else { format!("_{name}") })
    }

    pub fn references(&self) -> &FxHashSet<Atom> {
        &self.references
    }
}
