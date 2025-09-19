use std::{borrow::Cow, str::FromStr};

use tower_lsp_server::lsp_types::{
    self, CodeDescription, DiagnosticRelatedInformation, DiagnosticSeverity, NumberOrString,
    Position, Range, Uri,
};

use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_diagnostics::{OxcCode, OxcDiagnostic, Severity};
use oxc_linter::{Fix, Message, PossibleFixes};
use oxc_span::GetSpan;

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub enum PossibleFixContent {
    #[default]
    None,
    Single(FixedContent),
    Multiple(Vec<FixedContent>),
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

    let range = message.labels.as_ref().map_or(Range::default(), |labels| {
        let start = labels.first().map(SpanPositionMessage::start).cloned().unwrap_or_default();
        let end = labels.first().map(SpanPositionMessage::end).cloned().unwrap_or_default();
        Range {
            start: Position::new(start.line, start.character),
            end: Position::new(end.line, end.character),
        }
    });
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

pub fn generate_inverted_diagnostics(
    diagnostics: &[DiagnosticReport],
    uri: &Uri,
) -> Vec<DiagnosticReport> {
    let mut inverted_diagnostics = vec![];
    for d in diagnostics {
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
            });
        }
    }
    inverted_diagnostics
}

#[derive(Clone, Debug)]
pub struct SpanPositionMessage<'a> {
    /// A brief suggestion message describing the fix. Will be shown in
    /// editors via code actions.
    message: Option<Cow<'a, str>>,

    start: SpanPosition,
    end: SpanPosition,
}

impl<'a> SpanPositionMessage<'a> {
    pub fn new(start: SpanPosition, end: SpanPosition) -> Self {
        Self { start, end, message: None }
    }

    #[must_use]
    pub fn with_message(mut self, message: Option<Cow<'a, str>>) -> Self {
        self.message = message;
        self
    }

    pub fn start(&self) -> &SpanPosition {
        &self.start
    }

    pub fn end(&self) -> &SpanPosition {
        &self.end
    }

    pub fn message(&self) -> Option<&Cow<'a, str>> {
        self.message.as_ref()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpanPosition {
    pub line: u32,
    pub character: u32,
}

impl SpanPosition {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, character: column }
    }
}

pub fn offset_to_position(rope: &Rope, offset: u32, source_text: &str) -> SpanPosition {
    let (line, column) = get_line_column(rope, offset, source_text);
    SpanPosition::new(line, column)
}

#[derive(Debug)]
pub struct MessageWithPosition<'a> {
    pub message: Cow<'a, str>,
    pub labels: Option<Vec<SpanPositionMessage<'a>>>,
    pub help: Option<Cow<'a, str>>,
    pub severity: Severity,
    pub code: OxcCode,
    pub url: Option<Cow<'a, str>>,
    pub fixes: PossibleFixesWithPosition<'a>,
}

// clippy: the source field is checked and assumed to be less than 4GB, and
// we assume that the fix offset will not exceed 2GB in either direction
#[expect(clippy::cast_possible_truncation)]
pub fn oxc_diagnostic_to_message_with_position<'a>(
    diagnostic: OxcDiagnostic,
    source_text: &str,
    rope: &Rope,
) -> MessageWithPosition<'a> {
    let inner = diagnostic.inner_owned();

    let labels = inner.labels.as_ref().map(|labels| {
        labels
            .iter()
            .map(|labeled_span| {
                let offset = labeled_span.offset() as u32;
                let start_position = offset_to_position(rope, offset, source_text);
                let end_position =
                    offset_to_position(rope, offset + labeled_span.len() as u32, source_text);
                let message = labeled_span.label().map(|label| Cow::Owned(label.to_string()));

                SpanPositionMessage::new(start_position, end_position).with_message(message)
            })
            .collect::<Vec<_>>()
    });

    MessageWithPosition {
        message: inner.message,
        severity: inner.severity,
        help: inner.help,
        url: inner.url,
        code: inner.code,
        labels,
        fixes: PossibleFixesWithPosition::None,
    }
}

