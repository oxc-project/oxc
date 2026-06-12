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

    /// Is there a line terminator immediately before `position` (skipping only horizontal whitespace, space / tab)?
    ///
    /// Recognizes the full ECMAScript line-terminator set, scanning by `char` so a multi-byte terminator is never split.
    /// Matches Prettier's `skipSpaces` + `isLineTerminator`.
    fn has_line_terminator_before(&self, position: u32) -> bool;

    /// Is there a line terminator immediately after `position`? The forward counterpart of [`Self::has_line_terminator_before`].
    fn has_line_terminator_after(&self, position: u32) -> bool;

    /// Is there a line terminator after `position`,
    /// treating an intervening comment as transparent (matches Prettier detecting the newline in `{ /* comment */\n`)?
    fn has_line_terminator_after_skipping_comments(&self, position: u32) -> bool;
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

    fn has_line_terminator_before(&self, position: u32) -> bool {
        let text: &str = self;
        for c in text[..position as usize].chars().rev() {
            match c {
                ' ' | '\t' => {}
                _ if is_line_terminator(c) => return true,
                _ => return false,
            }
        }
        false
    }

    fn has_line_terminator_after(&self, position: u32) -> bool {
        let text: &str = self;
        for c in text[position as usize..].chars() {
            match c {
                ' ' | '\t' => {}
                _ if is_line_terminator(c) => return true,
                _ => return false,
            }
        }
        false
    }

    fn has_line_terminator_after_skipping_comments(&self, position: u32) -> bool {
        let text: &str = self;
        let mut chars = text[position as usize..].chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                ' ' | '\t' => {}
                _ if is_line_terminator(c) => return true,
                '/' => match chars.peek() {
                    Some(&'/') => {
                        chars.next();
                        // Line comment: scan until line terminator or EOF.
                        return chars.any(is_line_terminator);
                    }
                    Some(&'*') => {
                        chars.next();
                        // Block comment: scan for `*/`, returning early on any inner terminator.
                        while let Some(c) = chars.next() {
                            if is_line_terminator(c) {
                                return true;
                            }
                            if c == '*' && chars.peek() == Some(&'/') {
                                chars.next();
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

    #[test]
    fn has_line_terminator_recognizes_ls_ps_and_crlf() {
        // Each case is `a` + gap + `b`. `has_*_after` scans from byte 1 (right after `a`);
        // `has_*_before` scans back from the start of `b` (byte `1 + gap.len()`).
        for gap in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}", "  \u{2028}"] {
            let src = format!("a{gap}b");
            let st = SourceText::new(&src);
            let before_b = u32::try_from(1 + gap.len()).unwrap();
            assert!(st.has_line_terminator_after(1), "after: {src:?}");
            assert!(st.has_line_terminator_before(before_b), "before: {src:?}");
        }

        // No terminator (incl. other 0xE2-led chars: em dash U+2014, bullet U+2022).
        for gap in [" ", "\u{2014}", " \u{2022} "] {
            let src = format!("a{gap}b");
            let st = SourceText::new(&src);
            let before_b = u32::try_from(1 + gap.len()).unwrap();
            assert!(!st.has_line_terminator_after(1), "after: {src:?}");
            assert!(!st.has_line_terminator_before(before_b), "before: {src:?}");
        }
    }

    #[test]
    fn has_line_terminator_after_skipping_comments_is_ls_ps_aware() {
        // Terminator hidden behind a comment is still detected.
        let st = SourceText::new("{ /* c */\u{2028}x }");
        assert!(st.has_line_terminator_after_skipping_comments(1));
        // Terminator inside the block comment itself counts.
        let st = SourceText::new("{ /* c\u{2029} */ x }");
        assert!(st.has_line_terminator_after_skipping_comments(1));
        // Line comment then terminator.
        let st = SourceText::new("{ // c\u{2028}x }");
        assert!(st.has_line_terminator_after_skipping_comments(1));
        // No terminator before the next token.
        let st = SourceText::new("{ /* c */ x }");
        assert!(!st.has_line_terminator_after_skipping_comments(1));
    }
}
