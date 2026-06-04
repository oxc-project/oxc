use std::ops::Deref;

use oxc_span::{GetSpan, Span};

/// Source text wrapper providing mechanical byte/offset access for the formatter.
///
/// This owns only language-agnostic, offset-keyed access (slicing, raw-byte newline checks).
/// Lexical-semantic scanning whose answer is language-defined
/// ("what counts as a newline / comment / trivia") lives in each consumer. (e.g. `oxc_formatter`'s `SourceTextExt`)
///
/// All positions are `u32` UTF-8 byte offsets into `text`.
/// This is the only hard prerequisite for any consumer:
/// - `u32` means byte offsets, not UTF-16 code units or `char` indices
/// - `u32` is the oxc-wide convention
///   - `oxc_span::Span` is `u32`-based, and `oxc_parser` rejects sources longer than `u32::MAX` bytes
///   - so casting a `usize` offset down to `u32` never truncates for parsed sources)
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
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    // Text slicing
    /// Get text between two positions
    pub fn slice_range(&self, start: u32, end: u32) -> &'a str {
        &self.text[start as usize..end as usize]
    }

    // Byte slicing
    /// Get bytes from position to end
    fn bytes_from(&self, position: u32) -> impl Iterator<Item = u8> {
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

    // Newline detection
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

    /// Check for a newline starting at `position`, scanning through comments.
    /// Unlike [`Self::has_newline_after`], a comment between `position` and the newline is
    /// transparent (matches Prettier detecting the newline in `{ /* comment */\n`).
    ///
    /// NOTE: comment scanning assumes C-family comment syntax (`//`, `/* */`),
    /// shared by all current consumers (JS/TS and JSON, a JS subset).
    /// A future non-C-family consumer must supply its own comment-aware scan,
    /// or make this more generic. (e.g. pass comments ranges to skip)
    pub fn has_newline_after_skipping_comments(&self, position: u32) -> bool {
        let mut iter = self.bytes_from(position).peekable();

        while let Some(byte) = iter.next() {
            match byte {
                b'\n' | b'\r' => return true,
                b' ' | b'\t' => {}
                b'/' => match iter.peek() {
                    Some(&b'/') => {
                        iter.next();
                        // Line comment: scan until newline or EOF
                        return iter.any(|b| b == b'\n' || b == b'\r');
                    }
                    Some(&b'*') => {
                        iter.next();
                        // Block comment: scan for */ and check for newlines
                        while let Some(b) = iter.next() {
                            if matches!(b, b'\n' | b'\r') {
                                return true;
                            }
                            if b == b'*' && matches!(iter.peek(), Some(&b'/')) {
                                iter.next();
                                break;
                            }
                        }
                    }
                    _ => return false,
                },
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
}

// Span-based access
impl<'a> SourceText<'a> {
    /// Extract text for an object that has a span
    pub fn text_for<T: GetSpan>(&self, obj: &T) -> &'a str {
        obj.span().source_text(self.text)
    }

    // Utility methods
    /// Get character count of span
    pub fn span_width(&self, span: Span) -> usize {
        self.text_for(&span).chars().count()
    }
}
