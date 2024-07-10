use std::cell::Cell;

use oxc_ast::ast::{BindingIdentifier, IdentifierReference};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::{
    reference::ReferenceFlag,
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::TraverseCtx;

/// Store for a created binding identifier
#[derive(Clone)]
pub struct BoundIdentifier<'a> {
    pub name: Atom<'a>,
    pub symbol_id: SymbolId,
}

impl<'a> BoundIdentifier<'a> {
    /// Create `BoundIdentifier` for new binding
    pub fn new_uid(
        name: &str,
        scope_id: ScopeId,
        flags: SymbolFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> Self {
        let symbol_id = ctx.generate_uid(name, scope_id, flags);
        let name = ctx.ast.atom(&ctx.symbols().names[symbol_id]);
        Self { name, symbol_id }
    }

    /// Create `BoundIdentifier` for new binding in root scope
    pub fn new_root_uid(name: &str, flags: SymbolFlags, ctx: &mut TraverseCtx<'a>) -> Self {
        let scope_id = ctx.scopes().root_scope_id();
        Self::new_uid(name, scope_id, flags, ctx)
    }

    /// Create `IdentifierReference` referencing this binding which is read from
    /// in current scope
    pub fn create_read_reference(&self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        self.create_spanned_read_reference(SPAN, ctx)
    }

    /// Create `IdentifierReference` referencing this binding which is read from
    /// in current scope
    pub fn create_spanned_read_reference(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        ctx.create_bound_reference_id(span, self.name.clone(), self.symbol_id, ReferenceFlag::Read)
    }

    /// Create `BindingIdentifier` for this binding
    pub fn create_binding_identifier(&self) -> BindingIdentifier<'a> {
        BindingIdentifier {
            span: SPAN,
            name: self.name.clone(),
            symbol_id: Cell::new(Some(self.symbol_id)),
        }
    }
}
