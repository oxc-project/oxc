use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use oxc_ast::AstBuilder;
use oxc_semantic::{ScopeId, ScopeTree, Semantic, SymbolId, SymbolTable};
use oxc_span::Atom;

#[derive(Clone)]
pub struct TransformerCtx<'a> {
    pub ast: Rc<AstBuilder<'a>>,
    pub semantic: Rc<RefCell<Semantic<'a>>>,
}

impl<'a> TransformerCtx<'a> {
    pub fn symbols(&self) -> Ref<'_, SymbolTable> {
        Ref::map(self.semantic.borrow(), |semantic| semantic.symbols())
    }

    pub fn scopes(&self) -> Ref<'_, ScopeTree> {
        Ref::map(self.semantic.borrow(), |semantic| semantic.scopes())
    }

    pub fn source_text(&self) -> &str {
        return self.semantic.borrow().source_text();
    }

    pub fn add_binding(&self, name: Atom) {
        // TODO: use the correct scope and symbol id
        self.semantic.borrow_mut().scopes_mut().add_binding(
            ScopeId::new(0),
            name,
            SymbolId::new(0),
        );
    }
}
