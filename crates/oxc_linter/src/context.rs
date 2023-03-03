use std::{cell::RefCell, rc::Rc};

use oxc_ast::AstKind;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;

use crate::{autofix::Fix, AstNode};

pub struct LintContext<'a> {
    source_text: &'a str,

    semantic: Rc<Semantic<'a>>,

    diagnostics: RefCell<Vec<Error>>,

    fixes: RefCell<Vec<Fix>>,
}

impl<'a> LintContext<'a> {
    pub fn new(source_text: &'a str, semantic: Rc<Semantic<'a>>) -> Self {
        Self {
            source_text,
            semantic,
            diagnostics: RefCell::new(vec![]),
            fixes: RefCell::new(vec![]),
        }
    }

    pub const fn source_text(&self) -> &'a str {
        self.source_text
    }

    pub fn into_diagnostics(self) -> (Vec<Fix>, Vec<Error>) {
        (self.fixes.into_inner(), self.diagnostics.into_inner())
    }

    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    pub fn diagnostic<T: Into<Error>>(&self, diagnostic: T) {
        self.diagnostics.borrow_mut().push(diagnostic.into());
    }

    pub fn fix(&self, fix: Fix) {
        self.fixes.borrow_mut().push(fix);
    }

    pub fn into_fixes(self) -> Vec<Fix> {
        self.fixes.into_inner()
    }

    pub fn parent_kind(&self, node: &AstNode<'a>) -> AstKind<'a> {
        self.semantic().nodes().parent_kind(node)
    }
}
