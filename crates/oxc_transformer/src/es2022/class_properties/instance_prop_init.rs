//! ES2022: Class Properties
//! Transform of instance property initializers.

use std::cell::Cell;

use rustc_hash::FxHashMap;

use oxc_ast::{ast::*, visit::Visit};
use oxc_data_structures::stack::Stack;
use oxc_span::Atom;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};
use oxc_traverse::TraverseCtx;

use super::ClassProperties;

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Transform instance property initializer.
    ///
    /// Instance property initializers move from the class body into either class constructor,
    /// or a `_super` function. Change parent scope of first-level scopes in initializer to reflect this.
    pub(super) fn transform_instance_initializer(
        &mut self,
        value: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut updater = InstanceInitializerVisitor::new(self, ctx);
        updater.visit_expression(value);
    }
}

// TODO: If no `constructor_scope_id`, then don't need to traverse beyond first-level scope,
// as all we need to do is update scopes. Add a faster visitor for this more limited traversal.

/// Visitor to change parent scope of first-level scopes in instance property initializer,
/// and find any `IdentifierReference`s which would be shadowed by bindings in constructor,
/// once initializer moves into constructor body.
struct InstanceInitializerVisitor<'a, 'v> {
    /// Pushed to when entering a scope, popped when exiting it.
    /// Parent `ScopeId` should be updated when stack is empty (i.e. this is a first-level scope).
    scope_ids_stack: Stack<ScopeId>,
    /// New parent scope for first-level scopes in initializer
    parent_scope_id: ScopeId,
    /// Constructor scope, if need to check for clashing bindings with constructor.
    /// `None` if constructor is newly created, or inits are being inserted in `_super` function
    /// outside class, because in those cases there are no bindings which can clash.
    constructor_scope_id: Option<ScopeId>,
    /// Clashing symbols
    clashing_symbols: &'v mut FxHashMap<SymbolId, Atom<'a>>,
    /// `TraverseCtx` object.
    ctx: &'v mut TraverseCtx<'a>,
}

impl<'a, 'v> InstanceInitializerVisitor<'a, 'v> {
    fn new(
        class_properties: &'v mut ClassProperties<'a, '_>,
        ctx: &'v mut TraverseCtx<'a>,
    ) -> Self {
        Self {
            // Most initializers don't contain any scopes, so best default is 0 capacity
            // to avoid an allocation in most cases
            scope_ids_stack: Stack::new(),
            parent_scope_id: class_properties.instance_inits_scope_id,
            constructor_scope_id: class_properties.instance_inits_constructor_scope_id,
            clashing_symbols: &mut class_properties.clashing_constructor_symbols,
            ctx,
        }
    }
}

impl<'a, 'v> Visit<'a> for InstanceInitializerVisitor<'a, 'v> {
    /// Update parent scope for first level of scopes.
    /// Convert scope to sloppy mode if `self.make_sloppy_mode == true`.
    //
    // `#[inline]` because this function is small and called from many `walk` functions
    #[inline]
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        let scope_id = scope_id.get().unwrap();
        if self.scope_ids_stack.is_empty() {
            self.reparent_scope(scope_id);
        }
        self.scope_ids_stack.push(scope_id);
    }

    // `#[inline]` because this function is tiny and called from many `walk` functions
    #[inline]
    fn leave_scope(&mut self) {
        self.scope_ids_stack.pop();
    }

    // `#[inline]` because this is a hot path
    #[inline]
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let Some(constructor_scope_id) = self.constructor_scope_id else { return };

        self.check_for_symbol_clash(ident, constructor_scope_id);
    }
}

impl<'a, 'v> InstanceInitializerVisitor<'a, 'v> {
    /// Update parent of scope.
    fn reparent_scope(&mut self, scope_id: ScopeId) {
        self.ctx.scopes_mut().change_parent_id(scope_id, Some(self.parent_scope_id));
    }

    /// Check if symbol referenced by `ident` is shadowed by a binding in constructor's scope.
    fn check_for_symbol_clash(
        &mut self,
        ident: &IdentifierReference<'a>,
        constructor_scope_id: ScopeId,
    ) {
        // TODO: It would be ideal if could get reference `&Bindings` for constructor
        // in `InstanceInitializerVisitor::new` rather than indexing into `ScopeTree::bindings`
        // with same `ScopeId` every time here, but `ScopeTree` doesn't allow that, and we also
        // take a `&mut ScopeTree` in `reparent_scope`, so borrow-checker doesn't allow that.
        let Some(constructor_symbol_id) =
            self.ctx.scopes().get_binding(constructor_scope_id, &ident.name)
        else {
            return;
        };

        // Check the symbol this identifier refers to is bound outside of the initializer itself.
        // If it's bound within the initializer, there's no clash, so exit.
        // e.g. `class C { double = (n) => n * 2; constructor(n) {} }`
        // Even though there's a binding `n` in constructor, it doesn't shadow the use of `n` in init.
        // This is an improvement over Babel.
        let reference_id = ident.reference_id();
        if let Some(ident_symbol_id) = self.ctx.symbols().get_reference(reference_id).symbol_id() {
            let scope_id = self.ctx.symbols().get_scope_id(ident_symbol_id);
            if self.scope_ids_stack.contains(&scope_id) {
                return;
            }
        }

        // Record the symbol clash. Symbol in constructor needs to be renamed.
        self.clashing_symbols.entry(constructor_symbol_id).or_insert(ident.name.clone());
    }
}
