use oxc_data_structures::rope::{Rope, get_line_column};
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

impl From<OxcDiagnostic> for MessageWithPosition<'_> {
    fn from(from: OxcDiagnostic) -> Self {
        Self {
            message: from.message.clone(),
            labels: None,
            help: from.help.clone(),
            severity: from.severity,
            code: from.code.clone(),
            url: from.url.clone(),
            fixes: PossibleFixesWithPosition::None,
        }
    }
}

// clippy: the source field is checked and assumed to be less than 4GB, and
// we assume that the fix offset will not exceed 2GB in either direction
#[expect(clippy::cast_possible_truncation)]
pub fn message_to_message_with_position<'a>(
    message: &Message<'a>,
    source_text: &str,
    rope: &Rope,
) -> MessageWithPosition<'a> {
    let labels = message.error.labels.as_ref().map(|labels| {
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
        message: message.error.message.clone(),
        severity: message.error.severity,
        help: message.error.help.clone(),
        url: message.error.url.clone(),
        code: message.error.code.clone(),
        labels,
        fixes: match &message.fixes {
            PossibleFixes::None => PossibleFixesWithPosition::None,
            PossibleFixes::Single(fix) => {
                PossibleFixesWithPosition::Single(fix_to_fix_with_position(fix, rope, source_text))
            }
            PossibleFixes::Multiple(fixes) => PossibleFixesWithPosition::Multiple(
                fixes.iter().map(|fix| fix_to_fix_with_position(fix, rope, source_text)).collect(),
            ),
        },
    }
}

#[derive(Debug)]
pub enum PossibleFixesWithPosition<'a> {
    None,
    Single(FixWithPosition<'a>),
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
    let start_position = offset_to_position(rope, fix.span.start(), source_text);
    let end_position = offset_to_position(rope, fix.span.end(), source_text);
    FixWithPosition {
        content: fix.content.clone(),
        span: SpanPositionMessage::new(start_position, end_position)
            .with_message(fix.message.as_ref().map(|label| Cow::Owned(label.to_string()))),
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

    fn assert_position(source: &str, offset: u32, expected: (u32, u32)) {
        let position = offset_to_position(&Rope::from_str(source), offset, source);
        assert_eq!(position.line, expected.0);
        assert_eq!(position.character, expected.1);
    }
}
