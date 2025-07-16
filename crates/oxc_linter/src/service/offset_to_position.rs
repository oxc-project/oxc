use std::borrow::Cow;

use oxc_data_structures::rope::{Rope, get_line_column};

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
