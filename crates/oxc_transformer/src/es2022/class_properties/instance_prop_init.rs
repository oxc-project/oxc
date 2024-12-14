//! ES2022: Class Properties
//! Transform of instance property initializers.

use std::cell::Cell;

use oxc_ast::{ast::*, visit::Visit};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
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

/// Visitor to change parent scope of first-level scopes in instance property initializer.
struct InstanceInitializerVisitor<'a, 'v> {
    /// Incremented when entering a scope, decremented when exiting it.
    /// Parent `ScopeId` should be updated when `scope_depth == 0`.
    scope_depth: u32,
    /// Parent scope
    parent_scope_id: ScopeId,
    /// `TraverseCtx` object.
    ctx: &'v mut TraverseCtx<'a>,
}

impl<'a, 'v> InstanceInitializerVisitor<'a, 'v> {
    fn new(
        class_properties: &'v mut ClassProperties<'a, '_>,
        ctx: &'v mut TraverseCtx<'a>,
    ) -> Self {
        let parent_scope_id = class_properties.instance_inits_scope_id;
        Self { scope_depth: 0, parent_scope_id, ctx }
    }
}

impl<'a, 'v> Visit<'a> for InstanceInitializerVisitor<'a, 'v> {
    /// Update parent scope for first level of scopes.
    /// Convert scope to sloppy mode if `self.make_sloppy_mode == true`.
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        let scope_id = scope_id.get().unwrap();

        // TODO: Not necessary to do this check for all scopes.
        // In JS, only `Function`, `ArrowFunctionExpression` or `Class` can be the first-level scope,
        // as all other types which have a scope are statements or `StaticBlock` which would need to be
        // inside a function or class. But some TS types with scopes could be first level via
        // e.g. `TaggedTemplateExpression::type_parameters`, which contains `TSType`.
        // Not sure if that matters though, as they'll be stripped out anyway by TS transform.
        if self.scope_depth == 0 {
            self.reparent_scope(scope_id);
        }
        self.scope_depth += 1;
    }

    fn leave_scope(&mut self) {
        self.scope_depth -= 1;
    }
}

impl<'a, 'v> InstanceInitializerVisitor<'a, 'v> {
    /// Update parent of scope to scope above class.
    fn reparent_scope(&mut self, scope_id: ScopeId) {
        self.ctx.scopes_mut().change_parent_id(scope_id, Some(self.parent_scope_id));
    }
}
