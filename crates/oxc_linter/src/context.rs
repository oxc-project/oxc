use std::{cell::RefCell, rc::Rc};

use indextree::{Ancestors, NodeId};
use oxc_ast::{ast::IdentifierReference, AstKind, SourceType};
use oxc_diagnostics::Error;
use oxc_printer::{Printer, PrinterOptions};
use oxc_semantic::{AstNodes, Scope, ScopeTree, Semantic, SemanticNode};

use crate::{
    fixer::{Fix, Message},
    AstNode,
};

pub struct LintContext<'a> {
    source_text: &'a str,

    semantic: Rc<Semantic<'a>>,

    diagnostics: RefCell<Vec<Message<'a>>>,

    /// Whether or not to apply code fixes during linting.
    fix: bool,
}

impl<'a> LintContext<'a> {
    pub fn new(source_text: &'a str, semantic: Rc<Semantic<'a>>, fix: bool) -> Self {
        Self { source_text, semantic, diagnostics: RefCell::new(vec![]), fix }
    }

    #[must_use]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    #[must_use]
    pub fn source_text(&self) -> &'a str {
        self.source_text
    }

    #[must_use]
    pub fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    /* Diagnostics */

    pub fn into_message(self) -> Vec<Message<'a>> {
        self.diagnostics.into_inner()
    }

    pub fn diagnostic<T: Into<Error>>(&self, diagnostic: T) {
        self.diagnostics.borrow_mut().push(Message::new(diagnostic.into(), None));
    }

    pub fn diagnostic_with_fix<T, F>(&self, diagnostic: T, fix: F)
    where
        T: Into<Error>,
        F: FnOnce() -> Fix<'a>,
    {
        if self.fix {
            self.diagnostics.borrow_mut().push(Message::new(diagnostic.into(), Some(fix())));
        } else {
            self.diagnostic(diagnostic);
        }
    }

    /* Nodes */

    #[must_use]
    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic().nodes()
    }

    #[must_use]
    pub fn kind(&self, node_id: NodeId) -> AstKind<'a> {
        self.nodes().kind(node_id)
    }

    #[must_use]
    pub fn parent_kind(&self, node: &AstNode<'a>) -> AstKind<'a> {
        self.nodes().parent_kind(node)
    }

    #[must_use]
    pub fn parent_node(&self, node: &AstNode<'a>) -> Option<&AstNode<'a>> {
        node.parent().and_then(|node_id| self.nodes().get(node_id))
    }

    #[must_use]
    pub fn ancestors(&self, node: &AstNode<'a>) -> Ancestors<'_, SemanticNode<'a>> {
        let node_id = self.nodes().get_node_id(node).unwrap();
        node_id.ancestors(self.nodes())
    }

    /* Scopes */

    #[must_use]
    pub fn scopes(&self) -> &ScopeTree {
        self.semantic().scopes()
    }

    #[must_use]
    pub fn scope(&self, node: &AstNode) -> &Scope {
        self.semantic().scopes().node_scope(node)
    }

    #[must_use]
    pub fn scope_ancestors(&self, node: &AstNode) -> Ancestors<'_, Scope> {
        self.semantic().scopes().ancestors(node.get().scope_id())
    }

    #[must_use]
    pub fn strict_mode(&self, node: &AstNode) -> bool {
        let scope = self.scope(node);
        node.get().strict_mode(scope)
    }

    /* Symbols */

    #[allow(clippy::unused_self)]
    pub fn is_reference_to_global_variable(&self, _ident: &IdentifierReference) -> bool {
        true
    }

    #[allow(clippy::unused_self)]
    pub fn printer(&self) -> Printer {
        Printer::new(0, PrinterOptions::default())
    }
}
