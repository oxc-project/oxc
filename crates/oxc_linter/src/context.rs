use std::{cell::RefCell, rc::Rc};

use oxc_diagnostics::Error;
use oxc_semantic::Semantic;

pub struct LintContext<'a> {
    semantic: Rc<Semantic<'a>>,

    diagnostics: RefCell<Vec<Error>>,
}

impl<'a> LintContext<'a> {
    pub fn new(semantic: Rc<Semantic<'a>>) -> Self {
        Self { semantic, diagnostics: RefCell::new(vec![]) }
    }

    pub fn into_diagnostics(self) -> Vec<Error> {
        self.diagnostics.into_inner()
    }

    #[allow(unused)]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    pub fn diagnostic<T: Into<Error>>(&self, diagnostic: T) {
        self.diagnostics.borrow_mut().push(diagnostic.into());
    }
}
