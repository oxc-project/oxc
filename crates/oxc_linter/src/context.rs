#![allow(rustdoc::private_intra_doc_links)] // useful for intellisense
use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use oxc_cfg::ControlFlowGraph;
use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_semantic::{AstNodes, JSDocFinder, ScopeTree, Semantic, SymbolTable};
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::module_record::ModuleRecord;

#[cfg(debug_assertions)]
use crate::rule::RuleFixMeta;
use crate::{
    config::OxlintRules,
    disable_directives::{DisableDirectives, DisableDirectivesBuilder},
    fixer::{FixKind, Message, RuleFix, RuleFixer},
    javascript_globals::GLOBALS,
    AllowWarnDeny, FrameworkFlags, OxlintConfig, OxlintEnv, OxlintGlobals, OxlintSettings,
};

#[derive(Clone)]
#[must_use]
pub struct LintContext<'a> {
    semantic: Rc<Semantic<'a>>,

    /// Diagnostics reported by the linter.
    ///
    /// Contains diagnostics for all rules across all files.
    diagnostics: RefCell<Vec<Message<'a>>>,

    disable_directives: Rc<DisableDirectives<'a>>,

    /// Whether or not to apply code fixes during linting. Defaults to
    /// [`FixKind::None`] (no fixing).
    ///
    /// Set via the `--fix`, `--fix-suggestions`, and `--fix-dangerously` CLI
    /// flags.
    fix: FixKind,

    file_path: Rc<Path>,

    eslint_config: Arc<OxlintConfig>,

    // states
    current_plugin_prefix: &'static str,
    current_rule_name: &'static str,
    #[cfg(debug_assertions)]
    current_rule_fix_capabilities: RuleFixMeta,

    /// Current rule severity. Allows for user severity overrides, e.g.
    /// ```json
    /// // .oxlintrc.json
    /// {
    ///   "rules": {
    ///     "no-debugger": "error"
    ///   }
    /// }
    /// ```
    severity: Severity,
    frameworks: FrameworkFlags,
}

impl<'a> LintContext<'a> {
    /// # Panics
    /// If `semantic.cfg()` is `None`.
    pub fn new(file_path: Box<Path>, semantic: Rc<Semantic<'a>>) -> Self {
        const DIAGNOSTICS_INITIAL_CAPACITY: usize = 128;

        // We should always check for `semantic.cfg()` being `Some` since we depend on it and it is
        // unwrapped without any runtime checks after construction.
        assert!(
            semantic.cfg().is_some(),
            "`LintContext` depends on `Semantic::cfg`, Build your semantic with cfg enabled(`SemanticBuilder::with_cfg`)."
        );
        let disable_directives =
            DisableDirectivesBuilder::new(semantic.source_text(), semantic.trivias().clone())
                .build();
        Self {
            semantic,
            diagnostics: RefCell::new(Vec::with_capacity(DIAGNOSTICS_INITIAL_CAPACITY)),
            disable_directives: Rc::new(disable_directives),
            fix: FixKind::None,
            file_path: file_path.into(),
            eslint_config: Arc::new(OxlintConfig::default()),
            current_plugin_prefix: "eslint",
            current_rule_name: "",
            #[cfg(debug_assertions)]
            current_rule_fix_capabilities: RuleFixMeta::None,
            severity: Severity::Warning,
            frameworks: FrameworkFlags::empty(),
        }
    }

    /// Enable/disable automatic code fixes.
    pub fn with_fix(mut self, fix: FixKind) -> Self {
        self.fix = fix;
        self
    }

    pub fn with_eslint_config(mut self, eslint_config: &Arc<OxlintConfig>) -> Self {
        self.eslint_config = Arc::clone(eslint_config);
        self
    }

    pub fn with_plugin_name(mut self, plugin: &'static str) -> Self {
        self.current_plugin_prefix = plugin_name_to_prefix(plugin);
        self
    }

    pub fn with_rule_name(mut self, name: &'static str) -> Self {
        self.current_rule_name = name;
        self
    }

    #[cfg(debug_assertions)]
    pub fn with_rule_fix_capabilities(mut self, capabilities: RuleFixMeta) -> Self {
        self.current_rule_fix_capabilities = capabilities;
        self
    }

    pub fn with_severity(mut self, severity: AllowWarnDeny) -> Self {
        self.severity = Severity::from(severity);
        self
    }

    /// Set [`FrameworkFlags`], overwriting any existing flags.
    pub fn with_frameworks(mut self, frameworks: FrameworkFlags) -> Self {
        self.frameworks = frameworks;
        self
    }

    /// Add additional [`FrameworkFlags`]
    pub fn and_frameworks(mut self, frameworks: FrameworkFlags) -> Self {
        self.frameworks |= frameworks;
        self
    }

