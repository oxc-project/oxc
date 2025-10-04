use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use log::debug;
use rustc_hash::FxHashSet;
use tower_lsp_server::{UriExt, lsp_types::Uri};

use oxc_allocator::Allocator;
use oxc_linter::{
    AllowWarnDeny, ConfigStore, DirectivesStore, DisableDirectives, LINTABLE_EXTENSIONS,
    LintOptions, LintService, LintServiceOptions, Linter, MessageWithPosition, read_to_arena_str,
};
use oxc_linter::{RuntimeFileSystem, read_to_string};

use super::error_with_position::{
    DiagnosticReport, generate_inverted_diagnostics, message_with_position_to_lsp_diagnostic_report,
};
use super::options::UnusedDisableDirectives;

/// smaller subset of LintServiceOptions, which is used by IsolatedLintHandler
#[derive(Debug, Clone)]
pub struct IsolatedLintHandlerOptions {
    pub use_cross_module: bool,
    pub root_path: PathBuf,
    pub tsconfig_path: Option<PathBuf>,
    pub unused_disable_directives: UnusedDisableDirectives,
}

pub struct IsolatedLintHandler {
    service: LintService,
    directives_coordinator: DirectivesStore,
    unused_directives_severity: Option<AllowWarnDeny>,
}

pub struct IsolatedLintHandlerFileSystem {
    path_to_lint: PathBuf,
    source_text: Arc<str>,
}

impl IsolatedLintHandlerFileSystem {
    pub fn new(path_to_lint: PathBuf, source_text: Arc<str>) -> Self {
        Self { path_to_lint, source_text }
    }
}

impl RuntimeFileSystem for IsolatedLintHandlerFileSystem {
    fn read_to_arena_str<'a>(
        &'a self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<&'a str, std::io::Error> {
        if path == self.path_to_lint {
            return Ok(&self.source_text);
        }

        read_to_arena_str(path, allocator)
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
        let directives_coordinator = DirectivesStore::new();
        let unused_directives_severity = match options.unused_disable_directives {
            UnusedDisableDirectives::Allow => None,
            UnusedDisableDirectives::Warn => Some(AllowWarnDeny::Warn),
            UnusedDisableDirectives::Deny => Some(AllowWarnDeny::Deny),
        };

        let linter = Linter::new(lint_options, config_store, None);
        let mut lint_service_options = LintServiceOptions::new(options.root_path.clone())
            .with_cross_module(options.use_cross_module);

        if let Some(tsconfig_path) = &options.tsconfig_path
            && tsconfig_path.is_file()
        {
            debug_assert!(tsconfig_path.is_absolute());
            lint_service_options = lint_service_options.with_tsconfig(tsconfig_path);
        }

        let mut service = LintService::new(linter, lint_service_options);
        service.set_disable_directives_map(directives_coordinator.map());

        Self { service, directives_coordinator, unused_directives_severity }
    }

    pub fn run_single(
        &mut self,
        uri: &Uri,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        let path = uri.to_file_path()?;

        if !Self::should_lint_path(&path) {
            return None;
        }

        let mut allocator = Allocator::default();
        let source_text = content.or_else(|| read_to_string(&path).ok())?;
        let errors = self.lint_path(&mut allocator, &path, &source_text);

        let mut diagnostics: Vec<DiagnosticReport> =
            errors.iter().map(|e| message_with_position_to_lsp_diagnostic_report(e, uri)).collect();

        let mut inverted_diagnostics = generate_inverted_diagnostics(&diagnostics, uri);
        diagnostics.append(&mut inverted_diagnostics);
        Some(diagnostics)
    }

    fn lint_path<'a>(
        &mut self,
        allocator: &'a mut Allocator,
        path: &Path,
        source_text: &str,
    ) -> Vec<MessageWithPosition<'a>> {
        debug!("lint {}", path.display());

        let mut messages = self
            .service
            .with_file_system(Box::new(IsolatedLintHandlerFileSystem::new(
                path.to_path_buf(),
                Arc::from(source_text),
            )))
            .with_paths(vec![Arc::from(path.as_os_str())])
            .run_source(allocator);

        // Add unused directives if configured
        if let Some(severity) = self.unused_directives_severity
            && let Some(directives) = self.directives_coordinator.get(path)
        {
            messages.extend(self.create_unused_directives_messages(
                &directives,
                severity,
                source_text,
            ));
        }

        messages
    }

    #[expect(clippy::unused_self)]
    fn create_unused_directives_messages(
        &self,
        directives: &DisableDirectives,
        severity: AllowWarnDeny,
        source_text: &str,
    ) -> Vec<MessageWithPosition<'static>> {
        use oxc_data_structures::rope::Rope;
        use oxc_diagnostics::OxcDiagnostic;
        use oxc_linter::{RuleCommentType, oxc_diagnostic_to_message_with_position};

        let mut diagnostics = Vec::new();
        let rope = Rope::from_str(source_text);

        // Report unused disable comments
        let unused_disable = directives.collect_unused_disable_comments();
        for unused_comment in unused_disable {
            let span = unused_comment.span;
            match unused_comment.r#type {
                RuleCommentType::All => {
                    let diagnostic = OxcDiagnostic::warn(
                        "Unused eslint-disable directive (no problems were reported).",
                    )
                    .with_label(span)
                    .with_severity(if severity == AllowWarnDeny::Deny {
                        oxc_diagnostics::Severity::Error
                    } else {
                        oxc_diagnostics::Severity::Warning
                    });
                    diagnostics.push(diagnostic);
                }
                RuleCommentType::Single(rules) => {
                    // Report each unused rule separately
                    for rule in rules {
                        let rule_message = format!(
                            "Unused eslint-disable directive (no problems were reported from {}).",
                            rule.rule_name
                        );
                        let diagnostic = OxcDiagnostic::warn(rule_message)
                            .with_label(rule.name_span)
                            .with_severity(if severity == AllowWarnDeny::Deny {
                                oxc_diagnostics::Severity::Error
                            } else {
                                oxc_diagnostics::Severity::Warning
                            });
                        diagnostics.push(diagnostic);
                    }
                }
            }
        }

        // Report unused enable comments
        let unused_enable = directives.unused_enable_comments();
        for (rule_name, span) in unused_enable {
            let message = if let Some(rule_name) = rule_name {
                format!(
                    "Unused eslint-enable directive (no matching eslint-disable found for {rule_name})."
                )
            } else {
                "Unused eslint-enable directive (no matching eslint-disable found).".to_string()
            };

            let diagnostic = OxcDiagnostic::warn(message).with_label(*span).with_severity(
                if severity == AllowWarnDeny::Deny {
                    oxc_diagnostics::Severity::Error
                } else {
                    oxc_diagnostics::Severity::Warning
                },
            );
            diagnostics.push(diagnostic);
        }

        // Convert diagnostics to MessageWithPosition
        diagnostics
            .into_iter()
            .map(|diagnostic| {
                oxc_diagnostic_to_message_with_position(diagnostic, source_text, &rope)
            })
            .collect()
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
