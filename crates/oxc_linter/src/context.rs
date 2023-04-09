use std::{cell::RefCell, rc::Rc};

use indextree::{Ancestors, NodeId};
use oxc_ast::{ast::IdentifierReference, AstKind, SourceType};
use oxc_diagnostics::Error;
use oxc_printer::{Printer, PrinterOptions};
use oxc_semantic::{AstNodes, Scope, ScopeTree, Semantic, SemanticNode, SymbolTable};

use crate::{
    disable_directives::{DisableDirectives, DisableDirectivesBuilder},
    fixer::{Fix, Message},
    AstNode,
};

pub struct LintContext<'a> {
    semantic: Rc<Semantic<'a>>,

    diagnostics: RefCell<Vec<Message<'a>>>,

    disable_directives: DisableDirectives<'a>,

    /// Whether or not to apply code fixes during linting.
    fix: bool,

    current_rule_name: &'static str,
}

impl<'a> LintContext<'a> {
    pub fn new(semantic: &Rc<Semantic<'a>>, fix: bool) -> Self {
        let disable_directives =
            DisableDirectivesBuilder::new(semantic.source_text(), semantic.trivias()).build();
        Self {
            semantic: Rc::clone(semantic),
            diagnostics: RefCell::new(vec![]),
            disable_directives,
            fix,
            current_rule_name: "",
        }
    }

    #[must_use]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    #[must_use]
    pub fn source_text(&self) -> &'a str {
        self.semantic().source_text()
    }

    #[must_use]
    pub fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    pub fn with_rule_name(&mut self, name: &'static str) {
        self.current_rule_name = name;
    }

    /* Diagnostics */

    pub fn into_message(self) -> Vec<Message<'a>> {
        self.diagnostics.into_inner()
    }

    fn add_diagnostic(&self, message: Message<'a>) {
        if !self.disable_directives.contains(self.current_rule_name, message.start()) {
            self.diagnostics.borrow_mut().push(message);
        }
    }

    pub fn diagnostic<T: Into<Error>>(&self, diagnostic: T) {
        self.add_diagnostic(Message::new(diagnostic.into(), None));
    }

    pub fn diagnostic_with_fix<T, F>(&self, diagnostic: T, fix: F)
    where
        T: Into<Error>,
        F: FnOnce() -> Fix<'a>,
    {
        if self.fix {
            self.add_diagnostic(Message::new(diagnostic.into(), Some(fix())));
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
        self.nodes().ancestors(node)
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
    pub fn symbols(&self) -> &SymbolTable {
        self.semantic().symbols()
    }

    #[must_use]
    pub fn strict_mode(&self, node: &AstNode) -> bool {
        let scope = self.scope(node);
        node.get().strict_mode(scope)
    }

    /* Symbols */

    pub fn is_reference_to_global_variable(&self, ident: &IdentifierReference) -> bool {
        self.semantic().is_reference_to_global_variables(ident)
    }

    #[allow(clippy::unused_self)]
    pub fn printer(&self) -> Printer {
        Printer::new(0, PrinterOptions::default())
    }
}
