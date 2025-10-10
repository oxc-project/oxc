use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use log::debug;
use oxc_data_structures::rope::Rope;
use rustc_hash::FxHashSet;
use tower_lsp_server::{UriExt, lsp_types::Uri};

use oxc_allocator::Allocator;
use oxc_linter::{
    AllowWarnDeny, ConfigStore, DirectivesStore, DisableDirectives, LINTABLE_EXTENSIONS,
    LintOptions, LintService, LintServiceOptions, Linter, Message, RuntimeFileSystem,
    create_unused_directives_diagnostics, read_to_arena_str, read_to_string,
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
                self.create_unused_directives_messages(&directives, severity)
                    .iter()
                    .map(|message| message_to_lsp_diagnostic(message, uri, source_text, rope)),
            );
        }

        messages
    }

    #[expect(clippy::unused_self)]
    fn create_unused_directives_messages(
        &self,
        directives: &DisableDirectives,
        severity: AllowWarnDeny,
    ) -> Vec<Message> {
        let diagnostics = create_unused_directives_diagnostics(directives, severity);
        diagnostics
            .into_iter()
            // TODO: unused directives should be fixable, `RuleCommentRule.create_fix()` can be used
            .map(|diagnostic| Message::new(diagnostic, oxc_linter::PossibleFixes::None))
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
