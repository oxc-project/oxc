use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use oxc_cfg::ControlFlowGraph;
use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_semantic::{AstNodes, JSDocFinder, ScopeTree, Semantic, SymbolTable};
use oxc_span::{SourceType, Span};
use oxc_syntax::module_record::ModuleRecord;

use crate::{
    disable_directives::{DisableDirectives, DisableDirectivesBuilder},
    fixer::{Fix, Message, RuleFixer},
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

#[derive(Clone)]
pub struct CFGLintContext<'a>(LintContext<'a>);

impl<'a> CFGLintContext<'a> {
    /// # Panics
    /// If rule doesn't have `#[use_cfg]` in it's declaration it would panic.
    pub fn cfg(&self) -> &ControlFlowGraph {
        if let Some(cfg) = self.semantic().cfg() {
            cfg
        } else {
            unreachable!("for creating a control flow aware rule you have to add `#[use_cfg]` attribute to its `declare_oxc_lint` declaration");
        }
    }
}

impl<'a> From<LintContext<'a>> for CFGLintContext<'a> {
    fn from(ctx: LintContext<'a>) -> Self {
        Self(ctx)
    }
}

impl<'a, 'b> From<&'b CFGLintContext<'a>> for &'b LintContext<'a> {
    fn from(ctx: &'b CFGLintContext<'a>) -> Self {
        &ctx.0
    }
}

impl<'a> LintCtx<'a> for CFGLintContext<'a> {
    fn new(file_path: Box<Path>, semantic: Rc<Semantic<'a>>) -> Self {
        Self(LintContext::new(file_path, semantic))
    }

    fn with_fix(mut self, fix: bool) -> Self {
        self.0 = self.0.with_fix(fix);
        self
    }

    fn with_eslint_config(mut self, eslint_config: &Arc<OxlintConfig>) -> Self {
        self.0 = self.0.with_eslint_config(eslint_config);
        self
    }

    fn with_rule_name(mut self, name: &'static str) -> Self {
        self.0 = self.0.with_rule_name(name);
        self
    }

    fn with_severity(mut self, severity: AllowWarnDeny) -> Self {
        self.0 = self.0.with_severity(severity);
        self
    }

    fn semantic(&self) -> &Rc<Semantic<'a>> {
        self.0.semantic()
    }

    fn disable_directives(&self) -> &DisableDirectives<'a> {
        self.0.disable_directives()
    }

    fn source_text(&self) -> &'a str {
        self.0.source_text()
    }

    fn source_range(&self, span: Span) -> &'a str {
        self.0.source_range(span)
    }

    fn source_type(&self) -> &SourceType {
        self.0.source_type()
    }

    fn file_path(&self) -> &Path {
        self.0.file_path()
    }

    fn settings(&self) -> &OxlintSettings {
        self.0.settings()
    }

    fn globals(&self) -> &OxlintGlobals {
        self.0.globals()
    }

    fn env(&self) -> &OxlintEnv {
        self.0.env()
    }

    fn env_contains_var(&self, var: &str) -> bool {
        self.0.env_contains_var(var)
    }

    /* Diagnostics */
    fn into_message(self) -> Vec<Message<'a>> {
        self.0.into_message()
    }

    fn add_diagnostic(&self, message: Message<'a>) {
        self.0.add_diagnostic(message);
    }

    fn diagnostic(&self, diagnostic: OxcDiagnostic) {
        self.0.diagnostic(diagnostic);
    }

    fn diagnostic_with_fix<F: FnOnce(RuleFixer<'_, 'a>) -> Fix<'a>>(
        &self,
        diagnostic: OxcDiagnostic,
        fix: F,
    ) {
        self.0.diagnostic_with_fix(diagnostic, fix);
    }

    fn nodes(&self) -> &AstNodes<'a> {
        self.0.nodes()
    }

    fn scopes(&self) -> &ScopeTree {
        self.0.scopes()
    }

    fn symbols(&self) -> &SymbolTable {
        self.0.symbols()
    }

    fn module_record(&self) -> &ModuleRecord {
        self.0.module_record()
    }

    /* JSDoc */
    fn jsdoc(&self) -> &JSDocFinder<'a> {
        self.0.jsdoc()
    }
}

