use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
};

use crate::options::LintOptions;
use crate::walk::Walk;
use miette::NamedSource;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self},
    Error, Severity,
};
use oxc_linter::{LintContext, Linter, RuleCategory, RULES};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use ropey::Rope;
use tower_lsp::lsp_types::{self, Position, Range, Url};

#[derive(Debug)]
struct ErrorWithPosition {
    pub start_pos: Position,
    pub end_pos: Position,
    pub miette_err: Error,
    pub fixed_content: Option<FixedContent>,
    pub labels_with_pos: Vec<LabeledSpanWithPosition>,
}

#[derive(Debug)]
struct LabeledSpanWithPosition {
    pub start_pos: Position,
    pub end_pos: Position,
    pub message: Option<String>,
}

impl ErrorWithPosition {
    pub fn new(error: Error, text: &str, fixed_content: Option<FixedContent>) -> Self {
        let labels = error.labels().map_or(vec![], Iterator::collect);

        let labels_with_pos: Vec<LabeledSpanWithPosition> = labels
            .iter()
            .map(|labeled_span| LabeledSpanWithPosition {
                start_pos: offset_to_position(labeled_span.offset(), text).unwrap_or_default(),
                end_pos: offset_to_position(labeled_span.offset() + labeled_span.len(), text)
                    .unwrap_or_default(),
                message: labeled_span.label().map(ToString::to_string),
            })
            .collect();

        let start_pos = labels_with_pos[0].start_pos;
        let end_pos = labels_with_pos[labels_with_pos.len() - 1].end_pos;

        Self { miette_err: error, start_pos, end_pos, labels_with_pos, fixed_content }
    }

    fn to_lsp_diagnostic(&self, path: &PathBuf) -> lsp_types::Diagnostic {
        let severity = match self.miette_err.severity() {
            Some(Severity::Error) => Some(lsp_types::DiagnosticSeverity::ERROR),
            Some(Severity::Warning) => Some(lsp_types::DiagnosticSeverity::WARNING),
            _ => Some(lsp_types::DiagnosticSeverity::INFORMATION),
        };

        let related_information = Some(
            self.labels_with_pos
                .iter()
                .map(|labeled_span| lsp_types::DiagnosticRelatedInformation {
                    location: lsp_types::Location {
                        uri: lsp_types::Url::from_file_path(path).unwrap(),
                        range: lsp_types::Range {
                            start: lsp_types::Position {
                                line: labeled_span.start_pos.line,
                                character: labeled_span.start_pos.character,
                            },
                            end: lsp_types::Position {
                                line: labeled_span.end_pos.line,
                                character: labeled_span.end_pos.character,
                            },
                        },
                    },
                    message: labeled_span.message.clone().unwrap_or_default(),
                })
                .collect(),
        );

        let message = self.miette_err.help().map_or_else(
            || self.miette_err.to_string(),
            |help| format!("{}\nhelp: {}", self.miette_err, help),
        );

        lsp_types::Diagnostic {
            range: Range { start: self.start_pos, end: self.end_pos },
            severity,
            code: None,
            message,
            source: Some("oxc".into()),
            code_description: None,
            related_information,
            tags: None,
            data: None,
        }
    }

