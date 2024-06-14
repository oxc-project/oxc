use std::{cell::RefCell, mem, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_diagnostics::OxcDiagnostic;

pub type Ctx<'a> = Rc<TransformDtsCtx<'a>>;

pub struct TransformDtsCtx<'a> {
    errors: RefCell<Vec<OxcDiagnostic>>,
    pub ast: AstBuilder<'a>,
}

impl<'a> TransformDtsCtx<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { errors: RefCell::new(vec![]), ast: AstBuilder::new(allocator) }
    }

    pub fn take_errors(&self) -> Vec<OxcDiagnostic> {
        mem::take(&mut self.errors.borrow_mut())
    }

    /// Add an Error
    pub fn error(&self, error: OxcDiagnostic) {
        self.errors.borrow_mut().push(error);
    }
}
