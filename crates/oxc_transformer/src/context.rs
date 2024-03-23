use std::{
    borrow::Cow,
    cell::{Ref, RefCell, RefMut},
    mem,
    rc::Rc,
};

use oxc_ast::AstBuilder;
use oxc_diagnostics::Error;
use oxc_semantic::{ScopeId, ScopeTree, Semantic, SymbolId, SymbolTable};
use oxc_span::{CompactStr, SourceType};

use crate::TransformOptions;

#[derive(Clone)]
pub struct TransformerCtx<'a> {
    pub ast: Rc<AstBuilder<'a>>,
    pub options: Cow<'a, TransformOptions>,
    semantic: Rc<RefCell<Semantic<'a>>>,
    errors: Rc<RefCell<Vec<Error>>>,
}

impl<'a> TransformerCtx<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, semantic: Semantic<'a>, options: TransformOptions) -> Self {
        Self {
            ast,
            semantic: Rc::new(RefCell::new(semantic)),
            options: Cow::Owned(options),
            errors: Rc::new(RefCell::new(vec![])),
        }
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

    pub fn add_binding(&self, name: CompactStr) {
        // TODO: use the correct scope and symbol id
        self.scopes_mut().add_binding(ScopeId::new(0), name, SymbolId::new(0));
    }

    pub fn source_type(&self) -> Ref<'_, SourceType> {
        Ref::map(self.semantic.borrow(), |semantic| semantic.source_type())
    }

    pub fn errors(&self) -> Vec<Error> {
        mem::take(&mut self.errors.borrow_mut())
    }

    /// Push a Transform Error
    pub fn error<T: Into<Error>>(&self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }
}
