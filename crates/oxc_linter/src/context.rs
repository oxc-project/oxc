use std::{cell::RefCell, rc::Rc};

use indextree::{Ancestors, NodeId};
use oxc_ast::AstKind;
use oxc_diagnostics::Error;
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_semantic::{AstNodes, JSDocComment, ScopeTree, Semantic, SemanticNode, SymbolTable};
use oxc_span::SourceType;

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

    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    pub fn source_text(&self) -> &'a str {
        self.semantic().source_text()
    }

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

    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic().nodes()
    }

    pub fn kind(&self, node_id: NodeId) -> AstKind<'a> {
        self.nodes().kind(node_id)
    }

    pub fn parent_kind(&self, node: &AstNode<'a>) -> AstKind<'a> {
        self.nodes().parent_kind(node)
    }

    pub fn parent_node(&self, node: &AstNode<'a>) -> Option<&AstNode<'a>> {
        node.parent().and_then(|node_id| self.nodes().get(node_id))
    }

    pub fn ancestors(&self, node: &AstNode<'a>) -> Ancestors<'_, SemanticNode<'a>> {
        self.nodes().ancestors(node)
    }

    /* Scopes */

    pub fn scopes(&self) -> &ScopeTree {
        self.semantic().scopes()
    }

    pub fn symbols(&self) -> &SymbolTable {
        self.semantic().symbols()
    }

    #[allow(clippy::unused_self)]
    pub fn formatter(&self) -> Formatter {
        Formatter::new(0, FormatterOptions::default())
    }

    /* JSDoc */
    pub fn jsdoc(&self, node: &AstNode<'a>) -> Option<JSDocComment<'a>> {
        self.semantic().jsdoc().get_by_node(node)
    }
}
