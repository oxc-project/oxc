use std::{
    cell::{Ref, RefCell, RefMut},
    mem,
    rc::Rc,
};

use oxc_ast::AstBuilder;
use oxc_diagnostics::Error;
use oxc_semantic::{ScopeId, ScopeTree, Semantic, SymbolId, SymbolTable};
use oxc_span::Atom;

#[derive(Clone)]
pub struct TransformerCtx<'a> {
    pub ast: Rc<AstBuilder<'a>>,
    semantic: Rc<RefCell<Semantic<'a>>>,
    errors: Rc<RefCell<Vec<Error>>>,
}

impl<'a> TransformerCtx<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, semantic: Rc<RefCell<Semantic<'a>>>) -> Self {
        Self { ast, semantic, errors: Rc::new(RefCell::new(vec![])) }
    }

    pub fn semantic(&self) -> Ref<'_, Semantic<'a>> {
        self.semantic.borrow()
    }

    pub fn symbols(&self) -> Ref<'_, SymbolTable> {
        Ref::map(self.semantic.borrow(), |semantic| semantic.symbols())
    }

    pub fn scopes(&self) -> Ref<'_, ScopeTree> {
        Ref::map(self.semantic.borrow(), |semantic| semantic.scopes())
    }

    pub fn scopes_mut(&self) -> RefMut<'_, ScopeTree> {
        RefMut::map(self.semantic.borrow_mut(), |semantic| semantic.scopes_mut())
    }

    pub fn add_binding(&self, name: Atom) {
        // TODO: use the correct scope and symbol id
        self.scopes_mut().add_binding(ScopeId::new(0), name, SymbolId::new(0));
    }

    pub fn errors(&self) -> Vec<Error> {
        mem::replace(&mut self.errors.borrow_mut(), vec![])
    }

    /// Push a Transform Error
    pub fn error<T: Into<Error>>(&mut self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }
}
