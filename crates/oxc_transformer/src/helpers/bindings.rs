use oxc_ast::ast::IdentifierReference;
use oxc_span::{Atom, SPAN};
use oxc_syntax::{reference::ReferenceFlag, symbol::SymbolId};
use oxc_traverse::TraverseCtx;

/// Store for a created binding identifier
#[derive(Clone)]
pub struct BoundIdentifier<'a> {
    pub name: Atom<'a>,
    pub symbol_id: SymbolId,
}

impl<'a> BoundIdentifier<'a> {
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
}
