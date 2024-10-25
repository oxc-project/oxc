use oxc_semantic::Semantic;
use oxc_span::SourceType;
use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use crate::{
    config::LintConfig,
    disable_directives::{DisableDirectives, DisableDirectivesBuilder},
    fixer::{FixKind, Message},
    frameworks,
    options::{LintOptions, LintPlugins},
    utils, FrameworkFlags, RuleWithSeverity,
};

use super::{plugin_name_to_prefix, LintContext};

/// Stores shared information about a file being linted.
///
/// When linting a file, there are a number of shared resources that are
/// independent of the rule being linted. [`ContextHost`] stores this context
/// subset. When a lint rule is run, a [`LintContext`] with rule-specific
/// information is spawned using [`ContextHost::spawn`].
///
/// ## API Encapsulation
///
/// In most cases, lint rules should be interacting with a [`LintContext`].  The
/// only current exception to this is
/// [should_run](`crate::rule::Rule::should_run`). Just before a file is linted,
/// rules are filtered out based on that method. Creating a [`LintContext`] for
/// rules that would never get run is a waste of resources, so they must use
/// [`ContextHost`].
///
/// ## References
/// - [Flyweight Pattern](https://en.wikipedia.org/wiki/Flyweight_pattern)
#[must_use]
#[non_exhaustive]
pub(crate) struct ContextHost<'a> {
    /// Shared semantic information about the file being linted, which includes scopes, symbols
    /// and AST nodes. See [`Semantic`].
    pub(super) semantic: Rc<Semantic<'a>>,
    /// Information about specific rules that should be disabled or enabled, via comment directives like
    /// `eslint-disable` or `eslint-disable-next-line`.
    pub(super) disable_directives: DisableDirectives<'a>,
    /// Diagnostics reported by the linter.
    ///
    /// Contains diagnostics for all rules across a single file.
    diagnostics: RefCell<Vec<Message<'a>>>,
    /// Whether or not to apply code fixes during linting. Defaults to
    /// [`FixKind::None`] (no fixing).
    ///
    /// Set via the `--fix`, `--fix-suggestions`, and `--fix-dangerously` CLI
    /// flags.
    pub(super) fix: FixKind,
    /// Path to the file being linted.
    pub(super) file_path: Box<Path>,
    /// Global linter configuration, such as globals to include and the target
    /// environments, and other settings.
    pub(super) config: Arc<LintConfig>,
    /// Front-end frameworks that might be in use in the target file.
    pub(super) frameworks: FrameworkFlags,
    /// A list of all available linter plugins.
    pub(super) plugins: LintPlugins,
}

