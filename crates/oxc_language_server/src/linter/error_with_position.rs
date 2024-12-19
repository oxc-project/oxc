use std::{path::PathBuf, str::FromStr};

use tower_lsp::lsp_types::{
    self, CodeDescription, DiagnosticRelatedInformation, NumberOrString, Position, Range, Url,
};

use cow_utils::CowUtils;
use oxc_diagnostics::{Error, Severity};

use crate::linter::offset_to_position;

const LINT_DOC_LINK_PREFIX: &str = "https://oxc.rs/docs/guide/usage/linter/rules";

#[derive(Debug)]
pub struct ErrorWithPosition {
    pub start_pos: Position,
    pub end_pos: Position,
    pub miette_err: Error,
    pub fixed_content: Option<FixedContent>,
    pub labels_with_pos: Vec<LabeledSpanWithPosition>,
}

#[derive(Debug)]
pub struct LabeledSpanWithPosition {
    pub start_pos: Position,
    pub end_pos: Position,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub diagnostic: lsp_types::Diagnostic,
    pub fixed_content: Option<FixedContent>,
}
#[derive(Debug)]
pub struct ErrorReport {
    pub error: Error,
    pub fixed_content: Option<FixedContent>,
}

#[derive(Debug, Clone)]
pub struct FixedContent {
    pub code: String,
    pub range: Range,
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
                start_pos: offset_to_position(labeled_span.offset() + start, text),
                end_pos: offset_to_position(
                    labeled_span.offset() + start + labeled_span.len(),
                    text,
                ),
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
                href: Url::from_str(&format!(
                    "{LINT_DOC_LINK_PREFIX}/{}/{number}",
                    scope.strip_prefix("eslint-plugin-").unwrap_or(scope).cow_replace("-", "_")
                ))
                .ok()?,
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

    pub fn into_diagnostic_report(self, path: &PathBuf) -> DiagnosticReport {
        DiagnosticReport {
            diagnostic: self.to_lsp_diagnostic(path),
            fixed_content: self.fixed_content,
        }
    }
}
