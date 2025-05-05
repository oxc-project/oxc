use std::str;

use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_semantic::{NodeId, Reference, Scoping};
use oxc_span::SPAN;
use oxc_syntax::{
    reference::{ReferenceFlags, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use crate::{BoundIdentifier, scopes_collector::ChildScopeCollector};

use super::uid::UidGenerator;

/// Traverse scope context.
///
/// Contains the scope tree and symbols table, and provides methods to access them.
///
/// `current_scope_id` is the ID of current scope during traversal.
/// `walk_*` functions update this field when entering/exiting a scope.
pub struct TraverseScoping<'a> {
    scoping: Scoping,
    uid_generator: Option<UidGenerator<'a>>,
    current_scope_id: ScopeId,
    current_hoist_scope_id: ScopeId,
    current_block_scope_id: ScopeId,
}

// Public methods
impl<'a> TraverseScoping<'a> {
    /// Get current scope ID
    #[inline]
    pub fn current_scope_id(&self) -> ScopeId {
        self.current_scope_id
    }

    /// Get current var hoisting scope ID
    #[inline]
    pub(crate) fn current_hoist_scope_id(&self) -> ScopeId {
        self.current_hoist_scope_id
    }

    /// Get current block scope ID
    #[inline]
    pub(crate) fn current_block_scope_id(&self) -> ScopeId {
        self.current_block_scope_id
    }

    /// Get current scope flags
    #[inline]
    pub fn current_scope_flags(&self) -> ScopeFlags {
        self.scoping.scope_flags(self.current_scope_id)
    }

    /// Get scopes tree
    #[inline]
    pub fn scoping(&self) -> &Scoping {
        &self.scoping
    }

    /// Get mutable scopes tree
    #[inline]
    pub fn scoping_mut(&mut self) -> &mut Scoping {
        &mut self.scoping
    }

    /// Get iterator over scopes, starting with current scope and working up
    pub fn ancestor_scopes(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.scoping.scope_ancestors(self.current_scope_id)
    }

    /// Create new scope as child of provided scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn create_child_scope(&mut self, parent_id: ScopeId, flags: ScopeFlags) -> ScopeId {
        let flags = self.scoping.get_new_scope_flags(flags, parent_id);
        self.scoping.add_scope(Some(parent_id), NodeId::DUMMY, flags)
    }

    /// Create new scope as child of current scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn create_child_scope_of_current(&mut self, flags: ScopeFlags) -> ScopeId {
        self.create_child_scope(self.current_scope_id, flags)
    }

    /// Insert a scope into scope tree below a statement.
    ///
    /// Statement must be in current scope.
    /// New scope is created as child of current scope.
    /// All child scopes of the statement are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn insert_scope_below_statement(&mut self, stmt: &Statement, flags: ScopeFlags) -> ScopeId {
        self.insert_scope_below_statement_from_scope_id(stmt, self.current_scope_id, flags)
    }

    /// Insert a scope into scope tree below a statement.
    ///
    /// Statement must be in provided scope.
    /// New scope is created as child of the provided scope.
    /// All child scopes of the statement are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn insert_scope_below_statement_from_scope_id(
        &mut self,
        stmt: &Statement,
        scope_id: ScopeId,
        flags: ScopeFlags,
    ) -> ScopeId {
        let mut collector = ChildScopeCollector::new();
        collector.visit_statement(stmt);
        self.insert_scope_below(scope_id, &collector.scope_ids, flags)
    }

    /// Insert a scope into scope tree below an expression.
    ///
    /// Expression must be in current scope.
    /// New scope is created as child of current scope.
    /// All child scopes of the expression are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn insert_scope_below_expression(
        &mut self,
        expr: &Expression,
        flags: ScopeFlags,
    ) -> ScopeId {
        let mut collector = ChildScopeCollector::new();
        collector.visit_expression(expr);
        self.insert_scope_below(self.current_scope_id, &collector.scope_ids, flags)
    }

    /// Insert a scope into scope tree below a `Vec` of statements.
    ///
    /// Statements must be in current scope.
    /// New scope is created as child of current scope.
    /// All child scopes of the statement are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn insert_scope_below_statements(
        &mut self,
        stmts: &ArenaVec<Statement>,
        flags: ScopeFlags,
    ) -> ScopeId {
        let mut collector = ChildScopeCollector::new();
        collector.visit_statements(stmts);
        self.insert_scope_below(self.current_scope_id, &collector.scope_ids, flags)
    }

    fn insert_scope_below(
        &mut self,
        scope_id: ScopeId,
        child_scope_ids: &[ScopeId],
        flags: ScopeFlags,
    ) -> ScopeId {
        // Remove these scopes from parent's children
        if self.scoping.has_scope_child_ids() {
            self.scoping.remove_child_scopes(scope_id, child_scope_ids);
        }

        // Create new scope as child of parent
        let new_scope_id = self.create_child_scope(scope_id, flags);

        // Set scopes as children of new scope instead
        for &child_id in child_scope_ids {
            self.scoping.set_scope_parent_id(child_id, Some(new_scope_id));
        }

        new_scope_id
    }

    /// Insert a scope between a parent and a child scope.
    ///
    /// For example, given the following scopes
    /// ```ts
    /// parentScope1: {
    ///     childScope: { }
    ///     childScope2: { }
    /// }
    /// ```
    /// and calling this function with `parentScope1` and `childScope`,
    /// the resulting scopes will be:
    /// ```ts
    /// parentScope1: {
    ///     newScope: {
    ///         childScope: { }
    ///     }
    ///     childScope2: { }
    /// }
    /// ```
    pub fn insert_scope_between(
        &mut self,
        parent_id: ScopeId,
        child_id: ScopeId,
        flags: ScopeFlags,
    ) -> ScopeId {
        let scope_id = self.create_child_scope(parent_id, flags);

        debug_assert_eq!(
            self.scoping.scope_parent_id(child_id),
            Some(parent_id),
            "Child scope must be a child of parent scope"
        );

        if self.scoping.has_scope_child_ids() {
            self.scoping.remove_child_scope(parent_id, child_id);
        }
        self.scoping.set_scope_parent_id(child_id, Some(scope_id));
        scope_id
    }

    /// Remove scope for an expression from the scope chain.
    ///
    /// Delete the scope and set parent of its child scopes to its parent scope.
    /// e.g.:
    /// * Starting scopes parentage `A -> B`, `B -> C`, `B -> D`.
    /// * Remove scope `B` from chain.
    /// * End result: scopes `A -> C`, `A -> D`.
    ///
    /// Use this when removing an expression which owns a scope, without removing its children.
    /// For example when unwrapping `(() => foo)()` to just `foo`.
    /// `foo` here could be an expression which itself contains scopes.
    pub fn remove_scope_for_expression(&mut self, scope_id: ScopeId, expr: &Expression) {
        let mut collector = ChildScopeCollector::new();
        collector.visit_expression(expr);

        let child_ids = collector.scope_ids;
        if !child_ids.is_empty() {
            let parent_id = self.scoping.scope_parent_id(scope_id);
            for child_id in child_ids {
                self.scoping.set_scope_parent_id(child_id, parent_id);
            }
        }

        self.scoping.delete_scope(scope_id);
    }

    /// Add binding to `ScopeTree` and `SymbolTable`.
    #[inline]
    pub(crate) fn add_binding(
        &mut self,
        name: &str,
        scope_id: ScopeId,
        flags: SymbolFlags,
    ) -> SymbolId {
        let symbol_id = self.scoping.create_symbol(SPAN, name, flags, scope_id, NodeId::DUMMY);
        self.scoping.add_binding(scope_id, name, symbol_id);

        symbol_id
    }

    /// Generate binding.
    ///
    /// Creates a symbol with the provided name and flags and adds it to the specified scope.
    pub fn generate_binding(
        &mut self,
        name: Atom<'a>,
        scope_id: ScopeId,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        let symbol_id = self.add_binding(name.as_str(), scope_id, flags);
        BoundIdentifier::new(name, symbol_id)
    }

    /// Generate binding in current scope.
    ///
    /// Creates a symbol with the provided name and flags and adds it to the current scope.
    pub fn generate_binding_in_current_scope(
        &mut self,
        name: Atom<'a>,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        self.generate_binding(name, self.current_scope_id, flags)
    }

    /// Generate UID var name.
    ///
    /// Finds a unique variable name which does clash with any other variables used in the program.
    ///
    /// Caller must ensure `name` is a valid JS identifier, after a `_` is prepended on start.
    /// The fact that a `_` will be prepended on start means providing an empty string or a string
    /// starting with a digit (0-9) is fine.
    ///
    /// See comments on `UidGenerator` for further details.
    pub fn generate_uid_name(&mut self, name: &str, allocator: &'a Allocator) -> Atom<'a> {
        // If `uid_generator` is not already populated, initialize it
        let uid_generator =
            self.uid_generator.get_or_insert_with(|| UidGenerator::new(&self.scoping, allocator));
        // Generate unique name
        uid_generator.create(name)
    }

    /// Create a reference bound to a `SymbolId`
    pub fn create_bound_reference(
        &mut self,
        symbol_id: SymbolId,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let reference = Reference::new_with_symbol_id(NodeId::DUMMY, symbol_id, flags);
        let reference_id = self.scoping.create_reference(reference);
        self.scoping.add_resolved_reference(symbol_id, reference_id);
        reference_id
    }

    /// Create an unbound reference
    pub fn create_unbound_reference(&mut self, name: &str, flags: ReferenceFlags) -> ReferenceId {
        let reference = Reference::new(NodeId::DUMMY, flags);
        let reference_id = self.scoping.create_reference(reference);
        self.scoping.add_root_unresolved_reference(name, reference_id);
        reference_id
    }

    /// Create a reference optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_reference`
    /// or `TraverseCtx::create_unbound_reference`.
    pub fn create_reference(
        &mut self,
        name: &str,
        symbol_id: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        if let Some(symbol_id) = symbol_id {
            self.create_bound_reference(symbol_id, flags)
        } else {
            self.create_unbound_reference(name, flags)
        }
    }

    /// Create reference in current scope, looking up binding for `name`
    pub fn create_reference_in_current_scope(
        &mut self,
        name: &str,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let symbol_id = self.scoping.find_binding(self.current_scope_id, name);
        self.create_reference(name, symbol_id, flags)
    }

    /// Delete a reference.
    ///
    /// Provided `name` must match `reference_id`.
    pub fn delete_reference(&mut self, reference_id: ReferenceId, name: &str) {
        let symbol_id = self.scoping.get_reference(reference_id).symbol_id();
        if let Some(symbol_id) = symbol_id {
            self.scoping.delete_resolved_reference(symbol_id, reference_id);
        } else {
            self.scoping.delete_root_unresolved_reference(name, reference_id);
        }
    }

    /// Delete reference for an `IdentifierReference`.
    pub fn delete_reference_for_identifier(&mut self, ident: &IdentifierReference) {
        self.delete_reference(ident.reference_id(), &ident.name);
    }
}