pub trait LintCtx<'a>: Clone {
    fn new(file_path: Box<Path>, semantic: Rc<Semantic<'a>>) -> Self;
    #[must_use]
    fn with_fix(self, fix: bool) -> Self;
    #[must_use]
    fn with_eslint_config(self, eslint_config: &Arc<OxlintConfig>) -> Self;
    #[must_use]
    fn with_rule_name(self, name: &'static str) -> Self;
    #[must_use]
    fn with_severity(self, severity: AllowWarnDeny) -> Self;
    fn semantic(&self) -> &Rc<Semantic<'a>>;
    fn disable_directives(&self) -> &DisableDirectives<'a>;
    fn source_text(&self) -> &'a str;
    fn source_range(&self, span: Span) -> &'a str;
    fn source_type(&self) -> &SourceType;
    fn file_path(&self) -> &Path;
    fn settings(&self) -> &OxlintSettings;
    fn globals(&self) -> &OxlintGlobals;
    fn env(&self) -> &OxlintEnv;
    fn env_contains_var(&self, var: &str) -> bool;

    /* Diagnostics */
    fn into_message(self) -> Vec<Message<'a>>;
    fn add_diagnostic(&self, message: Message<'a>);

    fn diagnostic(&self, diagnostic: OxcDiagnostic);
    fn diagnostic_with_fix<F: FnOnce(RuleFixer<'_, 'a>) -> Fix<'a>>(
        &self,
        diagnostic: OxcDiagnostic,
        fix: F,
    );
    fn nodes(&self) -> &AstNodes<'a>;
    fn scopes(&self) -> &ScopeTree;
    fn symbols(&self) -> &SymbolTable;
    fn module_record(&self) -> &ModuleRecord;

    /* JSDoc */
    fn jsdoc(&self) -> &JSDocFinder<'a>;
}

impl<'a> LintCtx<'a> for LintContext<'a> {
    fn new(file_path: Box<Path>, semantic: Rc<Semantic<'a>>) -> Self {
        let disable_directives =
            DisableDirectivesBuilder::new(semantic.source_text(), semantic.trivias().clone())
                .build();
        Self {
            semantic,
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
    fn with_fix(mut self, fix: bool) -> Self {
        self.fix = fix;
        self
    }

    #[must_use]
    fn with_eslint_config(mut self, eslint_config: &Arc<OxlintConfig>) -> Self {
        self.eslint_config = Arc::clone(eslint_config);
        self
    }

    #[must_use]
    fn with_rule_name(mut self, name: &'static str) -> Self {
        self.current_rule_name = name;
        self
    }

    #[must_use]
    fn with_severity(mut self, severity: AllowWarnDeny) -> Self {
        self.severity = Severity::from(severity);
        self
    }

    fn semantic(&self) -> &Rc<Semantic<'a>> {
        &self.semantic
    }

    // TODO ?
    fn disable_directives(&self) -> &DisableDirectives<'a> {
        &self.disable_directives
    }

    // TODO ?
    /// Source code of the file being linted.
    fn source_text(&self) -> &'a str {
        self.semantic().source_text()
    }

    /// Get a snippet of source text covered by the given [`Span`]. For details,
    /// see [`Span::source_text`].
    fn source_range(&self, span: Span) -> &'a str {
        span.source_text(self.semantic().source_text())
    }

    fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    fn file_path(&self) -> &Path {
        &self.file_path
    }

    fn settings(&self) -> &OxlintSettings {
        &self.eslint_config.settings
    }

    fn globals(&self) -> &OxlintGlobals {
        &self.eslint_config.globals
    }

    fn env(&self) -> &OxlintEnv {
        &self.eslint_config.env
    }

    fn env_contains_var(&self, var: &str) -> bool {
        for env in self.env().iter() {
            let env = GLOBALS.get(env).unwrap_or(&GLOBALS["builtin"]);
            if env.get(var).is_some() {
                return true;
            }
        }

        false
    }

    /* Diagnostics */

    fn into_message(self) -> Vec<Message<'a>> {
        self.diagnostics.borrow().iter().cloned().collect::<Vec<_>>()
    }

    /// TODO ?
    fn add_diagnostic(&self, message: Message<'a>) {
        if !self.disable_directives.contains(self.current_rule_name, message.start()) {
            let mut message = message;
            if message.error.severity != self.severity {
                message.error = message.error.with_severity(self.severity);
            }
            self.diagnostics.borrow_mut().push(message);
        }
    }

    /// Report a lint rule violation.
    ///
    /// Use [`LintContext::diagnostic_with_fix`] to provide an automatic fix.
    fn diagnostic(&self, diagnostic: OxcDiagnostic) {
        self.add_diagnostic(Message::new(diagnostic, None));
    }

    /// Report a lint rule violation and provide an automatic fix.
    fn diagnostic_with_fix<F: FnOnce(RuleFixer<'_, 'a>) -> Fix<'a>>(
        &self,
        diagnostic: OxcDiagnostic,
        fix: F,
    ) {
        if self.fix {
            let fixer = RuleFixer::new(self);
            self.add_diagnostic(Message::new(diagnostic, Some(fix(fixer))));
        } else {
            self.diagnostic(diagnostic);
        }
    }

    fn nodes(&self) -> &AstNodes<'a> {
        self.semantic().nodes()
    }

    fn scopes(&self) -> &ScopeTree {
        self.semantic().scopes()
    }

    fn symbols(&self) -> &SymbolTable {
        self.semantic().symbols()
    }

    fn module_record(&self) -> &ModuleRecord {
        self.semantic().module_record()
    }

    /* JSDoc */
    fn jsdoc(&self) -> &JSDocFinder<'a> {
        self.semantic().jsdoc()
    }
}
