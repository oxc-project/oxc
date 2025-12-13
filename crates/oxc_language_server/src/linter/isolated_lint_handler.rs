use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use log::{debug, warn};
use oxc_data_structures::rope::Rope;
use rustc_hash::{FxHashMap, FxHashSet};
use tower_lsp_server::ls_types::Uri;

use oxc_allocator::Allocator;
use oxc_linter::{
    AllowWarnDeny, ConfigStore, DisableDirectives, Fix, FixKind, LINTABLE_EXTENSIONS, LintOptions,
    LintRunner, LintRunnerBuilder, LintServiceOptions, Linter, Message, PossibleFixes,
    RuleCommentType, RuntimeFileSystem, read_to_arena_str, read_to_string,
};

use super::error_with_position::{
    DiagnosticReport, generate_inverted_diagnostics, message_to_lsp_diagnostic,
};

/// smaller subset of LintServiceOptions, which is used by IsolatedLintHandler
#[derive(Debug, Clone)]
pub struct IsolatedLintHandlerOptions {
    pub use_cross_module: bool,
    pub type_aware: bool,
    pub fix_kind: FixKind,
    pub root_path: PathBuf,
    pub tsconfig_path: Option<PathBuf>,
}

pub struct IsolatedLintHandler {
    runner: LintRunner,
    unused_directives_severity: Option<AllowWarnDeny>,
}

#[derive(Default)]
pub struct IsolatedLintHandlerFileSystem {
    map: FxHashMap<PathBuf, Arc<str>>,
}

impl IsolatedLintHandlerFileSystem {
    pub fn add_file(&mut self, path: PathBuf, content: Arc<str>) {
        self.map.insert(path, content);
    }
}

impl RuntimeFileSystem for IsolatedLintHandlerFileSystem {
    fn read_to_arena_str<'a>(
        &'a self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<&'a str, std::io::Error> {
        match self.map.get(path) {
            Some(s) => Ok(&**s),
            None => read_to_arena_str(path, allocator),
        }
    }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<(), std::io::Error> {
        panic!("writing file should not be allowed in Language Server");
    }
}

impl IsolatedLintHandler {
    pub fn new(
        lint_options: LintOptions,
        config_store: ConfigStore,
        options: &IsolatedLintHandlerOptions,
    ) -> Self {
        let config_store_clone = config_store.clone();

        let linter = Linter::new(lint_options, config_store, None);
        let mut lint_service_options = LintServiceOptions::new(options.root_path.clone())
            .with_cross_module(options.use_cross_module);

        if let Some(tsconfig_path) = &options.tsconfig_path
            && tsconfig_path.is_file()
        {
            debug_assert!(tsconfig_path.is_absolute());
            lint_service_options = lint_service_options.with_tsconfig(tsconfig_path);
        }

        let runner = match LintRunnerBuilder::new(lint_service_options.clone(), linter)
            .with_type_aware(options.type_aware)
            .with_fix_kind(options.fix_kind)
            .build()
        {
            Ok(runner) => runner,
            Err(e) => {
                warn!("Failed to initialize type-aware linting: {e}");
                let linter = Linter::new(lint_options, config_store_clone, None);
                LintRunnerBuilder::new(lint_service_options, linter)
                    .with_type_aware(false)
                    .with_fix_kind(options.fix_kind)
                    .build()
                    .expect("Failed to build LintRunner without type-aware linting")
            }
        };

        Self { runner, unused_directives_severity: lint_options.report_unused_directive }
    }

    pub fn run_single(&self, uri: &Uri, content: Option<&str>) -> Option<Vec<DiagnosticReport>> {
        let path = uri.to_file_path()?;

        if !Self::should_lint_path(&path) {
            return None;
        }

        let source_text =
            if let Some(content) = content { content } else { &read_to_string(&path).ok()? };

        let mut diagnostics = self.lint_path(&path, uri, source_text);
        diagnostics.append(&mut generate_inverted_diagnostics(&diagnostics, uri));
        Some(diagnostics)
    }

    fn lint_path(&self, path: &Path, uri: &Uri, source_text: &str) -> Vec<DiagnosticReport> {
        debug!("lint {}", path.display());
        let rope = &Rope::from_str(source_text);

        let mut fs = IsolatedLintHandlerFileSystem::default();
        fs.add_file(path.to_path_buf(), Arc::from(source_text));

        let mut messages: Vec<DiagnosticReport> = self
            .runner
            .run_source(&Arc::from(path.as_os_str()), &fs)
            .into_iter()
            .map(|message| message_to_lsp_diagnostic(message, uri, source_text, rope))
            .collect();

        // Add unused directives if configured
        if let Some(severity) = self.unused_directives_severity
            && let Some(directives) = self.runner.directives_coordinator().get(path)
        {
            messages.extend(
                create_unused_directives_messages(&directives, severity, source_text)
                    .into_iter()
                    .map(|message| message_to_lsp_diagnostic(message, uri, source_text, rope)),
            );
        }

        messages
    }

    fn should_lint_path(path: &Path) -> bool {
        static WANTED_EXTENSIONS: OnceLock<FxHashSet<&'static str>> = OnceLock::new();
        let wanted_exts =
            WANTED_EXTENSIONS.get_or_init(|| LINTABLE_EXTENSIONS.iter().copied().collect());

        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| wanted_exts.contains(ext))
    }
}

/// Almost the same as [oxc_linter::create_unused_directives_diagnostics], but returns `Message`s
/// with a `PossibleFixes` instead of `OxcDiagnostic`s.
fn create_unused_directives_messages(
    directives: &DisableDirectives,
    severity: AllowWarnDeny,
    source_text: &str,
) -> Vec<Message> {
    use oxc_diagnostics::OxcDiagnostic;

    let mut diagnostics = Vec::new();
    let fix_message = "remove unused disable directive";

    let severity = if severity == AllowWarnDeny::Deny {
        oxc_diagnostics::Severity::Error
    } else {
        oxc_diagnostics::Severity::Warning
    };

    // Report unused disable comments
    let unused_disable = directives.collect_unused_disable_comments();
    for unused_comment in unused_disable {
        let span = unused_comment.span;
        match unused_comment.r#type {
            RuleCommentType::All => {
                diagnostics.push(Message::new(
                    OxcDiagnostic::warn(
                        "Unused eslint-disable directive (no problems were reported).",
                    )
                    .with_label(span)
                    .with_severity(severity),
                    PossibleFixes::Single(Fix::delete(span).with_message(fix_message)),
                ));
            }
            RuleCommentType::Single(rules) => {
                for rule in rules {
                    let rule_message = format!(
                        "Unused eslint-disable directive (no problems were reported from {}).",
                        rule.rule_name
                    );
                    diagnostics.push(Message::new(
                        OxcDiagnostic::warn(rule_message)
                            .with_label(rule.name_span)
                            .with_severity(severity),
                        PossibleFixes::Single(
                            rule.create_fix(source_text, span).with_message(fix_message),
                        ),
                    ));
                }
            }
        }
    }

    // Report unused enable comments
    let unused_enable = directives.unused_enable_comments();
    for (rule_name, span) in unused_enable {
        let message = if let Some(rule_name) = rule_name {
            format!(
                "Unused eslint-enable directive (no matching eslint-disable directives were found for {rule_name})."
            )
        } else {
            "Unused eslint-enable directive (no matching eslint-disable directives were found)."
                .to_string()
        };
        diagnostics.push(Message::new(
            OxcDiagnostic::warn(message).with_label(*span).with_severity(severity),
            // TODO: fixer
            // copy the structure of disable directives
            PossibleFixes::None,
        ));
    }

    diagnostics
}