    fn into_diagnostic_report(self, path: &PathBuf) -> DiagnosticReport {
        DiagnosticReport {
            diagnostic: self.to_lsp_diagnostic(path),
            fixed_content: self.fixed_content,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub diagnostic: lsp_types::Diagnostic,
    pub fixed_content: Option<FixedContent>,
}

#[derive(Debug)]
struct ErrorReport {
    pub error: Error,
    pub fixed_content: Option<FixedContent>,
}

#[derive(Debug, Clone)]
pub struct FixedContent {
    pub code: String,
    pub range: Range,
}

#[derive(Debug)]
pub struct IsolatedLintHandler {
    options: Arc<LintOptions>,
    linter: Arc<Linter>,
}

impl IsolatedLintHandler {
    pub fn new(options: Arc<LintOptions>, linter: Arc<Linter>) -> Self {
        Self { options, linter }
    }

    /// # Panics
    ///
    /// * When `mpsc::channel` fails to send.
    pub fn run_full(&self) -> Vec<(PathBuf, Vec<DiagnosticReport>)> {
        let number_of_files = Arc::new(AtomicUsize::new(0));
        let (tx_error, rx_error) = mpsc::channel::<(PathBuf, Vec<ErrorWithPosition>)>();

        self.process_paths(&number_of_files, tx_error);
        Self::process_diagnostics(&rx_error)
    }

    pub fn run_single(&self, path: &Path) -> Option<Vec<DiagnosticReport>> {
        if Self::is_wanted_ext(path) {
            Some(Self::lint_path(&self.linter, path).map_or(vec![], |(p, errors)| {
                errors.into_iter().map(|e| e.into_diagnostic_report(&p)).collect()
            }))
        } else {
            None
        }
    }

    fn is_wanted_ext(path: &Path) -> bool {
        path.extension()
            .map_or(false, |ext| VALID_EXTENSIONS.contains(&ext.to_string_lossy().as_ref()))
    }

    fn process_paths(
        &self,
        number_of_files: &Arc<AtomicUsize>,
        tx_error: mpsc::Sender<(PathBuf, Vec<ErrorWithPosition>)>,
    ) {
        let (tx_path, rx_path) = mpsc::channel::<Box<Path>>();

        let walk = Walk::new(&self.options);
        let number_of_files = Arc::clone(number_of_files);
        rayon::spawn(move || {
            let mut count = 0;
            walk.iter().for_each(|path| {
                count += 1;
                tx_path.send(path).unwrap();
            });
            number_of_files.store(count, Ordering::Relaxed);
        });

        let linter = Arc::clone(&self.linter);
        rayon::spawn(move || {
            while let Ok(path) = rx_path.recv() {
                let tx_error = tx_error.clone();
                let linter = Arc::clone(&linter);
                rayon::spawn(move || {
                    if let Some(diagnostics) = Self::lint_path(&linter, &path) {
                        tx_error.send(diagnostics).unwrap();
                    }
                    drop(tx_error);
                });
            }
        });
    }

    fn process_diagnostics(
        rx_error: &mpsc::Receiver<(PathBuf, Vec<ErrorWithPosition>)>,
    ) -> Vec<(PathBuf, Vec<DiagnosticReport>)> {
        rx_error
            .iter()
            .map(|(path, errors)| {
                (
                    path.clone(),
                    errors.into_iter().map(|e| e.into_diagnostic_report(&path)).collect(),
                )
            })
            .collect()
    }

    fn lint_path(linter: &Linter, path: &Path) -> Option<(PathBuf, Vec<ErrorWithPosition>)> {
        let source_text =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path:?}"));
        let allocator = Allocator::default();
        let source_type =
            SourceType::from_path(path).unwrap_or_else(|_| panic!("Incorrect {path:?}"));
        let ret = Parser::new(&allocator, &source_text, source_type)
            .allow_return_outside_function(true)
            .parse();

        if !ret.errors.is_empty() {
            let reports = ret
                .errors
                .into_iter()
                .map(|diagnostic| ErrorReport { error: diagnostic, fixed_content: None })
                .collect();

            return Some(Self::wrap_diagnostics(path, &source_text, reports));
        };

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(&source_text, source_type)
            .with_trivias(&ret.trivias)
            .with_check_syntax_error(true)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            let reports = semantic_ret
                .errors
                .into_iter()
                .map(|diagnostic| ErrorReport { error: diagnostic, fixed_content: None })
                .collect();
            return Some(Self::wrap_diagnostics(path, &source_text, reports));
        };

        let lint_ctx = LintContext::new(&Rc::new(semantic_ret.semantic));
        let result = linter.run(lint_ctx);

        if result.is_empty() {
            return None;
        }

        if linter.has_fix() {
            let reports = result
                .into_iter()
                .map(|msg| {
                    let fixed_content = msg.fix.map(|f| FixedContent {
                        code: f.content.to_string(),
                        range: Range {
                            start: offset_to_position(f.span.start as usize, &source_text)
                                .unwrap_or_default(),
                            end: offset_to_position(f.span.end as usize, &source_text)
                                .unwrap_or_default(),
                        },
                    });

                    ErrorReport { error: msg.error, fixed_content }
                })
                .collect::<Vec<ErrorReport>>();

            return Some(Self::wrap_diagnostics(path, &source_text, reports));
        }

        let errors = result
            .into_iter()
            .map(|diagnostic| ErrorReport { error: diagnostic.error, fixed_content: None })
            .collect();
        Some(Self::wrap_diagnostics(path, &source_text, errors))
    }

    fn wrap_diagnostics(
        path: &Path,
        source_text: &str,
        reports: Vec<ErrorReport>,
    ) -> (PathBuf, Vec<ErrorWithPosition>) {
        let source = Arc::new(NamedSource::new(path.to_string_lossy(), source_text.to_owned()));
        let diagnostics = reports
            .into_iter()
            .map(|report| {
                ErrorWithPosition::new(
                    report.error.with_source_code(Arc::clone(&source)),
                    source_text,
                    report.fixed_content,
                )
            })
            .collect();
        (path.to_path_buf(), diagnostics)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn offset_to_position(offset: usize, source_text: &str) -> Option<Position> {
    let rope = Rope::from_str(source_text);
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}

#[derive(Debug)]
pub struct ServerLinter {
    linter: Arc<Linter>,
}

impl ServerLinter {
    pub fn new() -> Self {
        let linter = Linter::from_rules(
            RULES
                .iter()
                .cloned()
                .filter(|rule| rule.category() != RuleCategory::Nursery)
                .collect::<Vec<_>>(),
        )
        .with_fix(true);

        Self { linter: Arc::new(linter) }
    }

    pub fn run_full(&self, root_uri: &Url) -> Vec<(PathBuf, Vec<DiagnosticReport>)> {
        let options = LintOptions {
            paths: vec![root_uri.to_file_path().unwrap()],
            ignore_path: "node_modules".into(),
            ignore_pattern: vec!["!**/node_modules/**/*".into()],
            fix: true,
            ..LintOptions::default()
        };

        IsolatedLintHandler::new(Arc::new(options), Arc::clone(&self.linter)).run_full()
    }

    pub fn run_single(&self, root_uri: &Url, uri: &Url) -> Option<Vec<DiagnosticReport>> {
        let options = LintOptions {
            paths: vec![root_uri.to_file_path().unwrap()],
            ignore_path: "node_modules".into(),
            ignore_pattern: vec!["!**/node_modules/**/*".into()],
            fix: true,
            ..LintOptions::default()
        };

        IsolatedLintHandler::new(Arc::new(options), Arc::clone(&self.linter))
            .run_single(&uri.to_file_path().unwrap())
    }
}
