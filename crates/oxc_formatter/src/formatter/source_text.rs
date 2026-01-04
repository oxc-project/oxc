use std::ops::Deref;

use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::is_white_space_single_line,
    line_terminator::{CR, LF, is_line_terminator},
};

use super::Comments;

/// Source text wrapper providing utilities for text analysis in the formatter.
#[derive(Debug, Clone, Copy)]
pub struct SourceText<'a> {
    text: &'a str,
}

impl Deref for SourceText<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.text
    }
}

impl<'a> SourceText<'a> {
    /// Create a new SourceText wrapper
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    /// Extract text for an object that has a span
    pub fn text_for<T: GetSpan>(&self, obj: &T) -> &'a str {
        obj.span().source_text(self.text)
    }

    // Text slicing
    /// Get text from position to end
    pub fn slice_from(&self, position: u32) -> &'a str {
        &self.text[position as usize..]
    }

    /// Get text from start to position
    pub fn slice_to(&self, position: u32) -> &'a str {
        &self.text[..position as usize]
    }

    /// Get text between two positions
    pub fn slice_range(&self, start: u32, end: u32) -> &'a str {
        &self.text[start as usize..end as usize]
    }

    // Byte slicing
    /// Get bytes from position to end
    pub fn bytes_from(&self, position: u32) -> impl Iterator<Item = u8> {
        self.text.as_bytes()[position as usize..].iter().copied()
    }

    /// Get bytes from start to position in reverse
    pub fn bytes_to(&self, position: u32) -> impl Iterator<Item = u8> {
        self.text.as_bytes()[..position as usize].iter().copied().rev()
    }

    /// Get bytes between two positions
    pub fn bytes_range(&self, start: u32, end: u32) -> &'a [u8] {
        &self.text.as_bytes()[start as usize..end as usize]
    }

    // Byte checking
    /// Check if first non-whitespace byte at position matches expected
    pub fn next_non_whitespace_byte_is(&self, position: u32, expected_byte: u8) -> bool {
        self.bytes_from(position)
            .find(|byte| !byte.is_ascii_whitespace())
            .is_some_and(|b| b == expected_byte)
    }

    /// Get first byte at position
    pub fn byte_at(&self, position: u32) -> Option<u8> {
        self.text.as_bytes().get(position as usize).copied()
    }

    // Newline detection
    /// Check if span contains line terminators
    pub fn contains_newline(&self, span: Span) -> bool {
        self.contains_newline_between(span.start, span.end)
    }

    /// Check if range contains line terminators
    pub fn contains_newline_between(&self, start: u32, end: u32) -> bool {
        self.slice_range(start, end).chars().any(is_line_terminator)
    }

    /// Check for newlines before position, stopping at first non-whitespace
    pub fn has_newline_before(&self, position: u32) -> bool {
        for byte in self.bytes_to(position) {
            match byte {
                b'\n' | b'\r' => return true,
                b' ' | b'\t' => {}
                _ => return false,
            }
        }
        false
    }

    /// Check for newlines after position, stopping at first non-whitespace
    pub fn has_newline_after(&self, position: u32) -> bool {
        for byte in self.bytes_from(position) {
            match byte {
                b'\n' | b'\r' => return true,
                b' ' | b'\t' => {}
                _ => return false,
            }
        }
        false
    }

    // Byte range operations
    /// Check if byte range contains specific byte
    pub fn bytes_contain(&self, start: u32, end: u32, byte: u8) -> bool {
        self.bytes_range(start, end).contains(&byte)
    }

    /// Check if all bytes in range match predicate
    pub fn all_bytes_match<F>(&self, start: u32, end: u32, predicate: F) -> bool
    where
        F: Fn(u8) -> bool,
    {
        self.bytes_range(start, end).iter().all(|&b| predicate(b))
    }

    // Utility methods
    /// Get character count of span
    pub fn span_width(&self, span: Span) -> usize {
        self.text_for(&span).chars().count()
    }

    /// Count consecutive line breaks after position, returning `0` if only whitespace follows
    pub fn lines_after(&self, end: u32) -> usize {
        let mut count = 0;
        let mut chars = self.slice_from(end).chars().peekable();
        while let Some(char) = chars.next() {
            if is_white_space_single_line(char) {
                continue;
            }

            if is_line_terminator(char) {
                count += 1;
                if char == CR && chars.peek() == Some(&LF) {
                    chars.next();
                }
                continue;
            }

            return count;
        }

        // No non-whitespace characters found after position, so return `0` to avoid adding extra new lines
        0
    }

    /// Count line breaks between syntax nodes, considering comments and parentheses
    pub fn get_lines_before(&self, span: Span, comments: &Comments) -> usize {
        let mut start = span.start;
        let comments = comments.unprinted_comments();

        if let Some(comment) = comments.first()
            && comment.span.end <= start
        {
            start = comment.span.start;
        } else if start != 0 && matches!(self.byte_at(start - 1), Some(b';')) {
            start -= 1;
        }

        let mut following_source = self.bytes_from(span.end);
        let mut cursor = start as usize;
        let mut skip_cr = false;
        let mut count = 0;

        while cursor > 0 {
            let (ch, next_cursor) = char_before(self.text, cursor);
            cursor = next_cursor;

            if skip_cr && ch == CR {
                skip_cr = false;
                continue;
            }

            if is_white_space_single_line(ch) {
                skip_cr = false;
                continue;
            }

            if is_line_terminator(ch) {
                count += 1;
                skip_cr = ch == LF;
                continue;
            }

            if ch == '(' {
                if skip_wrapping_parenthesis(&mut following_source) {
                    count = 0;
                    skip_cr = false;
                    continue;
                }
                return count;
            }

            return count;
        }

        0
    }
}