    pub fn semantic(&self) -> &Rc<Semantic<'a>> {
        &self.semantic
    }

    pub fn cfg(&self) -> &ControlFlowGraph {
        // SAFETY: `LintContext::new` is the only way to construct a `LintContext` and we always
        // assert the existence of control flow so it should always be `Some`.
        unsafe { self.semantic().cfg().unwrap_unchecked() }
    }

    pub fn disable_directives(&self) -> &DisableDirectives<'a> {
        &self.disable_directives
    }

    /// Source code of the file being linted.
    pub fn source_text(&self) -> &'a str {
        self.semantic().source_text()
    }

    /// Get a snippet of source text covered by the given [`Span`]. For details,
    /// see [`Span::source_text`].
    pub fn source_range(&self, span: Span) -> &'a str {
        span.source_text(self.semantic().source_text())
    }

    /// [`SourceType`] of the file currently being linted.
    pub fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    /// Path to the file currently being linted.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Plugin settings
    pub fn settings(&self) -> &OxlintSettings {
        &self.eslint_config.settings
    }

    pub fn globals(&self) -> &OxlintGlobals {
        &self.eslint_config.globals
    }

    /// Runtime environments turned on/off by the user.
    ///
    /// Examples of environments are `builtin`, `browser`, `node`, etc.
    pub fn env(&self) -> &OxlintEnv {
        &self.eslint_config.env
    }

    pub fn rules(&self) -> &OxlintRules {
        &self.eslint_config.rules
    }

    pub fn env_contains_var(&self, var: &str) -> bool {
        if GLOBALS["builtin"].contains_key(var) {
            return true;
        }
        for env in self.env().iter() {
            if let Some(env) = GLOBALS.get(env) {
                if env.contains_key(var) {
                    return true;
                }
            }
        }
        false
    }

    /* Diagnostics */

    pub fn into_message(self) -> Vec<Message<'a>> {
        self.diagnostics.borrow().iter().cloned().collect::<Vec<_>>()
    }

    fn add_diagnostic(&self, message: Message<'a>) {
        if !self.disable_directives.contains(self.current_rule_name, message.span()) {
            let mut message = message;
            message.error =
                message.error.with_error_code(self.current_plugin_prefix, self.current_rule_name);
            if message.error.severity != self.severity {
                message.error = message.error.with_severity(self.severity);
            }
            self.diagnostics.borrow_mut().push(message);
        }
    }

    /// Report a lint rule violation.
    ///
    /// Use [`LintContext::diagnostic_with_fix`] to provide an automatic fix.
    #[inline]
    pub fn diagnostic(&self, diagnostic: OxcDiagnostic) {
        self.add_diagnostic(Message::new(diagnostic, None));
    }

    /// Report a lint rule violation and provide an automatic fix.
    ///
    /// The second argument is a [closure] that takes a [`RuleFixer`] and
    /// returns something that can turn into a `CompositeFix`.
    ///
    /// Fixes created this way should not create parse errors or change the
    /// semantics of the linted code. If your fix may change the code's
    /// semantics, use [`LintContext::diagnostic_with_suggestion`] instead. If
    /// your fix has the potential to create parse errors, use
    /// [`LintContext::diagnostic_with_dangerous_fix`].
    ///
    /// [closure]: <https://doc.rust-lang.org/book/ch13-01-closures.html>
    #[inline]
    pub fn diagnostic_with_fix<C, F>(&self, diagnostic: OxcDiagnostic, fix: F)
    where
        C: Into<RuleFix<'a>>,
        F: FnOnce(RuleFixer<'_, 'a>) -> C,
    {
        self.diagnostic_with_fix_of_kind(diagnostic, FixKind::SafeFix, fix);
    }

    /// Report a lint rule violation and provide a suggestion for fixing it.
    ///
    /// The second argument is a [closure] that takes a [`RuleFixer`] and
    /// returns something that can turn into a `CompositeFix`.
    ///
    /// Fixes created this way should not create parse errors, but have the
    /// potential to change the code's semantics. If your fix is completely safe
    /// and definitely does not change semantics, use [`LintContext::diagnostic_with_fix`].
    /// If your fix has the potential to create parse errors, use
    /// [`LintContext::diagnostic_with_dangerous_fix`].
    ///
    /// [closure]: <https://doc.rust-lang.org/book/ch13-01-closures.html>
    #[inline]
    pub fn diagnostic_with_suggestion<C, F>(&self, diagnostic: OxcDiagnostic, fix: F)
    where
        C: Into<RuleFix<'a>>,
        F: FnOnce(RuleFixer<'_, 'a>) -> C,
    {
        self.diagnostic_with_fix_of_kind(diagnostic, FixKind::Suggestion, fix);
    }

    /// Report a lint rule violation and provide a potentially dangerous
    /// automatic fix for it.
    ///
    /// The second argument is a [closure] that takes a [`RuleFixer`] and
    /// returns something that can turn into a `CompositeFix`.
    ///
    /// Dangerous fixes should be avoided and are not applied by default with
    /// `--fix`. Use this method if:
    /// - Your fix is experimental and you want to test it out in the wild
    ///   before marking it as safe.
    /// - Your fix is extremely aggressive and risky, but you want to provide
    ///   it as an option to users.
    ///
    /// When possible, prefer [`LintContext::diagnostic_with_fix`]. If the only
    /// risk your fix poses is minor(ish) changes to code semantics, use
    /// [`LintContext::diagnostic_with_suggestion`] instead.
    ///
    /// [closure]: <https://doc.rust-lang.org/book/ch13-01-closures.html>
    ///
    #[inline]
    pub fn diagnostic_with_dangerous_fix<C, F>(&self, diagnostic: OxcDiagnostic, fix: F)
    where
        C: Into<RuleFix<'a>>,
        F: FnOnce(RuleFixer<'_, 'a>) -> C,
    {
        self.diagnostic_with_fix_of_kind(diagnostic, FixKind::DangerousFix, fix);
    }

    #[allow(clippy::missing_panics_doc)] // only panics in debug mode
    pub fn diagnostic_with_fix_of_kind<C, F>(
        &self,
        diagnostic: OxcDiagnostic,
        fix_kind: FixKind,
        fix: F,
    ) where
        C: Into<RuleFix<'a>>,
        F: FnOnce(RuleFixer<'_, 'a>) -> C,
    {
        let fixer = RuleFixer::new(fix_kind, self);
        let rule_fix: RuleFix<'a> = fix(fixer).into();
        #[cfg(debug_assertions)]
        {
            assert!(
                self.current_rule_fix_capabilities.supports_fix(fix_kind),
                "Rule `{}` does not support safe fixes. Did you forget to update fix capabilities in declare_oxc_lint?.\n\tSupported fix kinds: {:?}\n\tAttempted fix kind: {:?}",
                self.current_rule_name,
                FixKind::from(self.current_rule_fix_capabilities),
                rule_fix.kind()
            );
        }
        let diagnostic = match (rule_fix.message(), &diagnostic.help) {
            (Some(message), None) => diagnostic.with_help(message.to_owned()),
            _ => diagnostic,
        };
        if self.fix.can_apply(rule_fix.kind()) {
            let fix = rule_fix.into_fix(self.source_text());
            self.add_diagnostic(Message::new(diagnostic, Some(fix)));
        } else {
            self.diagnostic(diagnostic);
        }
    }

    pub fn frameworks(&self) -> FrameworkFlags {
        self.frameworks
    }

    /// AST nodes
    ///
    /// Shorthand for `self.semantic().nodes()`.
    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic().nodes()
    }

    /// Scope tree
    ///
    /// Shorthand for `ctx.semantic().scopes()`.
    pub fn scopes(&self) -> &ScopeTree {
        self.semantic().scopes()
    }

    /// Symbol table
    ///
    /// Shorthand for `ctx.semantic().symbols()`.
    pub fn symbols(&self) -> &SymbolTable {
        self.semantic().symbols()
    }

    /// Imported modules and exported symbols
    ///
    /// Shorthand for `ctx.semantic().module_record()`.
    pub fn module_record(&self) -> &ModuleRecord {
        self.semantic().module_record()
    }

    /// JSDoc comments
    ///
    /// Shorthand for `ctx.semantic().jsdoc()`.
    pub fn jsdoc(&self) -> &JSDocFinder<'a> {
        self.semantic().jsdoc()
    }

    // #[inline]
    // fn plugin_name_to_prefix(&self, plugin_name: &'static str) -> &'static str {
    //     let plugin_name = if self. plugin_name == "jest" && self.frameworks.contains(FrameworkFlags::Vitest) {
    //         "vitest"
    //     } else {
    //         plugin_name
    //     };
    //     PLUGIN_PREFIXES.get(plugin_name).copied().unwrap_or(plugin_name)
    // }
}

#[inline]
fn plugin_name_to_prefix(plugin_name: &'static str) -> &'static str {
    PLUGIN_PREFIXES.get(plugin_name).copied().unwrap_or(plugin_name)
}

const PLUGIN_PREFIXES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "import" => "eslint-plugin-import",
    "jest" => "eslint-plugin-jest",
    "jsdoc" => "eslint-plugin-jsdoc",
    "jsx_a11y" => "eslint-plugin-jsx-a11y",
    "nextjs" => "eslint-plugin-next",
    "promise" => "eslint-plugin-promise",
    "react_perf" => "eslint-plugin-react-perf",
    "react" => "eslint-plugin-react",
    "tree_shaking" => "eslint-plugin-tree-shaking",
    "typescript" => "typescript-eslint",
    "unicorn" => "eslint-plugin-unicorn",
    "vitest" => "eslint-plugin-vitest",
};
