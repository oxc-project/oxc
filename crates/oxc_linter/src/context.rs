use std::{cell::RefCell, rc::Rc};

use oxc_ast::AstKind;
use oxc_diagnostics::Error;
use oxc_semantic::{AstNodes, Semantic};

use crate::{autofix::Fix, AstNode};

pub struct LintContext<'a> {
    source_text: &'a str,

    semantic: Rc<Semantic<'a>>,

    diagnostics: RefCell<Vec<Error>>,

    /// Whether or not to apply code fixes during linting.
    fix: bool,

    /// Collection of applicable fixes based on reported issues.
    fixes: RefCell<Vec<Fix<'a>>>,
}

impl<'a> LintContext<'a> {
    pub fn new(source_text: &'a str, semantic: Rc<Semantic<'a>>, fix: bool) -> Self {
        Self {
            source_text,
            semantic,
            diagnostics: RefCell::new(vec![]),
            fix,
            fixes: RefCell::new(vec![]),
        }
    }

    pub const fn source_text(&self) -> &'a str {
        self.source_text
    }

    pub fn into_diagnostics(self) -> (Vec<Fix<'a>>, Vec<Error>) {
        (self.fixes.into_inner(), self.diagnostics.into_inner())
    }

    pub fn diagnostic<T: Into<Error>>(&self, diagnostic: T) {
        self.diagnostics.borrow_mut().push(diagnostic.into());
    }

    pub fn fix(&self, fix: Fix<'a>) {
        if !self.fix {
            return;
        }
        self.fixes.borrow_mut().push(fix);
    }

    #[inline]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    #[inline]
    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic().nodes()
    }

    #[inline]
    pub fn parent_kind(&self, node: &AstNode<'a>) -> AstKind<'a> {
        self.nodes().parent_kind(node)
    }

    #[inline]
    pub fn parent_node(&self, node: &AstNode<'a>) -> Option<&AstNode<'a>> {
        node.parent().and_then(|node_id| self.nodes().get(node_id))
    }
}
