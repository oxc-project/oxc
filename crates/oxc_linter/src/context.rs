use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::Error;
use oxc_semantic::{AstNodes, JSDocFinder, ScopeTree, Semantic, SymbolTable};
use oxc_span::SourceType;

use crate::{
    disable_directives::{DisableDirectives, DisableDirectivesBuilder},
    fixer::{Fix, Message},
    javascript_globals::GLOBALS,
    ESLintEnv, ESLintSettings,
};

pub struct LintContext<'a> {
    semantic: Rc<Semantic<'a>>,

    diagnostics: RefCell<Vec<Message<'a>>>,

    disable_directives: DisableDirectives<'a>,

    /// Whether or not to apply code fixes during linting.
    fix: bool,

    current_rule_name: &'static str,

    file_path: Box<Path>,

    settings: Arc<ESLintSettings>,

    env: Arc<ESLintEnv>,
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
            settings: Arc::new(ESLintSettings::default()),
            env: Arc::new(ESLintEnv::default()),
        }
    }

    #[must_use]
    pub fn with_fix(mut self, fix: bool) -> Self {
        self.fix = fix;
        self
    }

    #[must_use]
    pub fn with_settings(mut self, settings: &Arc<ESLintSettings>) -> Self {
        self.settings = Arc::clone(settings);
        self
    }

    #[must_use]
    pub fn with_env(mut self, env: &Arc<ESLintEnv>) -> Self {
        self.env = Arc::clone(env);
        self
    }

    pub fn semantic(&self) -> &Rc<Semantic<'a>> {
        &self.semantic
    }

    pub fn disable_directives(&self) -> &DisableDirectives<'a> {
        &self.disable_directives
    }

    pub fn settings(&self) -> &ESLintSettings {
        &self.settings
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

    pub fn envs(&self) -> &ESLintEnv {
        &self.env
    }

    pub fn env_contains_var(&self, var: &str) -> bool {
        for env in self.env.iter() {
            let env = GLOBALS.get(env).unwrap_or(&GLOBALS["builtin"]);
            if env.get(var).is_some() {
                return true;
            }
        }

        false
    }

    #[inline]
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
    pub fn codegen(&self) -> Codegen<false> {
        Codegen::<false>::new(0, CodegenOptions::default())
    }

    /* JSDoc */
    pub fn jsdoc(&self) -> &JSDocFinder<'a> {
        self.semantic().jsdoc()
    }
}