impl<'a> ContextHost<'a> {
    /// # Panics
    /// If `semantic.cfg()` is `None`.
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        semantic: Rc<Semantic<'a>>,
        options: LintOptions,
    ) -> Self {
        const DIAGNOSTICS_INITIAL_CAPACITY: usize = 512;

        // We should always check for `semantic.cfg()` being `Some` since we depend on it and it is
        // unwrapped without any runtime checks after construction.
        assert!(
            semantic.cfg().is_some(),
            "`LintContext` depends on `Semantic::cfg`, Build your semantic with cfg enabled(`SemanticBuilder::with_cfg`)."
        );

        let disable_directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        let file_path = file_path.as_ref().to_path_buf().into_boxed_path();

        Self {
            semantic,
            disable_directives,
            diagnostics: RefCell::new(Vec::with_capacity(DIAGNOSTICS_INITIAL_CAPACITY)),
            fix: options.fix,
            file_path,
            config: Arc::new(LintConfig::default()),
            frameworks: options.framework_hints,
            plugins: options.plugins,
        }
        .sniff_for_frameworks()
    }

    /// Set the linter configuration for this context.
    #[inline]
    pub fn with_config(mut self, config: &Arc<LintConfig>) -> Self {
        self.config = Arc::clone(config);
        self
    }

    /// Shared reference to the [`Semantic`] analysis of the file.
    #[inline]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    /// Path to the file being linted.
    ///
    /// When created from a [`LintService`](`crate::service::LintService`), this
    /// will be an absolute path.
    #[inline]
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// The source type of the file being linted, e.g. JavaScript, TypeScript,
    /// CJS, ESM, etc.
    #[inline]
    pub fn source_type(&self) -> &SourceType {
        self.semantic.source_type()
    }

    /// Add a diagnostic message to the end of the list of diagnostics. Can be used
    /// by any rule to report issues.
    #[inline]
    pub(super) fn push_diagnostic(&self, diagnostic: Message<'a>) {
        self.diagnostics.borrow_mut().push(diagnostic);
    }

    /// Take ownership of all diagnostics collected during linting.
    pub fn take_diagnostics(&self) -> Vec<Message<'a>> {
        // NOTE: diagnostics are only ever borrowed here and in push_diagnostic.
        // The latter drops the reference as soon as the function returns, so
        // this should never panic.
        let mut messages = self.diagnostics.borrow_mut();
        std::mem::take(&mut *messages)
    }

    /// Creates a new [`LintContext`] for a specific rule.
    pub fn spawn(self: Rc<Self>, rule: &RuleWithSeverity) -> LintContext<'a> {
        let rule_name = rule.name();
        let plugin_name = self.map_jest_rule_to_vitest(rule);

        LintContext {
            parent: self,
            current_rule_name: rule_name,
            current_plugin_name: plugin_name,
            current_plugin_prefix: plugin_name_to_prefix(plugin_name),
            #[cfg(debug_assertions)]
            current_rule_fix_capabilities: rule.rule.fix(),
            severity: rule.severity.into(),
        }
    }

    /// Creates a new [`LintContext`] for testing purposes only.
    #[cfg(test)]
    pub(crate) fn spawn_for_test(self: Rc<Self>) -> LintContext<'a> {
        LintContext {
            parent: Rc::clone(&self),
            current_rule_name: "",
            current_plugin_name: "eslint",
            current_plugin_prefix: "eslint",
            #[cfg(debug_assertions)]
            current_rule_fix_capabilities: crate::rule::RuleFixMeta::None,
            severity: oxc_diagnostics::Severity::Warning,
        }
    }

    /// Maps Jest rule names and maps to Vitest rules when possible, returning the original plugin otherwise.
    ///
    /// Many Vitest rules are essentially ports of the Jest plugin rules with minor modifications.
    /// For these rules, we use the corresponding jest rules with some adjustments for compatibility.
    fn map_jest_rule_to_vitest(&self, rule: &RuleWithSeverity) -> &'static str {
        let plugin_name = rule.plugin_name();
        if self.plugins.has_vitest()
            && plugin_name == "jest"
            && utils::is_jest_rule_adapted_to_vitest(rule.name())
        {
            "vitest"
        } else {
            plugin_name
        }
    }

    /// Inspect the target file for clues about what frameworks are being used.
    /// Should only be called once immediately after construction.
    ///
    /// Before invocation, `self.frameworks` contains hints obtained at the
    /// project level. For example, Oxlint may (eventually) search for a
    /// `package.json`` and look for relevant dependencies. This method builds
    /// on top of those hints, providing a more granular understanding of the
    /// frameworks in use.
    fn sniff_for_frameworks(mut self) -> Self {
        if self.plugins.has_test() {
            // let mut test_flags = FrameworkFlags::empty();

            let vitest_like = frameworks::has_vitest_imports(self.semantic.module_record());
            let jest_like = frameworks::is_jestlike_file(&self.file_path)
                || frameworks::has_jest_imports(self.semantic.module_record());

            self.frameworks.set(FrameworkFlags::Vitest, vitest_like);
            self.frameworks.set(FrameworkFlags::Jest, jest_like);
        }

        self
    }

    /// Returns the framework hints for the target file.
    #[inline]
    pub fn frameworks(&self) -> FrameworkFlags {
        self.frameworks
    }
}

impl<'a> From<ContextHost<'a>> for Vec<Message<'a>> {
    fn from(ctx_host: ContextHost<'a>) -> Self {
        ctx_host.diagnostics.into_inner()
    }
}
