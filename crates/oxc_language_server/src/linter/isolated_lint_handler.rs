use std::{
    fs,
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
    LINTABLE_EXTENSIONS, LintService, LintServiceOptions, Linter, MessageWithPosition,
    loader::Loader,
};

use super::error_with_position::{
    DiagnosticReport, message_with_position_to_lsp_diagnostic_report,
};

/// smaller subset of LintServiceOptions, which is used by IsolatedLintHandler
#[derive(Debug, Clone)]
pub struct IsolatedLintHandlerOptions {
    pub use_cross_module: bool,
    pub root_path: PathBuf,
}

pub struct IsolatedLintHandler {
    linter: Arc<Linter>,
    options: Arc<IsolatedLintHandlerOptions>,
}

impl IsolatedLintHandler {
    pub fn new(linter: Arc<Linter>, options: Arc<IsolatedLintHandlerOptions>) -> Self {
        Self { linter, options }
    }

    pub fn run_single(
        &self,
        path: &Path,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if !Self::should_lint_path(path) {
            return None;
        }

        let allocator = Allocator::default();

        Some(self.lint_path(&allocator, path, content).map_or(vec![], |errors| {
            let path_buf = &path.to_path_buf();

            let mut diagnostics: Vec<DiagnosticReport> = errors
                .iter()
                .map(|e| message_with_position_to_lsp_diagnostic_report(e, path_buf))
                .collect();

            // a diagnostics connected from related_info to original diagnostic
            let mut inverted_diagnostics = vec![];
            for d in &diagnostics {
                let Some(related_info) = &d.diagnostic.related_information else {
                    continue;
                };
                let related_information = Some(vec![DiagnosticRelatedInformation {
                    location: lsp_types::Location {
                        uri: Uri::from_file_path(path).unwrap(),
                        range: d.diagnostic.range,
                    },
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
                        fixed_content: None,
                    });
                }
            }
            diagnostics.append(&mut inverted_diagnostics);
            diagnostics
        }))
    }

    fn lint_path<'a>(
        &self,
        allocator: &'a Allocator,
        path: &Path,
        source_text: Option<String>,
    ) -> Option<Vec<MessageWithPosition<'a>>> {
        if !Loader::can_load(path) {
            debug!("extension not supported yet.");
            return None;
        }
        let source_text = source_text.or_else(|| fs::read_to_string(path).ok())?;

        debug!("lint {path:?}");

        let lint_service_options = LintServiceOptions::new(
            self.options.root_path.clone(),
            vec![Arc::from(path.as_os_str())],
        )
        .with_cross_module(self.options.use_cross_module);
        // ToDo: do not clone the linter
        let path_arc = Arc::from(path.as_os_str());
        let mut lint_service = LintService::new((*self.linter).clone(), lint_service_options);
        let result = lint_service.run_source(allocator, &path_arc, &source_text);

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
