use oxc_linter::loader::LINT_PARTIAL_LOADER_EXT;
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use log::debug;
use oxc_allocator::Allocator;
use oxc_diagnostics::{Error, NamedSource, Severity};
use oxc_linter::{
    loader::{JavaScriptSource, Loader},
    FixKind, Linter,
};
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_span::VALID_EXTENSIONS;
use ropey::Rope;
use rustc_hash::FxHashSet;
use tower_lsp::lsp_types::{
    self, CodeDescription, DiagnosticRelatedInformation, DiagnosticSeverity, NumberOrString,
    Position, Range, Url,
};

const LINT_DOC_LINK_PREFIX: &str = "https://oxc.rs/docs/guide/usage/linter/rules";
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
    pub fn new(
        error: Error,
        text: &str,
        fixed_content: Option<FixedContent>,
        start: usize,
    ) -> Self {
        let labels = error.labels().map_or(vec![], Iterator::collect);
        let labels_with_pos: Vec<LabeledSpanWithPosition> = labels
            .iter()
            .map(|labeled_span| LabeledSpanWithPosition {
                start_pos: offset_to_position(labeled_span.offset() + start, text)
                    .unwrap_or_default(),
                end_pos: offset_to_position(
                    labeled_span.offset() + start + labeled_span.len(),
                    text,
                )
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
            _ => Some(lsp_types::DiagnosticSeverity::WARNING),
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
        let range = related_information.as_ref().map_or(
            Range { start: self.start_pos, end: self.end_pos },
            |infos: &Vec<DiagnosticRelatedInformation>| {
                let mut ret_range = Range {
                    start: Position { line: u32::MAX, character: u32::MAX },
                    end: Position { line: u32::MAX, character: u32::MAX },
                };
                for info in infos {
                    if cmp_range(&ret_range, &info.location.range) == std::cmp::Ordering::Greater {
                        ret_range = info.location.range;
                    }
                }
                ret_range
            },
        );
        let code = self.miette_err.code().map(|item| item.to_string());
        let code_description = code.as_ref().and_then(|code| {
            let (scope, number) = parse_diagnostic_code(code)?;
            Some(CodeDescription {
                href: Url::from_str(&format!("{LINT_DOC_LINK_PREFIX}/{scope}/{number}")).ok()?,
            })
        });
        let message = self.miette_err.help().map_or_else(
            || self.miette_err.to_string(),
            |help| format!("{}\nhelp: {}", self.miette_err, help),
        );

        lsp_types::Diagnostic {
            range,
            severity,
            code: code.map(NumberOrString::String),
            message,
            source: Some("oxc".into()),
            code_description,
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
        if Self::should_lint_path(path) {
            Some(self.lint_path(path, content).map_or(vec![], |(p, errors)| {
                let mut diagnostics: Vec<DiagnosticReport> =
                    errors.into_iter().map(|e| e.into_diagnostic_report(&p)).collect();
                // a diagnostics connected from related_info to original diagnostic
                let mut inverted_diagnostics = vec![];
                for d in &diagnostics {
                    let Some(ref related_info) = d.diagnostic.related_information else {
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
        } else {
            None
        }
    }

    fn lint_path(
        &self,
        path: &Path,
        source_text: Option<String>,
    ) -> Option<(PathBuf, Vec<ErrorWithPosition>)> {
        if !Loader::can_load(path) {
            debug!("extension not supported yet.");
            return None;
        }
        let source_text = source_text.map_or_else(
            || fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path:?}")),
            |source_text| source_text,
        );
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
            let result = self.linter.run(path, Rc::new(semantic));

            let reports = result
                .into_iter()
                .map(|msg| {
                    let fixed_content = msg.fix.map(|f| FixedContent {
                        code: f.content.to_string(),
                        range: Range {
                            start: offset_to_position(
                                (f.span.start + start) as usize,
                                source_text.as_str(),
                            )
                            .unwrap_or_default(),
                            end: offset_to_position(
                                (f.span.end + start) as usize,
                                source_text.as_str(),
                            )
                            .unwrap_or_default(),
                        },
                    });

                    ErrorReport { error: Error::from(msg.error), fixed_content }
                })
                .collect::<Vec<ErrorReport>>();
            let (_, errors_with_position) =
                Self::wrap_diagnostics(path, &source_text, reports, start);
            diagnostics.extend(errors_with_position);
        }

        Some((path.to_path_buf(), diagnostics))
    }

    fn should_lint_path(path: &Path) -> bool {
        static WANTED_EXTENSIONS: OnceLock<FxHashSet<&'static str>> = OnceLock::new();
        let wanted_exts = WANTED_EXTENSIONS.get_or_init(|| {
            VALID_EXTENSIONS.iter().chain(LINT_PARTIAL_LOADER_EXT.iter()).copied().collect()
        });

        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .map_or(false, |ext| wanted_exts.contains(ext))
    }

    fn wrap_diagnostics(
        path: &Path,
        source_text: &str,
        reports: Vec<ErrorReport>,
        start: u32,
    ) -> (PathBuf, Vec<ErrorWithPosition>) {
        let source = Arc::new(NamedSource::new(path.to_string_lossy(), source_text.to_owned()));
        let diagnostics = reports
            .into_iter()
            .map(|report| {
                ErrorWithPosition::new(
                    report.error.with_source_code(Arc::clone(&source)),
                    source_text,
                    report.fixed_content,
                    start as usize,
                )
            })
            .collect();
        (path.to_path_buf(), diagnostics)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn offset_to_position(offset: usize, source_text: &str) -> Option<Position> {
    let rope = Rope::from_str(source_text);
    let line = rope.try_byte_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    // Original offset is byte, but Rope uses char offset
    let offset = rope.try_byte_to_char(offset).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}

pub struct ServerLinter {
    linter: Arc<Linter>,
}

impl ServerLinter {
    pub fn new() -> Self {
        let linter = Linter::default().with_fix(FixKind::SafeFix);
        Self { linter: Arc::new(linter) }
    }

    pub fn new_with_linter(linter: Linter) -> Self {
        Self { linter: Arc::new(linter) }
    }

    pub fn run_single(&self, uri: &Url, content: Option<String>) -> Option<Vec<DiagnosticReport>> {
        IsolatedLintHandler::new(Arc::clone(&self.linter))
            .run_single(&uri.to_file_path().unwrap(), content)
    }
}

fn cmp_range(first: &Range, other: &Range) -> std::cmp::Ordering {
    match first.start.cmp(&other.start) {
        std::cmp::Ordering::Equal => first.end.cmp(&other.end),
        o => o,
    }
}

/// parse `OxcCode` to `Option<(scope, number)>`
fn parse_diagnostic_code(code: &str) -> Option<(&str, &str)> {
    if !code.ends_with(')') {
        return None;
    }
    let right_parenthesis_pos = code.rfind('(')?;
    Some((&code[0..right_parenthesis_pos], &code[right_parenthesis_pos + 1..code.len() - 1]))
}
