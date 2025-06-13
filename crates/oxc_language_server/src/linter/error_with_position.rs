use std::{borrow::Cow, str::FromStr};

use oxc_linter::{FixWithPosition, MessageWithPosition, PossibleFixesWithPosition};
use tower_lsp_server::lsp_types::{
    self, CodeDescription, DiagnosticRelatedInformation, NumberOrString, Position, Range, Uri,
};

use oxc_diagnostics::Severity;

// max range for LSP integer is 2^31 - 1
// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#baseTypes
const LSP_MAX_INT: u32 = 2u32.pow(31) - 1;

#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub diagnostic: lsp_types::Diagnostic,
    pub fixed_content: PossibleFixContent,
}

#[derive(Debug, Clone)]
pub struct FixedContent {
    pub message: Option<String>,
    pub code: String,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub enum PossibleFixContent {
    None,
    Single(FixedContent),
    Multiple(Vec<FixedContent>),
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
                message: span
                    .message()
                    .map_or_else(String::new, |message| message.clone().into_owned()),
            })
            .collect()
    });

    let range = related_information.as_ref().map_or(
        Range {
            start: Position { line: LSP_MAX_INT, character: LSP_MAX_INT },
            end: Position { line: LSP_MAX_INT, character: LSP_MAX_INT },
        },
        |infos: &Vec<DiagnosticRelatedInformation>| {
            let mut ret_range = Range {
                start: Position { line: LSP_MAX_INT, character: LSP_MAX_INT },
                end: Position { line: LSP_MAX_INT, character: LSP_MAX_INT },
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
    let message = match &message.help {
        Some(help) => {
            let mut msg = String::with_capacity(message.message.len() + help.len() + 7);
            msg.push_str(message.message.as_ref());
            msg.push_str("\nhelp: ");
            msg.push_str(help.as_ref());
            msg
        }
        None => match message.message {
            Cow::Borrowed(s) => s.to_string(),
            Cow::Owned(ref s) => s.clone(),
        },
    };

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

fn fix_with_position_to_fix_content(fix: &FixWithPosition<'_>) -> FixedContent {
    FixedContent {
        message: fix.span.message().map(std::string::ToString::to_string),
        code: fix.content.to_string(),
        range: Range {
            start: Position { line: fix.span.start().line, character: fix.span.start().character },
            end: Position { line: fix.span.end().line, character: fix.span.end().character },
        },
    }
}

pub fn message_with_position_to_lsp_diagnostic_report(
    message: &MessageWithPosition<'_>,
    uri: &Uri,
) -> DiagnosticReport {
    DiagnosticReport {
        diagnostic: message_with_position_to_lsp_diagnostic(message, uri),
        fixed_content: match &message.fixes {
            PossibleFixesWithPosition::None => PossibleFixContent::None,
            PossibleFixesWithPosition::Single(fix) => {
                PossibleFixContent::Single(fix_with_position_to_fix_content(fix))
            }
            PossibleFixesWithPosition::Multiple(fixes) => PossibleFixContent::Multiple(
                fixes.iter().map(fix_with_position_to_fix_content).collect(),
            ),
        },
    }
}
