use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use log::{debug, info};
use oxc_data_structures::rope::Rope;
use oxc_linter::read_to_string;
use rustc_hash::FxHashSet;
use std::collections::HashMap;
use tower_lsp_server::{UriExt, lsp_types::Uri};

use oxc_allocator::Allocator;
use oxc_linter::{
    AllowWarnDeny, ConfigStore, DirectivesStore, DisableDirectives, Fix, LINTABLE_EXTENSIONS,
    LintOptions, LintService, LintServiceOptions, Linter, Message, PossibleFixes, RuleCommentType,
    RuntimeFileSystem, read_to_arena_str,
};

use super::error_with_position::{
    DiagnosticReport, generate_inverted_diagnostics, message_to_lsp_diagnostic,
};

/// smaller subset of LintServiceOptions, which is used by IsolatedLintHandler
#[derive(Debug, Clone)]
pub struct IsolatedLintHandlerOptions {
    pub use_cross_module: bool,
    pub root_path: PathBuf,
    pub tsconfig_path: Option<PathBuf>,
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

        Self {
            service,
            directives_coordinator,
            unused_directives_severity: lint_options.report_unused_directive,
        }
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

        let source_text = content.or_else(|| read_to_string(&path).ok())?;

        let mut diagnostics = self.lint_path(&path, uri, &source_text);
        diagnostics.append(&mut generate_inverted_diagnostics(&diagnostics, uri));
        Some(diagnostics)
    }

    fn lint_path(&mut self, path: &Path, uri: &Uri, source_text: &str) -> Vec<DiagnosticReport> {
        debug!("lint {}", path.display());
        let rope = &Rope::from_str(source_text);

        let mut messages: Vec<DiagnosticReport> = self
            .service
            .with_file_system(Box::new(IsolatedLintHandlerFileSystem::new(
                path.to_path_buf(),
                Arc::from(source_text),
            )))
            .with_paths(vec![Arc::from(path.as_os_str())])
            .run_source()
            .iter()
            .map(|message| message_to_lsp_diagnostic(message, uri, source_text, rope))
            .collect();

        // Add unused directives if configured
        if let Some(severity) = self.unused_directives_severity
            && let Some(directives) = self.directives_coordinator.get(path)
        {
            messages.extend(
                create_unused_directives_messages(&directives, severity, source_text)
                    .iter()
                    .map(|message| message_to_lsp_diagnostic(message, uri, source_text, rope)),
            );
        }

        messages
    }

    /// Batch lint multiple paths using the underlying parallel runtime.
    /// Returns a vector of (Uri, DiagnosticReport list). Ignores non-lintable paths silently.
    pub fn run_workspace(&mut self, paths: &[PathBuf]) -> Vec<(Uri, Vec<DiagnosticReport>)> {
        // Filter to lintable extensions first.
        let lintable: Vec<PathBuf> =
            paths.iter().filter(|p| Self::should_lint_path(p)).cloned().collect();
        if lintable.is_empty() {
            return Vec::new();
        }
        info!(
            "[isolated] workspace batch start paths_total={} lintable={}",
            paths.len(),
            lintable.len()
        );
        let t_batch = Some(std::time::Instant::now());
        let arc_paths: Vec<Arc<std::ffi::OsStr>> =
            lintable.iter().map(|p| Arc::from(p.as_os_str())).collect();

        // Run parallel lint across all entry paths.
        let messages = self.service.with_paths(arc_paths).run_source();

        // Group messages by originating file path.
        let mut grouped: HashMap<Arc<std::ffi::OsStr>, Vec<Message>> = HashMap::new();
        for msg in messages {
            grouped.entry(msg.file_path.clone()).or_default().push(msg);
        }

        let mut out: Vec<(Uri, Vec<DiagnosticReport>)> = Vec::with_capacity(grouped.len());

        for (file_os, msgs) in grouped.into_iter() {
            let path_buf = PathBuf::from(file_os.as_ref());
            // Read source text (skip if unreadable).
            let Ok(source_text) = read_to_string(&path_buf) else {
                continue;
            };
            let rope = Rope::from_str(&source_text);
            let Some(uri) = Uri::from_file_path(&path_buf) else {
                continue;
            };

            let mut reports: Vec<DiagnosticReport> = msgs
                .iter()
                .map(|m| message_to_lsp_diagnostic(m, &uri, &source_text, &rope))
                .collect();

            // Append unused directives diagnostics if configured
            if let Some(severity) = self.unused_directives_severity
                && let Some(directives) = self.directives_coordinator.get(&path_buf)
            {
                let unused = create_unused_directives_messages(&directives, severity, &source_text);
                reports.extend(
                    unused.iter().map(|m| message_to_lsp_diagnostic(m, &uri, &source_text, &rope)),
                );
            }

            // Inverted related span diagnostics
            let inverted = generate_inverted_diagnostics(&reports, &uri);
            reports.extend(inverted);

            let count = reports.len();
            out.push((uri.clone(), reports));
            info!("[isolated] workspace file uri={} diagnostics={}", uri.as_str(), count);
        }

        // Stable ordering for deterministic publish (by URI string)
        out.sort_unstable_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        if let Some(t_batch) = t_batch {
            debug!(
                "[profile] workspace isolated batch lintable={} output_files={} total_diagnostics={} ms={}",
                lintable.len(),
                out.len(),
                out.iter().map(|(_, ds)| ds.len()).sum::<usize>(),
                t_batch.elapsed().as_millis()
            );
        }
        info!(
            "[isolated] workspace batch done lintable={} output_files={} total_diagnostics={}",
            lintable.len(),
            out.len(),
            out.iter().map(|(_, ds)| ds.len()).sum::<usize>()
        );
        out
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
