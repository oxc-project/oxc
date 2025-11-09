use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    ffi::OsStr,
    path::Path,
    rc::Rc,
    sync::Arc,
};

use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_semantic::Semantic;
use oxc_span::{SourceType, Span};

use crate::{
    AllowWarnDeny, FrameworkFlags,
    config::{LintConfig, LintPlugins, OxlintSettings},
    disable_directives::{DisableDirectives, DisableDirectivesBuilder, RuleCommentType},
    fixer::{Fix, FixKind, Message, PossibleFixes},
    frameworks::{self, FrameworkOptions},
    module_record::ModuleRecord,
    options::LintOptions,
    rules::RuleEnum,
};

use super::{LintContext, plugin_name_to_prefix};

/// Stores shared information about a script block being linted.
pub struct ContextSubHost<'a> {
    /// Semantic information about the file being linted, which includes scopes, symbols and AST nodes.
    /// See [`Semantic`].
    pub(super) semantic: Semantic<'a>,
    /// Cross module information.
    pub(super) module_record: Arc<ModuleRecord>,
    /// Information about specific rules that should be disabled or enabled, via comment directives like
    /// `eslint-disable` or `eslint-disable-next-line`.
    pub(super) disable_directives: DisableDirectives,
    // Specific framework options, for example, whether the context is inside `<script setup>` in Vue files.
    pub(super) framework_options: FrameworkOptions,
    /// The source text offset of the sub host
    pub(super) source_text_offset: u32,
}

impl<'a> ContextSubHost<'a> {
    pub fn new(
        semantic: Semantic<'a>,
        module_record: Arc<ModuleRecord>,
        source_text_offset: u32,
    ) -> Self {
        Self::new_with_framework_options(
            semantic,
            module_record,
            source_text_offset,
            FrameworkOptions::Default,
        )
    }

    /// # Panics
    /// If `semantic.cfg()` is `None`.
    pub fn new_with_framework_options(
        semantic: Semantic<'a>,
        module_record: Arc<ModuleRecord>,
        source_text_offset: u32,
        frameworks_options: FrameworkOptions,
    ) -> Self {
        // We should always check for `semantic.cfg()` being `Some` since we depend on it and it is
        // unwrapped without any runtime checks after construction.
        assert!(
            semantic.cfg().is_some(),
            "`LintContext` depends on `Semantic::cfg`, Build your semantic with cfg enabled(`SemanticBuilder::with_cfg`)."
        );

        let disable_directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        Self {
            semantic,
            module_record,
            source_text_offset,
            disable_directives,
            framework_options: frameworks_options,
        }
    }

    /// Shared reference to the [`Semantic`] analysis
    #[inline]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    /// Shared reference to the [`ModuleRecord`]
    #[inline]
    pub fn module_record(&self) -> &ModuleRecord {
        &self.module_record
    }

    /// Shared reference to the [`DisableDirectives`]
    pub fn disable_directives(&self) -> &DisableDirectives {
        &self.disable_directives
    }

    /// Shared reference to the [`FrameworkOptions`]
    pub fn framework_options(&self) -> FrameworkOptions {
        self.framework_options
    }
}

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
    /// A file can have multiple script entries.
    /// Some rules (like vue) need the information of the other entries.
    pub(super) sub_hosts: Vec<ContextSubHost<'a>>,
    /// The current index which will be linted.
    current_sub_host_index: Cell<usize>,
    /// Diagnostics reported by the linter.
    ///
    /// Contains diagnostics for all rules across a single file.
    diagnostics: RefCell<Vec<Message>>,
    /// Whether or not to apply code fixes during linting. Defaults to
    /// [`FixKind::None`] (no fixing).
    ///
    /// Set via the `--fix`, `--fix-suggestions`, and `--fix-dangerously` CLI
    /// flags.
    pub(super) fix: FixKind,
    /// Path to the file being linted.
    pub(super) file_path: Box<Path>,
    /// Extension of the file being linted.
    file_extension: Option<Box<OsStr>>,
    /// Global linter configuration, such as globals to include and the target
    /// environments, and other settings.
    pub(super) config: Arc<LintConfig>,
    /// Front-end frameworks that might be in use in the target file.
    pub(super) frameworks: FrameworkFlags,
}

impl std::fmt::Debug for ContextHost<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextHost").field("file_path", &self.file_path).finish_non_exhaustive()
    }
}

