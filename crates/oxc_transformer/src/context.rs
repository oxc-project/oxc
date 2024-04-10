use std::{cell::RefCell, mem, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;

pub type Ctx<'a> = Rc<TransformCtx<'a>>;

pub struct TransformCtx<'a> {
    pub ast: AstBuilder<'a>,
    pub source_type: SourceType,
    pub semantic: Semantic<'a>,
    errors: RefCell<Vec<Error>>,
}

impl<'a> TransformCtx<'a> {
    pub fn new(allocator: &'a Allocator, source_type: SourceType, semantic: Semantic<'a>) -> Self {
        Self {
            ast: AstBuilder::new(allocator),
            source_type,
            semantic,
            errors: RefCell::new(vec![]),
        }
    }

    pub fn take_errors(&self) -> Vec<Error> {
        mem::take(&mut self.errors.borrow_mut())
    }

    /// Add an Error
    #[allow(unused)]
    pub fn error<T: Into<Error>>(&self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }
}
