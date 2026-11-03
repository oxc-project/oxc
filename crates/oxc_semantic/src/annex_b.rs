//! Annex B.3.2 hoisting of function declarations in blocks.
//!
//! In sloppy mode, a plain function declaration in a block also creates a `var` binding in the
//! enclosing function or script, unless replacing it with `var name` would be an early error
//! (a lexical declaration of `name` in an enclosing block or the var scope, or a parameter).
//! This is modeled by moving the function's binding to the var scope. The decision is made when
//! the function is bound, and undone if a conflicting lexical declaration appears later.
//!
//! <https://tc39.es/ecma262/#sec-block-level-function-declarations-web-legacy-compatibility-semantics>

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_str::Ident;
use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::SemanticBuilder;

/// Annex B state of a [`SemanticBuilder`].
#[derive(Default)]
pub struct AnnexBHoists<'a> {
    /// Hoisted functions whose hoist a later lexical declaration could still undo,
    /// keyed by `(var scope, name)`, with their block scope.
    hoists: FxHashMap<(ScopeId, Ident<'a>), (SymbolId, ScopeId)>,
    /// Keys of `hoists` in insertion order, so that the entries of a var scope can be removed
    /// when leaving it. May contain keys already removed from `hoists`.
    keys: Vec<(ScopeId, Ident<'a>)>,
    /// Symbols created by function declarations that are the body of an `if` statement.
    /// Annex B.3.3 places these in their own implicit block, so they do not prevent hoisting.
    if_statement_functions: FxHashSet<SymbolId>,
}

impl AnnexBHoists<'_> {
    /// Whether leaving `scope_id` closes a var scope with pending hoists.
    #[inline]
    pub fn has_pending_in(&self, scope_id: ScopeId) -> bool {
        self.keys.last().is_some_and(|&(var_scope_id, _)| var_scope_id == scope_id)
    }
}

impl<'a> SemanticBuilder<'a> {
    /// Hoist the plain function declaration `symbol_id` from the current block scope to the
    /// enclosing var scope, if no lexical declaration declared so far prevents it.
    pub(crate) fn hoist_annex_b_function(&mut self, name: Ident<'a>, symbol_id: SymbolId) {
        let block_scope_id = self.current_scope_id;
        let Some(var_scope_id) = self.annex_b_var_scope(block_scope_id, name) else { return };
        if self.scoping.scope_has_binding(var_scope_id, name) {
            return;
        }
        self.scoping.move_binding_by_symbol_id(block_scope_id, var_scope_id, symbol_id);
        self.hoisting_variables.insert((block_scope_id, name), symbol_id);
        self.annex_b.hoists.insert((var_scope_id, name), (symbol_id, block_scope_id));
        self.annex_b.keys.push((var_scope_id, name));
    }

    /// Find the var scope a function declaration in `block_scope_id` is hoisted to.
    ///
    /// Returns `None` if a block between them already declares `name`, because a `var name` in
    /// place of the function would then be an early error. A simple catch parameter does not
    /// prevent the hoist (Annex B.3.4), but a destructured one does.
    fn annex_b_var_scope(&self, block_scope_id: ScopeId, name: Ident<'_>) -> Option<ScopeId> {
        for scope_id in self.scoping.scope_ancestors(block_scope_id).skip(1) {
            if self.scoping.scope_flags(scope_id).is_var() {
                return Some(scope_id);
            }
            if let Some(symbol_id) = self.scoping.get_binding(scope_id, name)
                && !self
                    .scoping
                    .symbol_flags(symbol_id)
                    .contains(SymbolFlags::CatchVariable | SymbolFlags::FunctionScopedVariable)
                && !self.annex_b.if_statement_functions.contains(&symbol_id)
            {
                return None;
            }
        }
        None
    }

    /// Must be called after binding a function declaration that is the body of an `if` statement.
    pub(crate) fn declare_annex_b_if_statement_function(&mut self, symbol_id: SymbolId) {
        // Only if the declaration created the symbol, rather than redeclaring another declaration.
        if self.scoping.symbol_declaration(symbol_id) == self.node_store.current_node_id {
            self.annex_b.if_statement_functions.insert(symbol_id);
        }
    }

    /// Must be called before declaring a lexical binding `name` in the current scope: a
    /// `let`/`const`/`using`/`class` declaration, or a function declaration in a block.
    ///
    /// Undoes the hoist of a function declaration named `name` that this declaration conflicts
    /// with: any hoisted function if the current scope is the var scope, otherwise a function
    /// in a block nested in the current scope.
    #[inline]
    pub(crate) fn declare_annex_b_blocker(&mut self, name: Ident<'a>) {
        if !self.annex_b.hoists.is_empty() {
            self.unhoist_annex_b_function(name);
        }
    }

    #[cold]
    fn unhoist_annex_b_function(&mut self, name: Ident<'a>) {
        let scope_id = self.current_scope_id;
        let Some(var_scope_id) = self
            .scoping
            .scope_ancestors(scope_id)
            .find(|&id| self.scoping.scope_flags(id).is_var())
        else {
            return;
        };
        let Some(&(symbol_id, block_scope_id)) = self.annex_b.hoists.get(&(var_scope_id, name))
        else {
            return;
        };
        if scope_id != var_scope_id
            && !self.scoping.scope_is_descendant_of(block_scope_id, scope_id)
        {
            return;
        }
        self.annex_b.hoists.remove(&(var_scope_id, name));
        debug_assert_eq!(self.scoping.get_binding(var_scope_id, name), Some(symbol_id));
        debug_assert!(!self.scoping.scope_has_binding(block_scope_id, name));
        self.hoisting_variables.remove(&(block_scope_id, name));
        self.scoping.move_binding_by_symbol_id(var_scope_id, block_scope_id, symbol_id);
    }

    /// Must be called after binding a `var` declaration of `name`, or a function declaration
    /// directly in the var scope `var_scope_id`.
    ///
    /// A hoisted function of the same name then keeps its hoisted binding, since a later lexical
    /// declaration of that name in the var scope is a genuine redeclaration.
    #[inline]
    pub(crate) fn declare_annex_b_var(&mut self, var_scope_id: ScopeId, name: Ident<'a>) {
        if !self.annex_b.hoists.is_empty() {
            self.annex_b.hoists.remove(&(var_scope_id, name));
        }
    }

    /// Called when leaving a var scope with pending hoists. No later declaration can conflict
    /// with them.
    #[cold]
    pub(crate) fn finish_annex_b_hoists(&mut self) {
        let var_scope_id = self.current_scope_id;
        while let Some(&key) = self.annex_b.keys.last()
            && key.0 == var_scope_id
        {
            self.annex_b.keys.pop();
            self.annex_b.hoists.remove(&key);
        }
    }
}
