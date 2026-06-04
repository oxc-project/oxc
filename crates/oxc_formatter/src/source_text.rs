//! JS/TS source-text scanning that depends on ECMAScript lexical rules.
//!
//! These extend the language-agnostic [`oxc_formatter_core::SourceText`] (which owns only
//! mechanical byte/offset access) with the JS/TS-specific lexis:
//! line terminators including U+2028/U+2029, ASI semicolons, and parentheses trivia.
//! They live here, not in core, because "what counts as a newline / trivia" is language-defined.

use oxc_formatter_core::SourceText;
use oxc_span::Span;
use oxc_syntax::{
    identifier::is_white_space_single_line,
    line_terminator::{CR, LF, is_line_terminator},
};

/// JS/TS-specific source scanning, layered on the core mechanical [`SourceText`].
pub trait SourceTextExt {
    /// Check if span contains line terminators.
    fn contains_newline(&self, span: Span) -> bool;

    /// Check if range contains line terminators.
    fn contains_newline_between(&self, start: u32, end: u32) -> bool;

    /// Count consecutive line breaks after position, returning `0` if only whitespace follows.
    fn lines_after(&self, end: u32) -> usize;

    /// Count line breaks between syntax nodes, considering comments and parentheses.
    ///
    /// Encodes JS/TS leading-trivia rules:
    /// it skips an ASI semicolon (`;(function(){});`)
    /// and discounts newlines inside non-preserved parens (`(`…`)`).
    ///
    /// `first_unprinted_comment` is the span of the first not-yet-printed comment, or `None`.
    /// When that comment ends before `span.start`, its leading trivia is included in the count.
    fn get_lines_before(&self, span: Span, first_unprinted_comment: Option<Span>) -> usize;
}

impl SourceTextExt for SourceText<'_> {
    fn contains_newline(&self, span: Span) -> bool {
        self.contains_newline_between(span.start, span.end)
    }

    fn contains_newline_between(&self, start: u32, end: u32) -> bool {
        self.slice_range(start, end).chars().any(is_line_terminator)
    }

    fn lines_after(&self, end: u32) -> usize {
        let text: &str = self;
        let mut count = 0;
        let mut chars = text[end as usize..].chars().peekable();
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

    fn get_lines_before(&self, span: Span, first_unprinted_comment: Option<Span>) -> usize {
        let text: &str = self;
        let bytes = text.as_bytes();
        let mut start = span.start;

        // Should skip the leading comments of the node.
        if let Some(comment) = first_unprinted_comment
            && comment.end <= start
        {
            start = comment.start;
        } else if start != 0 && matches!(bytes.get((start - 1) as usize).copied(), Some(b';')) {
            // Skip leading semicolon if present
            // `;(function() {});`
            start -= 1;
        }

        // Count the newlines in the leading trivia of the next node
        let mut count = 0;
        let mut following_source = bytes[span.end as usize..].iter().copied();
        let mut chars = text[..start as usize].chars().rev().peekable();
        while let Some(c) = chars.next() {
            if is_white_space_single_line(c) {
                continue;
            }

            if c == '(' {
                // We don't have a parenthesis node when `preserveParens` is turned off,
                // but we will find the `(` and `)` around the node if it exists.
                // If we find a `(`, we try to find the matching `)` and reset the count.
                // This is necessary to avoid counting the newlines inside the parenthesis.

                for c in following_source.by_ref() {
                    if c.is_ascii_whitespace() {
                        continue;
                    }

                    if c == b')' {
                        break;
                    }

                    return count;
                }

                count = 0;
                continue;
            }

            if !is_line_terminator(c) {
                return count;
            }

            count += 1;
            if c == LF && chars.peek() == Some(&CR) {
                chars.next();
            }
        }

        0
    }
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

        let span_x = Span::new(0, 12);
        let span_y = Span::new(14, 26);
        let span_z = Span::new(29, 41);
        assert_eq!(source_text.text_for(&span_x), "const x = 1;");
        assert_eq!(source_text.text_for(&span_y), "const y = 2;");
        assert_eq!(source_text.text_for(&span_z), "const z = 3;");

        assert_eq!(source_text.get_lines_before(span_x, None), 0);
        assert_eq!(source_text.get_lines_before(span_y, None), 2);
        assert_eq!(source_text.get_lines_before(span_z, None), 3);

        assert_eq!(source_text.lines_after(span_x.end), 2);
        assert_eq!(source_text.lines_after(span_y.end), 3);
        assert_eq!(source_text.lines_after(span_z.end), 0);
    }

    #[test]
    fn test_source_text_with_crlf() {
        let source_text = "const x = 1;\r\n\r\nconst y = 2;\r\n\r\n\r\nconst z = 3;";
        let source_text = SourceText::new(source_text);

        let span_x = Span::new(0, 12);
        let span_y = Span::new(16, 28);
        let span_z = Span::new(34, 46);
        assert_eq!(source_text.text_for(&span_x), "const x = 1;");
        assert_eq!(source_text.text_for(&span_y), "const y = 2;");
        assert_eq!(source_text.text_for(&span_z), "const z = 3;");

        assert_eq!(source_text.get_lines_before(span_y, None), 2);
        assert_eq!(source_text.get_lines_before(span_z, None), 3);

        assert_eq!(source_text.lines_after(span_x.end), 2);
        assert_eq!(source_text.lines_after(span_y.end), 3);
    }

    #[test]
    fn test_source_text_with_mixed_line_endings() {
        let source_text = "const x = 1;\n\r\nconst y = 2;\r\n\nconst z = 3;";
        let source_text = SourceText::new(source_text);

        let span_x = Span::new(0, 12);
        let span_y = Span::new(15, 27);
        let span_z = Span::new(30, 42);
        assert_eq!(source_text.text_for(&span_x), "const x = 1;");
        assert_eq!(source_text.text_for(&span_y), "const y = 2;");
        assert_eq!(source_text.text_for(&span_z), "const z = 3;");

        assert_eq!(source_text.get_lines_before(span_y, None), 2);
        assert_eq!(source_text.get_lines_before(span_z, None), 2);

        assert_eq!(source_text.lines_after(span_x.end), 2);
        assert_eq!(source_text.lines_after(span_y.end), 2);
    }
}
