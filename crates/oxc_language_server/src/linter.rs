use log::debug;
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::options::LintOptions;
use miette::NamedSource;
use oxc_allocator::Allocator;
use oxc_diagnostics::{miette, Error, Severity};
use oxc_linter::{partial_loader::LINT_PARTIAL_LOADER_EXT, LintContext, LintSettings, Linter};
use oxc_linter_plugin::{make_relative_path_parts, LinterPlugin};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use ropey::Rope;
use tower_lsp::lsp_types::{
    self, DiagnosticRelatedInformation, DiagnosticSeverity, Position, Range, Url,
};

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

        let message = self.miette_err.help().map_or_else(
            || self.miette_err.to_string(),
            |help| format!("{}\nhelp: {}", self.miette_err, help),
        );

        lsp_types::Diagnostic {
            range,
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

type Plugin = Arc<RwLock<Option<LinterPlugin>>>;

#[derive(Debug)]
pub struct IsolatedLintHandler {
    #[allow(unused)]
    options: Arc<LintOptions>,
    linter: Arc<Linter>,
    #[allow(unused)]
    plugin: Plugin,
}

impl IsolatedLintHandler {
    pub fn new(options: Arc<LintOptions>, linter: Arc<Linter>, plugin: Plugin) -> Self {
        Self { options, linter, plugin }
    }

    pub fn run_single(
        &self,
        path: &Path,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        if Self::is_wanted_ext(path) {
            Some(Self::lint_path(&self.linter, path, Arc::clone(&self.plugin), content).map_or(
                vec![],
                |(p, errors)| {
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
                                    source: Some("oxc".into()),
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
                },
            ))
        } else {
            None
        }
    }

    fn is_wanted_ext(path: &Path) -> bool {
        let extensions = get_extensions();
        path.extension().map_or(false, |ext| extensions.contains(&ext.to_string_lossy().as_ref()))
    }

    fn get_source_type_and_text(
        path: &Path,
        source_text: Option<String>,
        ext: &str,
    ) -> Option<(SourceType, String)> {
        let source_type = SourceType::from_path(path);
        let not_supported_yet =
            source_type.as_ref().is_err_and(|_| !LINT_PARTIAL_LOADER_EXT.contains(&ext));
        if not_supported_yet {
            debug!("extension {ext} not supported yet.");
            return None;
        }
        let source_type = source_type.unwrap_or_default();
        let source_text = source_text.map_or_else(
            || fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path:?}")),
            |source_text| source_text,
        );

        Some((source_type, source_text))
    }

    fn may_need_extract_js_content<'a>(
        _source_text: &'a str,
        _ext: &str,
    ) -> Option<(&'a str, SourceType)> {
        None
        // match ext {
        // "vue" => PartialLoader::Vue.build(source_text),
        // "astro" => PartialLoader::Astro.build(source_text)),
        // _ => None,
        // }
    }

    fn lint_path(
        linter: &Linter,
        path: &Path,
        plugin: Plugin,
        source_text: Option<String>,
    ) -> Option<(PathBuf, Vec<ErrorWithPosition>)> {
        let ext = path.extension().and_then(std::ffi::OsStr::to_str)?;
        let (source_type, source_text) = Self::get_source_type_and_text(path, source_text, ext)?;
        let (source_text, source_type) = Self::may_need_extract_js_content(&source_text, ext)
            .unwrap_or((&source_text, source_type));

        debug!("lint {path:?}");

        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(true)
            .parse();

        if !ret.errors.is_empty() {
            let reports = ret
                .errors
                .into_iter()
                .map(|diagnostic| ErrorReport { error: diagnostic, fixed_content: None })
                .collect();

            return Some(Self::wrap_diagnostics(path, source_text, reports));
        };

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .with_check_syntax_error(true)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            let reports = semantic_ret
                .errors
                .into_iter()
                .map(|diagnostic| ErrorReport { error: diagnostic, fixed_content: None })
                .collect();
            return Some(Self::wrap_diagnostics(path, source_text, reports));
        };

        let mut lint_ctx = LintContext::new(
            path.to_path_buf().into_boxed_path(),
            &Rc::new(semantic_ret.semantic),
            LintSettings::default(),
        );
        {
            if let Ok(guard) = plugin.read() {
                if let Some(plugin) = &*guard {
                    plugin
                        .lint_file(&mut lint_ctx, make_relative_path_parts(&path.into()))
                        .unwrap();
                }
            }
        }

        drop(plugin); // explicitly drop plugin so that we consume the plugin in this function's body

        let result = linter.run(lint_ctx);

        if result.is_empty() {
            return None;
        }

        if linter.options().fix {
            let reports = result
                .into_iter()
                .map(|msg| {
                    let fixed_content = msg.fix.map(|f| FixedContent {
                        code: f.content.to_string(),
                        range: Range {
                            start: offset_to_position(f.span.start as usize, source_text)
                                .unwrap_or_default(),
                            end: offset_to_position(f.span.end as usize, source_text)
                                .unwrap_or_default(),
                        },
                    });

                    ErrorReport { error: msg.error, fixed_content }
                })
                .collect::<Vec<ErrorReport>>();

            return Some(Self::wrap_diagnostics(path, source_text, reports));
        }

        let errors = result
            .into_iter()
            .map(|diagnostic| ErrorReport { error: diagnostic.error, fixed_content: None })
            .collect();
        Some(Self::wrap_diagnostics(path, source_text, errors))
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

fn get_extensions() -> Vec<&'static str> {
    VALID_EXTENSIONS
        .iter()
        .chain(LINT_PARTIAL_LOADER_EXT.iter())
        .copied()
        .collect::<Vec<&'static str>>()
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
    plugin: Plugin,
}

impl ServerLinter {
    pub fn new() -> Self {
        let linter = Linter::new().with_fix(true);
        Self { linter: Arc::new(linter), plugin: Arc::new(RwLock::new(None)) }
    }

    pub fn make_plugin(&self, root_uri: &Url) {
        let mut path = root_uri.to_file_path().unwrap();
        path.push(".oxc/");
        path.push("plugins");
        if path.exists() {
            let mut plugin = self.plugin.write().unwrap();
            plugin.replace(LinterPlugin::new(&path).unwrap());
        }
    }

    pub fn run_single(
        &self,
        root_uri: &Url,
        uri: &Url,
        content: Option<String>,
    ) -> Option<Vec<DiagnosticReport>> {
        let options = LintOptions {
            paths: vec![root_uri.to_file_path().unwrap()],
            ignore_path: "node_modules".into(),
            ignore_pattern: vec!["!**/node_modules/**/*".into()],
            fix: true,
            ..LintOptions::default()
        };

        IsolatedLintHandler::new(
            Arc::new(options),
            Arc::clone(&self.linter),
            Arc::clone(&self.plugin),
        )
        .run_single(&uri.to_file_path().unwrap(), content)
    }
}

fn cmp_range(first: &Range, other: &Range) -> std::cmp::Ordering {
    match first.start.cmp(&other.start) {
        std::cmp::Ordering::Equal => first.end.cmp(&other.end),
        o => o,
    }
}
