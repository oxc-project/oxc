use std::{cell::RefCell, path::Path, rc::Rc};

use oxc_diagnostics::Error;
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_semantic::{AstNodes, JSDocComment, ScopeTree, Semantic, SymbolTable};
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

    file_path: Box<Path>,
}

impl<'a> LintContext<'a> {
    pub fn new(file_path: Box<Path>, semantic: &Rc<Semantic<'a>>) -> Self {
        let disable_directives =
            DisableDirectivesBuilder::new(semantic.source_text(), semantic.trivias()).build();
        Self {
            semantic: Rc::clone(semantic),
            diagnostics: RefCell::new(vec![]),
            disable_directives,
            fix: false,
            current_rule_name: "",
            file_path,
        }
    }

    #[must_use]
    pub fn with_fix(mut self, fix: bool) -> Self {
        self.fix = fix;
        self
    }

    pub fn semantic(&self) -> &Rc<Semantic<'a>> {
        &self.semantic
    }

    pub fn disable_directives(&self) -> &DisableDirectives<'a> {
        &self.disable_directives
    }

    pub fn source_text(&self) -> &'a str {
        self.semantic().source_text()
    }

    pub fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
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

    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic().nodes()
    }

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
