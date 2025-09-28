use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_span::GetSpan;
use std::borrow::Cow;

use crate::fixer::{Fix, Message, PossibleFixes};
use oxc_diagnostics::{OxcCode, OxcDiagnostic, Severity};

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

#[derive(Clone, Debug)]
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

    PossibleFixesWithPosition::Multiple(new_fixes)
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
    let (content, offset) = if section_offset == 0 {
        // JS files - keep existing behavior
        (Cow::Owned(format!("// oxlint-disable {rule_name}\n")), section_offset)
    }
    // Vue files or other files with section_offset > 0
    // Check if there's a newline at the section offset
    else {
        let bytes = source_text.as_bytes();
        let offset_usize = section_offset as usize;

        let Some(first_char) = bytes.get(offset_usize) else {
            unreachable!("section_offset is out of bounds, there must be a end section marker");
        };

        // handle first new line after section offset
        if *first_char == b'\n' || *first_char == b'\r' {
            let msg = Cow::Owned(format!("// oxlint-disable {rule_name}\n"));
            if *first_char == b'\r' && bytes.get(offset_usize + 1) == Some(&b'\n') {
                (msg, section_offset + 2)
            } else {
                (msg, section_offset + 1)
            }
        } else {
            // Not at beginning of line, add newline before comment
            (Cow::Owned(format!("\n// oxlint-disable {rule_name}\n")), section_offset)
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
        let source = "_\nconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 1, &rope, source);

        assert_eq!(fix.content, "// oxlint-disable no-console\n");
        assert_eq!(fix.span.start.line, 1);
        assert_eq!(fix.span.start.character, 0);
    }

    #[test]
    fn disable_for_section_after_crlf() {
        let source = "_\r\nconsole.log('hello');";
        let rope = Rope::from_str(source);
        let fix = super::disable_for_this_section("no-console", 1, &rope, source);

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