// Methods used internally within crate
impl TraverseScoping<'_> {
    /// Create new `TraverseScoping`
    pub(super) fn new(scoping: Scoping) -> Self {
        Self {
            scoping,
            uid_generator: None,
            // Dummy values. Both immediately overwritten in `walk_program`.
            current_scope_id: ScopeId::new(0),
            current_hoist_scope_id: ScopeId::new(0),
            current_block_scope_id: ScopeId::new(0),
        }
    }

    /// Consume [`TraverseScoping`] and return [`Scoping`].
    pub(super) fn into_scoping(self) -> Scoping {
        self.scoping
    }

    /// Set current scope ID
    #[inline]
    pub(crate) fn set_current_scope_id(&mut self, scope_id: ScopeId) {
        self.current_scope_id = scope_id;
    }

    /// Set current hoist scope ID
    #[inline]
    pub(crate) fn set_current_hoist_scope_id(&mut self, scope_id: ScopeId) {
        self.current_hoist_scope_id = scope_id;
    }

    /// Set current block scope ID
    #[inline]
    pub(crate) fn set_current_block_scope_id(&mut self, scope_id: ScopeId) {
        self.current_block_scope_id = scope_id;
    }

    pub fn delete_typescript_bindings(&mut self) {
        self.scoping.delete_typescript_bindings();
    }
}