pub fn message_to_message_with_position<'a>(
    message: Message<'a>,
    source_text: &str,
    rope: &Rope,
) -> MessageWithPosition<'a> {
    let code = message.error.code.clone();
    let error_offset = message.span().start;
    let section_offset = message.section_offset;

    let mut result = oxc_diagnostic_to_message_with_position(message.error, source_text, rope);
    let fixes = match &message.fixes {
        PossibleFixes::None => PossibleFixesWithPosition::None,
        PossibleFixes::Single(fix) => {
            PossibleFixesWithPosition::Single(fix_to_fix_with_position(fix, rope, source_text))
        }
        PossibleFixes::Multiple(fixes) => PossibleFixesWithPosition::Multiple(
            fixes.iter().map(|fix| fix_to_fix_with_position(fix, rope, source_text)).collect(),
        ),
    };

    result.fixes = add_ignore_fixes(fixes, &code, error_offset, section_offset, rope, source_text);

    result
}

/// Possible fixes with position information.
///
/// This is similar to `PossibleFixes` but with position information.
/// It also includes "ignore this line" and "ignore this rule" fixes for the Language Server.
///
/// The struct should be build with `message_to_message_with_position`
/// or `oxc_diagnostic_to_message_with_position` function to ensure the ignore fixes are added correctly.
#[derive(Debug)]
pub enum PossibleFixesWithPosition<'a> {
    // No possible fixes.
    // This happens on parser/semantic errors.
    None,
    // A single possible fix.
    // This happens when a unused disable directive is reported.
    Single(FixWithPosition<'a>),
    // Multiple possible fixes.
    // This happens when a lint reports a violation, then ignore fixes are added.
    Multiple(Vec<FixWithPosition<'a>>),
}

#[derive(Debug)]
pub struct FixWithPosition<'a> {
    pub content: Cow<'a, str>,
    pub span: SpanPositionMessage<'a>,
}

fn fix_to_fix_with_position<'a>(
    fix: &Fix<'a>,
    rope: &Rope,
    source_text: &str,
) -> FixWithPosition<'a> {
    let start_position = offset_to_position(rope, fix.span.start, source_text);
    let end_position = offset_to_position(rope, fix.span.end, source_text);
    FixWithPosition {
        content: fix.content.clone(),
        span: SpanPositionMessage::new(start_position, end_position)
            .with_message(fix.message.as_ref().map(|label| Cow::Owned(label.to_string()))),
    }
}

/// Add "ignore this line" and "ignore this rule" fixes to the existing fixes.
/// These fixes will be added to the end of the existing fixes.
/// If the existing fixes already contain an "remove unused disable directive" fix,
/// then no ignore fixes will be added.
fn add_ignore_fixes<'a>(
    fixes: PossibleFixesWithPosition<'a>,
    code: &OxcCode,
    error_offset: u32,
    section_offset: u32,
    rope: &Rope,
    source_text: &str,
) -> PossibleFixesWithPosition<'a> {
    // do not append ignore code actions when the error is the ignore action
    if matches!(fixes, PossibleFixesWithPosition::Single(ref fix) if fix.span.message.as_ref().is_some_and(|message| message.starts_with("remove unused disable directive")))
    {
        return fixes;
    }

    let mut new_fixes: Vec<FixWithPosition<'a>> = vec![];
    if let PossibleFixesWithPosition::Single(fix) = fixes {
        new_fixes.push(fix);
    } else if let PossibleFixesWithPosition::Multiple(existing_fixes) = fixes {
        new_fixes.extend(existing_fixes);
    }

    if let Some(rule_name) = code.number.as_ref() {
        // TODO:  doesn't support disabling multiple rules by name for a given line.
        new_fixes.push(disable_for_this_line(rule_name, error_offset, rope, source_text));
        new_fixes.push(disable_for_this_section(rule_name, section_offset, rope, source_text));
    }

    if new_fixes.is_empty() {
        PossibleFixesWithPosition::None
    } else if new_fixes.len() == 1 {
        PossibleFixesWithPosition::Single(new_fixes.remove(0))
    } else {
        PossibleFixesWithPosition::Multiple(new_fixes)
    }
}

