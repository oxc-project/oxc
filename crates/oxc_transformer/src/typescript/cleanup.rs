//! Record semantic data in erased syntax before its AST nodes are discarded.

use oxc_ast::ast::{BindingIdentifier, IdentifierReference};
use oxc_ast_visit::Visit;
use oxc_span::Span;
use oxc_syntax::{reference::ReferenceId, symbol::SymbolId};
use rustc_hash::FxHashSet;

use crate::context::TraverseCtx;

#[derive(Default)]
pub struct TypeScriptCleanup {
    declarations: FxHashSet<(SymbolId, Span)>,
    references: FxHashSet<ReferenceId>,
}

impl TypeScriptCleanup {
    pub fn finish(self, ctx: &mut TraverseCtx<'_>) {
        let scoping = ctx.scoping_mut();
        scoping.remove_references(&self.references);
        scoping.delete_typescript_bindings_with(|id, span| self.declarations.contains(&(id, span)));
    }
}

/// All removals are deferred until import/export retention has finished.
pub(super) struct Erase<'c, 'a>(pub &'c mut TraverseCtx<'a>);

impl<'a> Visit<'a> for Erase<'_, 'a> {
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if let Some(id) = ident.symbol_id.get() {
            self.0.state.typescript_cleanup.declarations.insert((id, ident.span));
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(id) = ident.reference_id.get() {
            self.0.state.typescript_cleanup.references.insert(id);
        }
    }
}
