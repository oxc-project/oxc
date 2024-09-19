#![allow(rustdoc::private_intra_doc_links)] // useful for intellisense
mod host;

use std::{path::Path, rc::Rc};

use oxc_cfg::ControlFlowGraph;
use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_semantic::{AstNodes, JSDocFinder, ScopeTree, Semantic, SymbolTable};
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::module_record::ModuleRecord;

#[cfg(debug_assertions)]
use crate::rule::RuleFixMeta;
use crate::{
    disable_directives::DisableDirectives,
    fixer::{FixKind, Message, RuleFix, RuleFixer},
    javascript_globals::GLOBALS,
    AllowWarnDeny, FrameworkFlags, OxlintEnv, OxlintGlobals, OxlintSettings,
};

pub(crate) use host::ContextHost;

#[derive(Clone)]
#[must_use]
pub struct LintContext<'a> {
    /// Shared context independent of the rule being linted.
    parent: Rc<ContextHost<'a>>,

    // states
    current_plugin_name: &'static str,
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
}

impl<'a> LintContext<'a> {
    const WEBSITE_BASE_URL: &'static str = "https://oxc.rs/docs/guide/usage/linter/rules";

    pub fn with_plugin_name(mut self, plugin: &'static str) -> Self {
        self.current_plugin_name = plugin;
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

    /// Update the severity of diagnostics reported by the rule this context is
    /// associated with.
    #[inline]
    pub fn with_severity(mut self, severity: AllowWarnDeny) -> Self {
        self.severity = Severity::from(severity);
        self
    }

    /// Get information such as the control flow graph, bound symbols, AST, etc.
    /// for the file being linted.
    ///
    /// Refer to [`Semantic`]'s documentation for more information.
    #[inline]
    pub fn semantic(&self) -> &Rc<Semantic<'a>> {
        &self.parent.semantic
    }

    #[inline]
    pub fn cfg(&self) -> &ControlFlowGraph {
        // SAFETY: `LintContext::new` is the only way to construct a `LintContext` and we always
        // assert the existence of control flow so it should always be `Some`.
        unsafe { self.semantic().cfg().unwrap_unchecked() }
    }

    #[inline]
    pub fn disable_directives(&self) -> &DisableDirectives<'a> {
        &self.parent.disable_directives
    }

    /// Source code of the file being linted.
    #[inline]
    pub fn source_text(&self) -> &'a str {
        self.semantic().source_text()
    }

    /// Get a snippet of source text covered by the given [`Span`]. For details,
    /// see [`Span::source_text`].
    pub fn source_range(&self, span: Span) -> &'a str {
        span.source_text(self.semantic().source_text())
    }

    /// [`SourceType`] of the file currently being linted.
    #[inline]
    pub fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    /// Path to the file currently being linted.
    #[inline]
    pub fn file_path(&self) -> &Path {
        &self.parent.file_path
    }

    /// Plugin settings
    #[inline]
    pub fn settings(&self) -> &OxlintSettings {
        &self.parent.config.settings
    }

    #[inline]
    pub fn globals(&self) -> &OxlintGlobals {
        &self.parent.config.globals
    }

    /// Runtime environments turned on/off by the user.
    ///
    /// Examples of environments are `builtin`, `browser`, `node`, etc.
    #[inline]
    pub fn env(&self) -> &OxlintEnv {
        &self.parent.config.env
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

    fn add_diagnostic(&self, mut message: Message<'a>) {
        if self.parent.disable_directives.contains(self.current_rule_name, message.span()) {
            return;
        }
        message.error = message
            .error
            .with_error_code(self.current_plugin_prefix, self.current_rule_name)
            .with_url(format!(
                "{}/{}/{}.html",
                Self::WEBSITE_BASE_URL,
                self.current_plugin_name,
                self.current_rule_name
            ));
        if message.error.severity != self.severity {
            message.error = message.error.with_severity(self.severity);
        }

        self.parent.push_diagnostic(message);
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
        if self.parent.fix.can_apply(rule_fix.kind()) {
            let fix = rule_fix.into_fix(self.source_text());
            self.add_diagnostic(Message::new(diagnostic, Some(fix)));
        } else {
            self.diagnostic(diagnostic);
        }
    }

    pub fn frameworks(&self) -> FrameworkFlags {
        self.parent.frameworks
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
    "node" => "eslint-plugin-node",
};
