use oxc_span::Span;

use super::{Kind, Lexer, Token, cold_branch};
use crate::diagnostics;

impl Lexer<'_> {
    /// Section 12.8 Punctuators
    pub(super) fn read_dot(&mut self) -> Kind {
        if self.peek_2_bytes() == Some([b'.', b'.']) {
            self.consume_2_chars();
            return Kind::Dot3;
        }
        if self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
            self.decimal_literal_after_decimal_point()
        } else {
            Kind::Dot
        }
    }

    /// returns None for `SingleLineHTMLOpenComment` `<!--` in script mode
    pub(super) fn read_left_angle(&mut self) -> Option<Kind> {
        match self.peek_byte() {
            Some(b'<') => {
                self.consume_char();
                if self.next_ascii_byte_eq(b'=') {
                    Some(Kind::ShiftLeftEq)
                } else {
                    Some(Kind::ShiftLeft)
                }
            }
            Some(b'=') => {
                self.consume_char();
                Some(Kind::LtEq)
            }
            // `<!--` HTML comment (Annex B.1.1)
            Some(b'!') if self.remaining().starts_with("!--") => {
                // In Astro mode, HTML comments are allowed inside expressions
                // They should be added to trivia and skipped
                if self.source_type.is_astro() {
                    return cold_branch(|| Some(self.skip_astro_html_comment()));
                }

                if self.source_type.is_module() {
                    if self.token.is_on_new_line() {
                        let span = Span::new(self.token.start(), self.token.start() + 4);
                        self.errors.push(diagnostics::html_comment_in_module(span));
                        None
                    } else {
                        // In middle of expression (e.g. `foo <!--bar`) - parse as `<`
                        Some(Kind::LAngle)
                    }
                } else {
                    self.defer_html_comment_error(4);
                    None
                }
            }
            _ => Some(Kind::LAngle),
        }
    }

    /// Skip an HTML comment `<!-- ... -->` in Astro mode and add it to trivia.
    /// Returns `Kind::Skip` or `Kind::Eof` so the lexer continues to the next token.
    #[expect(clippy::cast_possible_truncation)]
    fn skip_astro_html_comment(&mut self) -> Kind {
        // We're at `!` after `<`, so comment starts at current token start (the `<`)
        let comment_start = self.token.start();

        // Skip `!--`
        self.consume_char(); // `!`
        self.consume_char(); // `-`
        self.consume_char(); // `-`

        // Find the closing `-->`
        let remaining = self.remaining();
        if let Some(end_offset) = remaining.find("-->") {
            // Calculate comment end position (after `-->`)
            let content_end = self.offset() + end_offset as u32;
            let comment_end = content_end + 3;

            // Add HTML comment to trivia
            self.trivia_builder.add_html_comment(comment_start, comment_end, self.source.whole());

            // Move lexer position past the comment
            // SAFETY: We verified the `-->` exists in the remaining source
            self.source.set_position(unsafe { self.source.position().add(end_offset + 3) });

            Kind::Skip
        } else {
            // Unterminated HTML comment - consume to end of file
            self.source.advance_to_end();
            self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
            Kind::Eof
        }
    }

    /// returns None for `SingleLineHTMLCloseComment` `-->` in script mode
    pub(super) fn read_minus(&mut self) -> Option<Kind> {
        match self.peek_byte() {
            Some(b'-') => {
                self.consume_char();
                // `-->` HTML comment (Annex B.1.1) - not recognized in strict mode (.mjs)
                if self.token.is_on_new_line()
                    && !self.source_type.is_strict()
                    && self.next_ascii_byte_eq(b'>')
                {
                    self.defer_html_comment_error(3);
                    None
                } else {
                    Some(Kind::Minus2)
                }
            }
            Some(b'=') => {
                self.consume_char();
                Some(Kind::MinusEq)
            }
            _ => Some(Kind::Minus),
        }
    }

    /// Defer HTML comment error for unambiguous mode (emitted if file resolves to module)
    fn defer_html_comment_error(&mut self, len: u32) {
        if self.source_type.is_unambiguous() {
            let span = Span::new(self.token.start(), self.token.start() + len);
            self.deferred_module_errors.push(diagnostics::html_comment_in_module(span));
        }
    }

    pub(crate) fn re_lex_right_angle(&mut self) -> Token {
        self.token.set_start(self.offset());
        let kind = self.read_right_angle();
        self.finish_next(kind)
    }

    fn read_right_angle(&mut self) -> Kind {
        if self.next_ascii_byte_eq(b'>') {
            if self.next_ascii_byte_eq(b'>') {
                if self.next_ascii_byte_eq(b'=') { Kind::ShiftRight3Eq } else { Kind::ShiftRight3 }
            } else if self.next_ascii_byte_eq(b'=') {
                Kind::ShiftRightEq
            } else {
                Kind::ShiftRight
            }
        } else if self.next_ascii_byte_eq(b'=') {
            Kind::GtEq
        } else {
            Kind::RAngle
        }
    }
}
