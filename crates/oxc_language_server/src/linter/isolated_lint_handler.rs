use std::{
    fs,
    path::Path,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use log::debug;
use rustc_hash::FxHashSet;
use tower_lsp::lsp_types::{self, DiagnosticRelatedInformation, DiagnosticSeverity, Range};

use oxc_allocator::Allocator;
use oxc_diagnostics::{Error, NamedSource};
use oxc_linter::{
    loader::{JavaScriptSource, Loader, LINT_PARTIAL_LOADER_EXT},
    Linter, ModuleRecord,
};
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_span::VALID_EXTENSIONS;

use crate::linter::error_with_position::{ErrorReport, ErrorWithPosition, FixedContent};
use crate::linter::offset_to_position;
use crate::DiagnosticReport;

pub struct IsolatedLintHandler {
    linter: Arc<Linter>,
    loader: Loader,
}

impl IsolatedLintHandler {
    pub fn new(linter: Arc<Linter>) -> Self {
        Self { linter, loader: Loader }
    }

    pub fn run_single(
        &self,
        path: &Path,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if !Self::should_lint_path(path) {
            return None;
        }

        Some(self.lint_path(path, content).map_or(vec![], |errors| {
            let path_buf = &path.to_path_buf();

            let mut diagnostics: Vec<DiagnosticReport> =
                errors.into_iter().map(|e| e.into_diagnostic_report(path_buf)).collect();

            // a diagnostics connected from related_info to original diagnostic
            let mut inverted_diagnostics = vec![];
            for d in &diagnostics {
                let Some(related_info) = &d.diagnostic.related_information else {
                    continue;
                };
                let related_information = Some(vec![DiagnosticRelatedInformation {
                    location: lsp_types::Location {
                        uri: lsp_types::Url::from_file_path(path).unwrap(),
                        range: d.diagnostic.range,
                    },
                    message: "original diagnostic".to_string(),
                }]);
                for r in related_info {
                    if r.location.range == d.diagnostic.range {
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

    fn lint_path(
        &self,
        path: &Path,
        source_text: Option<String>,
    ) -> Option<Vec<ErrorWithPosition>> {
        if !Loader::can_load(path) {
            debug!("extension not supported yet.");
            return None;
        }
        let source_text = source_text.or_else(|| fs::read_to_string(path).ok())?;
        let javascript_sources = match self.loader.load_str(path, &source_text) {
            Ok(s) => s,
            Err(e) => {
                debug!("failed to load {path:?}: {e}");
                return None;
            }
        };

        debug!("lint {path:?}");
        let mut diagnostics = vec![];
        for source in javascript_sources {
            let JavaScriptSource {
                source_text: javascript_source_text, source_type, start, ..
            } = source;
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, javascript_source_text, source_type)
                .with_options(ParseOptions {
                    allow_return_outside_function: true,
                    ..ParseOptions::default()
                })
                .parse();

            if !ret.errors.is_empty() {
                let reports = ret
                    .errors
                    .into_iter()
                    .map(|diagnostic| ErrorReport {
                        error: Error::from(diagnostic),
                        fixed_content: None,
                    })
                    .collect();
                return Some(Self::wrap_diagnostics(path, &source_text, reports, start));
            };

            let semantic_ret = SemanticBuilder::new()
                .with_cfg(true)
                .with_scope_tree_child_ids(true)
                .with_check_syntax_error(true)
                .build(&ret.program);

            if !semantic_ret.errors.is_empty() {
                let reports = semantic_ret
                    .errors
                    .into_iter()
                    .map(|diagnostic| ErrorReport {
                        error: Error::from(diagnostic),
                        fixed_content: None,
                    })
                    .collect();
                return Some(Self::wrap_diagnostics(path, &source_text, reports, start));
            };

            let mut semantic = semantic_ret.semantic;
            semantic.set_irregular_whitespaces(ret.irregular_whitespaces);
            let module_record = Arc::new(ModuleRecord::new(path, &ret.module_record, &semantic));
            let result = self.linter.run(path, Rc::new(semantic), module_record);

            let reports = result
                .into_iter()
                .map(|msg| {
                    let fixed_content = msg.fix.map(|f| FixedContent {
                        code: f.content.to_string(),
                        range: Range {
                            start: offset_to_position(
                                (f.span.start + start) as usize,
                                source_text.as_str(),
                            ),
                            end: offset_to_position(
                                (f.span.end + start) as usize,
                                source_text.as_str(),
                            ),
                        },
                    });

                    ErrorReport { error: Error::from(msg.error), fixed_content }
                })
                .collect::<Vec<ErrorReport>>();
            diagnostics.extend(Self::wrap_diagnostics(path, &source_text, reports, start));
        }

        Some(diagnostics)
    }

    fn should_lint_path(path: &Path) -> bool {
        static WANTED_EXTENSIONS: OnceLock<FxHashSet<&'static str>> = OnceLock::new();
        let wanted_exts = WANTED_EXTENSIONS.get_or_init(|| {
            VALID_EXTENSIONS.iter().chain(LINT_PARTIAL_LOADER_EXT.iter()).copied().collect()
        });

        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| wanted_exts.contains(ext))
    }

    fn wrap_diagnostics(
        path: &Path,
        source_text: &str,
        reports: Vec<ErrorReport>,
        start: u32,
    ) -> Vec<ErrorWithPosition> {
        let source = Arc::new(NamedSource::new(path.to_string_lossy(), source_text.to_owned()));

        reports
            .into_iter()
            .map(|report| {
                ErrorWithPosition::new(
                    report.error.with_source_code(Arc::clone(&source)),
                    source_text,
                    report.fixed_content,
                    start as usize,
                )
            })
            .collect()
    }
}
