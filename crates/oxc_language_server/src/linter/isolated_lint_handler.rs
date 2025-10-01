use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use log::debug;
use rustc_hash::FxHashSet;
use tower_lsp_server::{UriExt, lsp_types::Uri};

use oxc_allocator::Allocator;
use oxc_linter::{
    ConfigStore, LINTABLE_EXTENSIONS, LintOptions, LintService, LintServiceOptions, Linter,
    MessageWithPosition, read_to_arena_str,
};
use oxc_linter::{RuntimeFileSystem, read_to_string};

use super::error_with_position::{
    DiagnosticReport, generate_inverted_diagnostics, message_with_position_to_lsp_diagnostic_report,
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
}

pub struct IsolatedLintHandlerFileSystem {
    path_to_lint: PathBuf,
    source_text: String,
}

impl IsolatedLintHandlerFileSystem {
    pub fn new(path_to_lint: PathBuf, source_text: String) -> Self {
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
        let linter = Linter::new(lint_options, config_store, None);
        let mut lint_service_options = LintServiceOptions::new(options.root_path.clone())
            .with_cross_module(options.use_cross_module);

        if let Some(tsconfig_path) = &options.tsconfig_path
            && tsconfig_path.is_file()
        {
            debug_assert!(tsconfig_path.is_absolute());
            lint_service_options = lint_service_options.with_tsconfig(tsconfig_path);
        }

        let service = LintService::new(linter, lint_service_options);

        Self { service }
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
        let errors = self.lint_path(&mut allocator, &path, source_text);

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
        source_text: String,
    ) -> Vec<MessageWithPosition<'a>> {
        debug!("lint {}", path.display());

        self.service
            .with_file_system(Box::new(IsolatedLintHandlerFileSystem::new(
                path.to_path_buf(),
                source_text,
            )))
            .with_paths(vec![Arc::from(path.as_os_str())])
            .run_source(allocator)
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