fn char_before(text: &str, mut index: usize) -> (char, usize) {
    debug_assert!(index > 0);
    let bytes = text.as_bytes();

    index -= 1;
    while bytes[index] & 0b1100_0000 == 0b1000_0000 {
        index -= 1;
    }

    // SAFETY: `index` always points to the start of a character boundary.
    let ch = unsafe { text.get_unchecked(index..) }.chars().next().unwrap();
    (ch, index)
}

fn skip_wrapping_parenthesis(iter: &mut impl Iterator<Item = u8>) -> bool {
    for c in iter.by_ref() {
        if c.is_ascii_whitespace() {
            continue;
        }

        return c == b')';
    }
    false
}

// NOTE: Our test fixtures are managed under `.gitattributes` to enforce LF line endings.
// Therefore, we explicitly test CRLF and mixed line endings here.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_text() {
        let source_text = r"
const x = 1;

const y = 2;


const z = 3;
"
        .trim();
        let source_text = SourceText::new(source_text);
        let comments = vec![];
        let comments = Comments::new(source_text, &comments);

        let span_x = Span::new(0, 12);
        let span_y = Span::new(14, 26);
        let span_z = Span::new(29, 41);
        assert_eq!(source_text.text_for(&span_x), "const x = 1;");
        assert_eq!(source_text.text_for(&span_y), "const y = 2;");
        assert_eq!(source_text.text_for(&span_z), "const z = 3;");

        assert_eq!(source_text.get_lines_before(span_x, &comments), 0);
        assert_eq!(source_text.get_lines_before(span_y, &comments), 2);
        assert_eq!(source_text.get_lines_before(span_z, &comments), 3);

        assert_eq!(source_text.lines_after(span_x.end), 2);
        assert_eq!(source_text.lines_after(span_y.end), 3);
        assert_eq!(source_text.lines_after(span_z.end), 0);
    }

    #[test]
    fn test_source_text_with_crlf() {
        let source_text = "const x = 1;\r\n\r\nconst y = 2;\r\n\r\n\r\nconst z = 3;";
        let source_text = SourceText::new(source_text);
        let comments = vec![];
        let comments = Comments::new(source_text, &comments);

        let span_x = Span::new(0, 12);
        let span_y = Span::new(16, 28);
        let span_z = Span::new(34, 46);
        assert_eq!(source_text.text_for(&span_x), "const x = 1;");
        assert_eq!(source_text.text_for(&span_y), "const y = 2;");
        assert_eq!(source_text.text_for(&span_z), "const z = 3;");

        assert_eq!(source_text.get_lines_before(span_y, &comments), 2);
        assert_eq!(source_text.get_lines_before(span_z, &comments), 3);

        assert_eq!(source_text.lines_after(span_x.end), 2);
        assert_eq!(source_text.lines_after(span_y.end), 3);
    }

    #[test]
    fn test_source_text_with_mixed_line_endings() {
        let source_text = "const x = 1;\n\r\nconst y = 2;\r\n\nconst z = 3;";
        let source_text = SourceText::new(source_text);
        let comments = vec![];
        let comments = Comments::new(source_text, &comments);

        let span_x = Span::new(0, 12);
        let span_y = Span::new(15, 27);
        let span_z = Span::new(30, 42);
        assert_eq!(source_text.text_for(&span_x), "const x = 1;");
        assert_eq!(source_text.text_for(&span_y), "const y = 2;");
        assert_eq!(source_text.text_for(&span_z), "const z = 3;");

        assert_eq!(source_text.get_lines_before(span_y, &comments), 2);
        assert_eq!(source_text.get_lines_before(span_z, &comments), 2);

        assert_eq!(source_text.lines_after(span_x.end), 2);
        assert_eq!(source_text.lines_after(span_y.end), 2);
    }

    fn empty_comments(source_text: SourceText<'_>) -> Comments<'_> {
        Comments::new(source_text, &[])
    }

    #[test]
    fn test_get_lines_before_handles_utf8() {
        let source = "const 😀 = 1;\n\nconst β = 2;\nconst γ = 3;";
        let source_text = SourceText::new(source);
        let comments = empty_comments(source_text);

        let beta_start = u32::try_from(source.find("const β").unwrap()).unwrap();
        let gamma_start = u32::try_from(source.find("const γ").unwrap()).unwrap();

        assert_eq!(
            source_text.get_lines_before(Span::new(beta_start, beta_start + 6), &comments),
            2
        );
        assert_eq!(
            source_text.get_lines_before(Span::new(gamma_start, gamma_start + 6), &comments),
            1
        );
    }

    #[test]
    fn test_get_lines_before_skips_wrapping_parentheses() {
        let source = "(\n\nfoo\n)\n\nbar";
        let source_text = SourceText::new(source);
        let comments = empty_comments(source_text);

        let foo_start = u32::try_from(source.find("foo").unwrap()).unwrap();
        let bar_start = u32::try_from(source.find("bar").unwrap()).unwrap();

        // The node wrapped inside parentheses should not inherit the blank lines from within.
        assert_eq!(source_text.get_lines_before(Span::new(foo_start, foo_start + 3), &comments), 0);
        // The node outside the parentheses should still see the two preceding blank lines.
        assert_eq!(source_text.get_lines_before(Span::new(bar_start, bar_start + 3), &comments), 2);
    }
}
