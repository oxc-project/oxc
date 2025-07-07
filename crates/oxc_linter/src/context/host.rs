use std::{borrow::Cow, cell::RefCell, path::Path, rc::Rc, sync::Arc};

use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_semantic::Semantic;
use oxc_span::{SourceType, Span};

use crate::{
    AllowWarnDeny, FrameworkFlags,
    config::{LintConfig, LintPlugins},
    disable_directives::{DisableDirectives, DisableDirectivesBuilder, RuleCommentType},
    fixer::{Fix, FixKind, Message, PossibleFixes},
    frameworks,
    module_record::ModuleRecord,
    options::LintOptions,
    rules::RuleEnum,
};

use super::{LintContext, plugin_name_to_prefix};

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
pub struct ContextHost<'a> {
    /// Shared semantic information about the file being linted, which includes scopes, symbols
    /// and AST nodes. See [`Semantic`].
    pub(super) semantic: Rc<Semantic<'a>>,
    /// Cross module information.
    pub(super) module_record: Arc<ModuleRecord>,
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
}

impl<'a> ContextHost<'a> {
    /// # Panics
    /// If `semantic.cfg()` is `None`.
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        semantic: Rc<Semantic<'a>>,
        module_record: Arc<ModuleRecord>,
        options: LintOptions,
        config: Arc<LintConfig>,
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
            module_record,
            disable_directives,
            diagnostics: RefCell::new(Vec::with_capacity(DIAGNOSTICS_INITIAL_CAPACITY)),
            fix: options.fix,
            file_path,
            config,
            frameworks: options.framework_hints,
        }
        .sniff_for_frameworks()
    }

    /// Shared reference to the [`Semantic`] analysis of the file.
    #[inline]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    /// Shared reference to the [`ModuleRecord`] of the file.
    #[inline]
    pub fn module_record(&self) -> &ModuleRecord {
        &self.module_record
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

    #[inline]
    pub fn plugins(&self) -> &LintPlugins {
        &self.config.plugins
    }

    /// Add a diagnostic message to the end of the list of diagnostics. Can be used
    /// by any rule to report issues.
    #[inline]
    pub(super) fn push_diagnostic(&self, diagnostic: Message<'a>) {
        self.diagnostics.borrow_mut().push(diagnostic);
    }

    // Append a list of diagnostics. Only used in report_unused_directives.
    fn append_diagnostics(&self, diagnostics: Vec<Message<'a>>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }

    /// report unused enable/disable directives, add these as Messages to diagnostics
    pub fn report_unused_directives(&self, rule_severity: Severity) {
        // report unused disable
        // relate to lint result, check after linter run finish
        let unused_disable_comments = self.disable_directives.collect_unused_disable_comments();
        let message_for_disable = "Unused eslint-disable directive (no problems were reported).";
        let fix_message = "remove unused disable directive";
        let source_text = self.semantic.source_text();

        for unused_disable_comment in unused_disable_comments {
            let span = unused_disable_comment.span;
            match &unused_disable_comment.r#type {
                RuleCommentType::All => {
                    // eslint-disable
                    self.push_diagnostic(Message::new(
                        OxcDiagnostic::error(message_for_disable)
                            .with_label(span)
                            .with_severity(rule_severity),
                        PossibleFixes::Single(Fix::delete(span).with_message(fix_message)),
                    ));
                }
                RuleCommentType::Single(rules_vec) => {
                    for rule in rules_vec {
                        let rule_message = Cow::<str>::Owned(format!(
                            "Unused eslint-disable directive (no problems were reported from {}).",
                            rule.rule_name
                        ));

                        let fix = rule.create_fix(source_text, span).with_message(fix_message);

                        self.push_diagnostic(Message::new(
                            OxcDiagnostic::error(rule_message)
                                .with_label(rule.name_span)
                                .with_severity(rule_severity),
                            PossibleFixes::Single(fix),
                        ));
                    }
                }
            }
        }

        let unused_enable_comments = self.disable_directives.unused_enable_comments();
        let mut unused_directive_diagnostics: Vec<(Cow<str>, Span)> =
            Vec::with_capacity(unused_enable_comments.len());
        // report unused enable
        // not relate to lint result, check during comment directives' construction
        let message_for_enable =
            "Unused eslint-enable directive (no matching eslint-disable directives were found).";
        for (rule_name, enable_comment_span) in self.disable_directives.unused_enable_comments() {
            unused_directive_diagnostics.push((
                rule_name.map_or(Cow::Borrowed(message_for_enable), |name| {
                    Cow::Owned(format!(
                        "Unused eslint-enable directive (no matching eslint-disable directives were found for {name})."
                    ))
                }),
                *enable_comment_span,
            ));
        }

        self.append_diagnostics(
            unused_directive_diagnostics
                .into_iter()
                .map(|(message, span)| {
                    Message::new(
                        OxcDiagnostic::error(message).with_label(span).with_severity(rule_severity),
                        // TODO: fixer
                        // copy the structure of disable directives
                        PossibleFixes::None,
                    )
                })
                .collect(),
        );
    }

    /// Take ownership of all diagnostics collected during linting.
    pub fn take_diagnostics(&self) -> Vec<Message<'a>> {
        // NOTE: diagnostics are only ever borrowed here and in push_diagnostic, append_diagnostics.
        // The latter drops the reference as soon as the function returns, so
        // this should never panic.
        let mut messages = self.diagnostics.borrow_mut();
        std::mem::take(&mut *messages)
    }

    /// Creates a new [`LintContext`] for a specific rule.
    pub fn spawn(self: Rc<Self>, rule: &RuleEnum, severity: AllowWarnDeny) -> LintContext<'a> {
        let rule_name = rule.name();
        let plugin_name = rule.plugin_name();

        LintContext {
            parent: self,
            current_rule_name: rule_name,
            current_plugin_name: plugin_name,
            current_plugin_prefix: plugin_name_to_prefix(plugin_name),
            #[cfg(debug_assertions)]
            current_rule_fix_capabilities: rule.fix(),
            severity: severity.into(),
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

    /// Inspect the target file for clues about what frameworks are being used.
    /// Should only be called once immediately after construction.
    ///
    /// Before invocation, `self.frameworks` contains hints obtained at the
    /// project level. For example, Oxlint may (eventually) search for a
    /// `package.json`` and look for relevant dependencies. This method builds
    /// on top of those hints, providing a more granular understanding of the
    /// frameworks in use.
    fn sniff_for_frameworks(mut self) -> Self {
        if self.plugins().has_test() {
            // let mut test_flags = FrameworkFlags::empty();

            let vitest_like = frameworks::has_vitest_imports(self.module_record());
            let jest_like = frameworks::is_jestlike_file(&self.file_path)
                || frameworks::has_jest_imports(self.module_record());

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