impl<'a> ContextHost<'a> {
    /// # Panics
    /// If `sub_hosts` is empty.
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        sub_hosts: Vec<ContextSubHost<'a>>,
        options: LintOptions,
        config: Arc<LintConfig>,
    ) -> Self {
        const DIAGNOSTICS_INITIAL_CAPACITY: usize = 512;

        assert!(
            !sub_hosts.is_empty(),
            "ContextHost requires at least one ContextSubHost to be analyzed"
        );

        let file_path = file_path.as_ref().to_path_buf().into_boxed_path();
        let file_extension = file_path.extension().map(|ext| ext.to_owned().into_boxed_os_str());

        Self {
            sub_hosts,
            current_sub_host_index: Cell::new(0),
            diagnostics: RefCell::new(Vec::with_capacity(DIAGNOSTICS_INITIAL_CAPACITY)),
            fix: options.fix,
            file_path,
            file_extension,
            config,
            frameworks: options.framework_hints,
        }
        .sniff_for_frameworks()
    }

    /// The current [`ContextSubHost`]
    pub fn current_sub_host(&self) -> &ContextSubHost<'a> {
        &self.sub_hosts[self.current_sub_host_index.get()]
    }

    /// Get mutable reference to the current [`ContextSubHost`]
    fn current_sub_host_mut(&mut self) -> &mut ContextSubHost<'a> {
        &mut self.sub_hosts[self.current_sub_host_index.get()]
    }

    // Whether the current sub host is the first one.
    pub fn is_first_sub_host(&self) -> bool {
        self.current_sub_host_index.get() == 0
    }

    /// Shared reference to the [`Semantic`] analysis of current script block.
    #[inline]
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.current_sub_host().semantic
    }

    /// Mutable reference to the [`Semantic`] analysis of current script block.
    #[inline]
    pub fn semantic_mut(&mut self) -> &mut Semantic<'a> {
        &mut self.current_sub_host_mut().semantic
    }

    /// Shared reference to the [`ModuleRecord`] of the current script block.
    #[inline]
    pub fn module_record(&self) -> &ModuleRecord {
        &self.current_sub_host().module_record
    }

    /// Shared reference to the [`DisableDirectives`] of the current script block.
    pub fn disable_directives(&self) -> &DisableDirectives {
        &self.current_sub_host().disable_directives
    }

    /// Path to the file being linted.
    ///
    /// When created from a [`LintService`](`crate::service::LintService`), this
    /// will be an absolute path.
    #[inline]
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Extension of the file currently being linted, without the leading dot.
    #[inline]
    pub fn file_extension(&self) -> Option<&OsStr> {
        self.file_extension.as_deref()
    }

    /// The source type of the file being linted, e.g. JavaScript, TypeScript,
    /// CJS, ESM, etc.
    #[inline]
    pub fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    #[inline]
    pub fn plugins(&self) -> LintPlugins {
        self.config.plugins
    }

    #[inline]
    pub fn settings(&self) -> &OxlintSettings {
        &self.config.settings
    }

    /// Add a diagnostic message to the end of the list of diagnostics. Can be used
    /// by any rule to report issues.
    #[inline]
    pub(crate) fn push_diagnostic(&self, mut diagnostic: Message) {
        if self.current_sub_host().source_text_offset != 0 {
            diagnostic.move_offset(self.current_sub_host().source_text_offset);
        }
        self.diagnostics.borrow_mut().push(diagnostic);
    }

    // Append a list of diagnostics. Only used in report_unused_directives.
    fn append_diagnostics(&self, mut diagnostics: Vec<Message>) {
        if self.current_sub_host().source_text_offset != 0 {
            let offset = self.current_sub_host().source_text_offset;
            for diagnostic in &mut diagnostics {
                diagnostic.move_offset(offset);
            }
        }
        self.diagnostics.borrow_mut().extend(diagnostics);
    }

    // move the context to the next sub host
    pub fn next_sub_host(&self) -> bool {
        let next_index = self.current_sub_host_index.get() + 1;
        if next_index < self.sub_hosts.len() {
            self.current_sub_host_index.set(next_index);
            true
        } else {
            false
        }
    }

    /// report unused enable/disable directives, add these as Messages to diagnostics
    pub fn report_unused_directives(&self, rule_severity: Severity) {
        // report unused disable
        // relate to lint result, check after linter run finish
        let unused_disable_comments = self.disable_directives().collect_unused_disable_comments();
        let message_for_disable = "Unused eslint-disable directive (no problems were reported).";
        let fix_message = "remove unused disable directive";
        let source_text = self.semantic().source_text();

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

        let unused_enable_comments = self.disable_directives().unused_enable_comments();
        let mut unused_directive_diagnostics: Vec<(Cow<str>, Span)> =
            Vec::with_capacity(unused_enable_comments.len());
        // report unused enable
        // not relate to lint result, check during comment directives' construction
        let message_for_enable =
            "Unused eslint-enable directive (no matching eslint-disable directives were found).";
        for (rule_name, enable_comment_span) in self.disable_directives().unused_enable_comments() {
            unused_directive_diagnostics.push((
                rule_name.as_ref().map_or(Cow::Borrowed(message_for_enable), |name| {
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
    pub fn take_diagnostics(&self) -> Vec<Message> {
        // NOTE: diagnostics are only ever borrowed here and in push_diagnostic, append_diagnostics.
        // The latter drops the reference as soon as the function returns, so
        // this should never panic.
        let mut messages = self.diagnostics.borrow_mut();
        std::mem::take(&mut *messages)
    }

    /// Take ownership of the disable directives from the first sub host.
    /// This consumes the `ContextHost`.
    ///
    /// # Panics
    /// Panics if `sub_hosts` contains more than one sub host.
    pub fn into_disable_directives(self) -> Option<DisableDirectives> {
        assert!(
            self.sub_hosts.len() <= 1,
            "into_disable_directives expects at most one sub host, but found {}",
            self.sub_hosts.len()
        );
        self.sub_hosts.into_iter().next().map(|sub_host| sub_host.disable_directives)
    }

    #[cfg(debug_assertions)]
    pub fn get_diagnostics(&self, cb: impl FnOnce(&mut Vec<Message>)) {
        cb(self.diagnostics.borrow_mut().as_mut());
    }

    #[cfg(debug_assertions)]
    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.borrow().len()
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

    pub fn frameworks_options(&self) -> FrameworkOptions {
        self.current_sub_host().framework_options
    }

    pub fn other_file_hosts(&self) -> Vec<&ContextSubHost<'a>> {
        self.sub_hosts
            .iter()
            .enumerate()
            .filter(|&(index, _)| index != self.current_sub_host_index.get())
            .map(|(_, sub_host)| sub_host)
            .collect()
    }
}

impl<'a> From<ContextHost<'a>> for Vec<Message> {
    fn from(ctx_host: ContextHost<'a>) -> Self {
        ctx_host.diagnostics.into_inner()
    }
}
