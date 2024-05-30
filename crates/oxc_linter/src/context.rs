use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_semantic::{AstNodes, JSDocFinder, ScopeTree, Semantic, SymbolTable};
use oxc_span::SourceType;

use crate::{
    disable_directives::{DisableDirectives, DisableDirectivesBuilder},
    fixer::{Fix, Message},
    javascript_globals::GLOBALS,
    AllowWarnDeny, OxlintConfig, OxlintEnv, OxlintGlobals, OxlintSettings,
};

#[derive(Clone)]
pub struct LintContext<'a> {
    semantic: Rc<Semantic<'a>>,

    diagnostics: RefCell<Vec<Message<'a>>>,

    disable_directives: Rc<DisableDirectives<'a>>,

    /// Whether or not to apply code fixes during linting.
    fix: bool,

    file_path: Rc<Path>,

    eslint_config: Arc<OxlintConfig>,

    // states
    current_rule_name: &'static str,

    severity: Severity,
}

impl<'a> LintContext<'a> {
    pub fn new(file_path: Box<Path>, semantic: &Rc<Semantic<'a>>) -> Self {
        let disable_directives =
            DisableDirectivesBuilder::new(semantic.source_text(), semantic.trivias()).build();
        Self {
            semantic: Rc::clone(semantic),
            diagnostics: RefCell::new(vec![]),
            disable_directives: Rc::new(disable_directives),
            fix: false,
            file_path: file_path.into(),
            eslint_config: Arc::new(OxlintConfig::default()),
            current_rule_name: "",
            severity: Severity::Warning,
        }
    }

    #[must_use]
    pub fn with_fix(mut self, fix: bool) -> Self {
        self.fix = fix;
        self
    }

    #[must_use]
    pub fn with_eslint_config(mut self, eslint_config: &Arc<OxlintConfig>) -> Self {
        self.eslint_config = Arc::clone(eslint_config);
        self
    }

    #[must_use]
    pub fn with_rule_name(mut self, name: &'static str) -> Self {
        self.current_rule_name = name;
        self
    }

    #[must_use]
    pub fn with_severity(mut self, severity: AllowWarnDeny) -> Self {
        self.severity = Severity::from(severity);
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

    pub fn settings(&self) -> &OxlintSettings {
        &self.eslint_config.settings
    }

    pub fn globals(&self) -> &OxlintGlobals {
        &self.eslint_config.globals
    }

    pub fn env(&self) -> &OxlintEnv {
        &self.eslint_config.env
    }

    pub fn env_contains_var(&self, var: &str) -> bool {
        for env in self.env().iter() {
            let env = GLOBALS.get(env).unwrap_or(&GLOBALS["builtin"]);
            if env.get(var).is_some() {
                return true;
            }
        }

        false
    }

    /* Diagnostics */

    pub fn into_message(self) -> Vec<Message<'a>> {
        self.diagnostics.borrow().iter().cloned().collect::<Vec<_>>()
    }

    fn add_diagnostic(&self, message: Message<'a>) {
        if !self.disable_directives.contains(self.current_rule_name, message.start()) {
            let mut message = message;
            if message.error.severity != self.severity {
                message.error = message.error.with_severity(self.severity);
            }
            self.diagnostics.borrow_mut().push(message);
        }
    }

    pub fn diagnostic(&self, diagnostic: OxcDiagnostic) {
        self.add_diagnostic(Message::new(diagnostic, None));
    }

    pub fn diagnostic_with_fix<F: FnOnce() -> Fix<'a>>(&self, diagnostic: OxcDiagnostic, fix: F) {
        if self.fix {
            self.add_diagnostic(Message::new(diagnostic, Some(fix())));
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
        Codegen::<false>::new("", "", CodegenOptions::default(), None)
    }

    /* JSDoc */
    pub fn jsdoc(&self) -> &JSDocFinder<'a> {
        self.semantic().jsdoc()
    }
}