fn disable_for_this_line<'a>(
    rule_name: &str,
    error_offset: u32,
    rope: &Rope,
    source_text: &str,
) -> FixWithPosition<'a> {
    let mut start_position = offset_to_position(rope, error_offset, source_text);
    start_position.character = 0; // TODO: character should be set to match the first non-whitespace character in the source text to match the existing indentation.
    let end_position = start_position.clone();
    FixWithPosition {
        content: Cow::Owned(format!("// oxlint-disable-next-line {rule_name}\n")),
        span: SpanPositionMessage::new(start_position, end_position)
            .with_message(Some(Cow::Owned(format!("Disable {rule_name} for this line")))),
    }
}

fn disable_for_this_section<'a>(
    rule_name: &str,
    section_offset: u32,
    rope: &Rope,
    source_text: &str,
) -> FixWithPosition<'a> {
    let comment = format!("// oxlint-disable {rule_name}\n");

    let (content, offset) = if section_offset == 0 {
        // JS files - insert at the beginning
        (Cow::Owned(comment), section_offset)
    } else {
        // Framework files - check for line breaks at section_offset
        let bytes = source_text.as_bytes();
        let current = bytes.get(section_offset as usize);
        let next = bytes.get((section_offset + 1) as usize);

        match (current, next) {
            (Some(b'\n'), _) => {
                // LF at offset, insert after it
                (Cow::Owned(comment), section_offset + 1)
            }
            (Some(b'\r'), Some(b'\n')) => {
                // CRLF at offset, insert after both
                (Cow::Owned(comment), section_offset + 2)
            }
            _ => {
                // Not at line start, prepend newline
                (Cow::Owned("\n".to_owned() + &comment), section_offset)
            }
        }
    };

    let position = offset_to_position(rope, offset, source_text);

    FixWithPosition {
        content,
        span: SpanPositionMessage::new(position.clone(), position)
            .with_message(Some(Cow::Owned(format!("Disable {rule_name} for this file")))),
    }
}

#[cfg(test)]
mod test {
    use oxc_data_structures::rope::Rope;

    use super::offset_to_position;

    #[test]
    fn single_line() {
        let source = "foo.bar!;";
        assert_position(source, 0, (0, 0));
        assert_position(source, 4, (0, 4));
        assert_position(source, 9, (0, 9));
    }

    #[test]
    fn multi_line() {
        let source = "console.log(\n  foo.bar!\n);";
        assert_position(source, 0, (0, 0));
        assert_position(source, 12, (0, 12));
        assert_position(source, 13, (1, 0));
        assert_position(source, 23, (1, 10));
        assert_position(source, 24, (2, 0));
        assert_position(source, 26, (2, 2));
    }

    #[test]
    fn multi_byte() {
        let source = "let foo = \n  '👍';";
        assert_position(source, 10, (0, 10));
        assert_position(source, 11, (1, 0));
        assert_position(source, 14, (1, 3));
        assert_position(source, 18, (1, 5));
        assert_position(source, 19, (1, 6));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn out_of_bounds() {
        offset_to_position(&Rope::from_str("foo"), 100, "foo");
    }

    #[test]
    fn disable_for_section_js_file() {
        let source = "console.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 0, &rope, source);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span.start.line, 0);
        assert_eq!(fix.span.start.character, 0);
    }

    #[test]
    fn disable_for_section_after_lf() {
        let source = "<script>\nconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 8, &rope, source);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span.start.line, 1);
        assert_eq!(fix.span.start.character, 0);
    }

    #[test]
    fn disable_for_section_after_crlf() {
        let source = "<script>\r\nconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 8, &rope, source);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span.start.line, 1);
        assert_eq!(fix.span.start.character, 0);
    }

    #[test]
    fn disable_for_section_mid_line() {
        let source = "const x = 5;";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-unused-vars", 6, &rope, source);

        assert_eq!(fix.content, "\n// oxlint-disable no-unused-vars\n");
        assert_eq!(fix.span.start.line, 0);
        assert_eq!(fix.span.start.character, 6);
    }

    fn assert_position(source: &str, offset: u32, expected: (u32, u32)) {
        let position = offset_to_position(&Rope::from_str(source), offset, source);
        assert_eq!(position.line, expected.0);
        assert_eq!(position.character, expected.1);
    }
}
