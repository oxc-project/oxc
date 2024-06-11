use std::cell::Cell;

use oxc_ast::ast::{BindingIdentifier, IdentifierReference};
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    reference::ReferenceFlag,
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
    /// Create `BoundIdentifier` for new binding in root scope
    pub fn new_root_uid(name: &str, flags: SymbolFlags, ctx: &mut TraverseCtx<'a>) -> Self {
        let symbol_id = ctx.generate_uid_in_root_scope(name, flags);
        let name = ctx.ast.new_atom(&ctx.symbols().names[symbol_id]);
        Self { name, symbol_id }
    }

    /// Create `IdentifierReference` referencing this binding which is read from
    /// in current scope
    pub fn create_read_reference(&self, ctx: &mut TraverseCtx) -> IdentifierReference<'a> {
        let reference_id = ctx.create_bound_reference(
            self.name.to_compact_str(),
            self.symbol_id,
            ReferenceFlag::Read,
        );
        IdentifierReference::new_read(SPAN, self.name.clone(), Some(reference_id))
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
