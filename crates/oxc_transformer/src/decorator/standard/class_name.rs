use oxc_ast::ast::*;
use oxc_ast_visit::VisitMut;
use oxc_str::Ident;
use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

/// Rewrite references to the class's own name inside class body to `replacement`.
///
/// Inside the class body, the class name refers to the inner (immutable) class binding,
/// which a class decorator may replace. Element decorators are moved outside the class body,
/// where the name would refer to a different binding.
///
/// `replacement` is created lazily on first reference, if `create_in_scope` is `Some`.
pub struct ClassNameRewriter<'a, 'ctx> {
    symbol_id: SymbolId,
    name: Ident<'a>,
    replacement: Option<BoundIdentifier<'a>>,
    create_in_scope: Option<ScopeId>,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a, 'ctx> ClassNameRewriter<'a, 'ctx> {
    pub fn new(
        symbol_id: SymbolId,
        name: Ident<'a>,
        replacement: Option<BoundIdentifier<'a>>,
        create_in_scope: Option<ScopeId>,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) -> Self {
        Self { symbol_id, name, replacement, create_in_scope, ctx }
    }

    pub fn rewrite(mut self, body: &mut ClassBody<'a>) -> Option<BoundIdentifier<'a>> {
        self.visit_class_body(body);
        self.replacement
    }
}

impl<'a> VisitMut<'a> for ClassNameRewriter<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        let symbol_id = self.ctx.scoping().get_reference(ident.reference_id()).symbol_id();
        if symbol_id != Some(self.symbol_id) {
            return;
        }
        let replacement = if let Some(replacement) = &self.replacement {
            replacement.clone()
        } else {
            let Some(scope_id) = self.create_in_scope else { return };
            let binding =
                self.ctx.generate_uid(&self.name, scope_id, SymbolFlags::BlockScopedVariable);
            self.replacement = Some(binding.clone());
            binding
        };
        let reference_id = ident.reference_id();
        let flags = self.ctx.scoping().get_reference(reference_id).flags();
        self.ctx.scoping_mut().delete_resolved_reference(self.symbol_id, reference_id);
        *ident = replacement.create_reference(flags, self.ctx);
    }
}
