use std::{borrow::Cow, str::FromStr};

use oxc_linter::MessageWithPosition;
use tower_lsp_server::lsp_types::{
    self, CodeDescription, DiagnosticRelatedInformation, NumberOrString, Position, Range, Uri,
};

use oxc_diagnostics::Severity;

#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub diagnostic: lsp_types::Diagnostic,
    pub fixed_content: Option<FixedContent>,
}

#[derive(Debug, Clone)]
pub struct FixedContent {
    pub message: Option<String>,
    pub code: String,
    pub range: Range,
}

fn cmp_range(first: &Range, other: &Range) -> std::cmp::Ordering {
    match first.start.cmp(&other.start) {
        std::cmp::Ordering::Equal => first.end.cmp(&other.end),
        o => o,
    }
}

fn message_with_position_to_lsp_diagnostic(
    message: &MessageWithPosition<'_>,
    uri: &Uri,
) -> lsp_types::Diagnostic {
    let severity = match message.severity {
        Severity::Error => Some(lsp_types::DiagnosticSeverity::ERROR),
        _ => Some(lsp_types::DiagnosticSeverity::WARNING),
    };

    let related_information = message.labels.as_ref().map(|spans| {
        spans
            .iter()
            .map(|span| lsp_types::DiagnosticRelatedInformation {
                location: lsp_types::Location {
                    uri: uri.clone(),
                    range: lsp_types::Range {
                        start: lsp_types::Position {
                            line: span.start().line,
                            character: span.start().character,
                        },
                        end: lsp_types::Position {
                            line: span.end().line,
                            character: span.end().character,
                        },
                    },
                },
                message: span.message().unwrap_or(&Cow::Borrowed("")).to_string(),
            })
            .collect()
    });

    let range = related_information.as_ref().map_or(
        Range {
            start: Position { line: u32::MAX, character: u32::MAX },
            end: Position { line: u32::MAX, character: u32::MAX },
        },
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
    let code = message.code.to_string();
    let code_description =
        message.url.as_ref().map(|url| CodeDescription { href: Uri::from_str(url).ok().unwrap() });
    let message = message.help.as_ref().map_or_else(
        || message.message.to_string(),
        |help| format!("{}\nhelp: {}", message.message, help),
    );

    lsp_types::Diagnostic {
        range,
        severity,
        code: Some(NumberOrString::String(code)),
        message,
        source: Some("oxc".into()),
        code_description,
        related_information,
        tags: None,
        data: None,
    }
}

pub fn message_with_position_to_lsp_diagnostic_report(
    message: &MessageWithPosition<'_>,
    uri: &Uri,
) -> DiagnosticReport {
    DiagnosticReport {
        diagnostic: message_with_position_to_lsp_diagnostic(message, uri),
        fixed_content: message.fix.as_ref().map(|infos| FixedContent {
            message: infos.span.message().map(std::string::ToString::to_string),
            code: infos.content.to_string(),
            range: Range {
                start: Position {
                    line: infos.span.start().line,
                    character: infos.span.start().character,
                },
                end: Position {
                    line: infos.span.end().line,
                    character: infos.span.end().character,
                },
            },
        }),
    }
}
