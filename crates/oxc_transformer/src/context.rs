use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use oxc_ast::AstBuilder;
use oxc_semantic::{ScopeId, ScopeTree, SymbolId, SymbolTable};
use oxc_span::Atom;

#[derive(Clone)]
pub struct TransformerCtx<'a> {
    pub ast: Rc<AstBuilder<'a>>,
    pub symbols: Rc<RefCell<SymbolTable>>,
    pub scopes: Rc<RefCell<ScopeTree>>,
}

impl<'a> TransformerCtx<'a> {
    pub fn symbols(&self) -> Ref<SymbolTable> {
        self.symbols.borrow()
    }

    pub fn scopes(&self) -> Ref<ScopeTree> {
        self.scopes.borrow()
    }

    pub fn add_binding(&self, name: Atom) {
        // TODO: use the correct scope and symbol id
        self.scopes.borrow_mut().add_binding(ScopeId::new(0), name, SymbolId::new(0));
    }
}
