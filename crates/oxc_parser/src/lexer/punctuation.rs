use oxc_span::Span;

use crate::{config::LexerConfig as Config, diagnostics};

use super::{Kind, Lexer, Token, cold_branch};

impl<C: Config> Lexer<'_, C> {
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
            // `<!--` HTML comment (Annex B.1.1). Rare legacy syntax, so handle out of line:
            // it builds/pushes a diagnostic, whose `OxcDiagnostic` buffer would otherwise inflate
            // the `<` byte-handler's stack frame on the common `<`/`<<`/`<=` path.
            Some(b'!') if self.remaining().starts_with("!--") => cold_branch(|| {
                if self.source_type.is_module() {
                    if self.pending.is_on_new_line() {
                        let span = Span::sized(self.token_start, 4);
                        self.errors.push(diagnostics::html_comment_in_module(span));
                        None
                    } else {
                        // In middle of expression (e.g. `foo <!--bar`) - parse as `<`
                        Some(Kind::LAngle)
                    }
                } else {
                    // Script or unambiguous mode. In unambiguous mode the module kind is not
                    // known yet, so `<!--` is lexed as a comment and the error deferred.
                    // Unlike a comment at the start of a line, the Module goal would lex a
                    // mid-line `<!--` as `<` `!` `--` instead (e.g. `x = a <!--b` is
                    // `x = a < !--b`), so the source must be parsed again from the start if it
                    // resolves to a module.
                    let needs_module_reparse = !self.pending.is_on_new_line();
                    self.defer_html_comment_error(4, needs_module_reparse);
                    None
                }
            }),
            _ => Some(Kind::LAngle),
        }
    }

    /// returns None for `SingleLineHTMLCloseComment` `-->` in script mode
    pub(super) fn read_minus(&mut self) -> Option<Kind> {
        match self.peek_byte() {
            Some(b'-') => {
                self.consume_char();
                // `-->` HTML comment (Annex B.1.1) - not recognized in strict mode (.mjs)
                if self.pending.is_on_new_line()
                    && !self.source_type.is_strict()
                    && self.next_ascii_byte_eq(b'>')
                {
                    // In unambiguous mode the module kind is not known yet, so `-->` is lexed
                    // as a comment and the error deferred. The Module goal would lex `-->` as
                    // `--` `>` instead, so the source must be parsed again from the start if it
                    // resolves to a module.
                    self.defer_html_comment_error(3, true);
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

    /// Defer HTML comment error for unambiguous mode (emitted if file resolves to module).
    ///
    /// `needs_module_reparse` is set when the Module goal would not lex the comment the same
    /// way (it does not recognize HTML-like comments - Annex B.1.1), so the whole source must
    /// be parsed again as a module if the file resolves to one.
    fn defer_html_comment_error(&mut self, len: u32, needs_module_reparse: bool) {
        if self.source_type.is_unambiguous() {
            let span = Span::sized(self.token_start, len);
            self.deferred_module_errors.push(diagnostics::html_comment_in_module(span));
            self.needs_module_reparse |= needs_module_reparse;
        }
    }

    pub(crate) fn re_lex_right_angle(&mut self) -> Token {
        self.token_start = self.offset() - 1;
        let kind = self.read_right_angle();
        self.finish_next_retokenized(kind)
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
