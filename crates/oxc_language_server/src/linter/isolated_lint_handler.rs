use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use log::debug;
use rustc_hash::FxHashSet;
use tower_lsp_server::{
    UriExt,
    lsp_types::{self, DiagnosticRelatedInformation, DiagnosticSeverity, Uri},
};

use oxc_allocator::Allocator;
use oxc_linter::{
    ConfigStore, LINTABLE_EXTENSIONS, LintOptions, LintService, LintServiceOptions, Linter,
    MessageWithPosition, loader::Loader, read_to_arena_str,
};
use oxc_linter::{RuntimeFileSystem, read_to_string};

use super::error_with_position::{
    DiagnosticReport, PossibleFixContent, message_with_position_to_lsp_diagnostic_report,
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

        if let Some(tsconfig_path) = &options.tsconfig_path {
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

        Some(self.lint_path(&mut allocator, &path, content).map_or(vec![], |errors| {
            let mut diagnostics: Vec<DiagnosticReport> = errors
                .iter()
                .map(|e| message_with_position_to_lsp_diagnostic_report(e, uri))
                .collect();

            // a diagnostics connected from related_info to original diagnostic
            let mut inverted_diagnostics = vec![];
            for d in &diagnostics {
                let Some(related_info) = &d.diagnostic.related_information else {
                    continue;
                };
                let related_information = Some(vec![DiagnosticRelatedInformation {
                    location: lsp_types::Location { uri: uri.clone(), range: d.diagnostic.range },
                    message: "original diagnostic".to_string(),
                }]);
                for r in related_info {
                    if r.location.range == d.diagnostic.range {
                        continue;
                    }
                    // If there is no message content for this span, then don't produce an additional diagnostic
                    // which also has no content. This prevents issues where editors expect diagnostics to have messages.
                    if r.message.is_empty() {
                        continue;
                    }
                    inverted_diagnostics.push(DiagnosticReport {
                        diagnostic: lsp_types::Diagnostic {
                            range: r.location.range,
                            severity: Some(DiagnosticSeverity::HINT),
                            code: None,
                            message: r.message.clone(),
                            source: d.diagnostic.source.clone(),
                            code_description: None,
                            related_information: related_information.clone(),
                            tags: None,
                            data: None,
                        },
                        fixed_content: PossibleFixContent::None,
                        rule_name: None,
                    });
                }
            }
            diagnostics.append(&mut inverted_diagnostics);
            diagnostics
        }))
    }

    fn lint_path<'a>(
        &mut self,
        allocator: &'a mut Allocator,
        path: &Path,
        source_text: Option<String>,
    ) -> Option<Vec<MessageWithPosition<'a>>> {
        if !Loader::can_load(path) {
            debug!("extension not supported yet.");
            return None;
        }

        let source_text = source_text.or_else(|| read_to_string(path).ok())?;

        debug!("lint {}", path.display());

        let result = self
            .service
            .with_file_system(Box::new(IsolatedLintHandlerFileSystem::new(
                path.to_path_buf(),
                source_text,
            )))
            .with_paths(vec![Arc::from(path.as_os_str())])
            .run_source(allocator);

        Some(result)
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
