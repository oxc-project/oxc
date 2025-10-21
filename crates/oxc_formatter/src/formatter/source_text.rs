use std::ops::Deref;

use oxc_ast::Comment;
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::{is_line_terminator, is_white_space_single_line};

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
    pub fn bytes_from(&self, position: u32) -> &'a [u8] {
        &self.text.as_bytes()[position as usize..]
    }

    /// Get bytes from start to position
    pub fn bytes_to(&self, position: u32) -> &'a [u8] {
        &self.text.as_bytes()[..position as usize]
    }

    /// Get bytes between two positions
    pub fn bytes_range(&self, start: u32, end: u32) -> &'a [u8] {
        &self.text.as_bytes()[start as usize..end as usize]
    }

    // Byte checking
    /// Check if first non-whitespace byte at position matches expected
    pub fn next_non_whitespace_byte_is(&self, position: u32, expected_byte: u8) -> bool {
        self.bytes_from(position).trim_ascii_start().first().is_some_and(|&b| b == expected_byte)
    }

    /// Get first byte at position
    pub fn byte_at(&self, position: u32) -> Option<u8> {
        self.bytes_from(position).first().copied()
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
        for &byte in self.bytes_to(position).iter().rev() {
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
        for &byte in self.bytes_from(position) {
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
        for char in self.slice_from(end).chars() {
            if is_white_space_single_line(char) {
                continue;
            }

            if is_line_terminator(char) {
                count += 1;
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

        // Should skip the leading comments of the node.
        if let Some(comment) = comments.first()
            && comment.span.end <= start
        {
            start = comment.span.start;
        } else if start != 0 && matches!(self.byte_at(start - 1), Some(b';')) {
            // Skip leading semicolon if present
            // `;(function() {});`
            start -= 1;
        }

        // Count the newlines in the leading trivia of the next node
        let mut count = 0;
        let mut following_source = self.bytes_from(span.end).iter();
        for c in self.slice_to(start).chars().rev() {
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

                    if c == &b')' {
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
        }

        0
    }
}
