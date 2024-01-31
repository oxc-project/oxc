use super::{AutoCow, Kind, Lexer, Span};
use crate::diagnostics;

use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

impl<'a> Lexer<'a> {
    /// Section 12.7.1 Identifier Names
    pub(super) fn identifier_name_handler(&mut self) -> &'a str {
        let builder = AutoCow::new(self);
        self.consume_char();
        self.identifier_name(builder)
    }

    pub(super) fn identifier_name(&mut self, builder: AutoCow<'a>) -> &'a str {
        self.identifier_tail(builder)
    }

    pub(super) fn private_identifier(&mut self) -> Kind {
        let mut builder = AutoCow::new(self);
        let start = self.offset();
        match self.next_char() {
            Some(c) if is_identifier_start(c) => {
                builder.push_matching(c);
            }
            Some('\\') => {
                builder.force_allocation_without_current_ascii_char(self);
                self.identifier_unicode_escape_sequence(&mut builder, true);
            }
            Some(c) => {
                #[allow(clippy::cast_possible_truncation)]
                self.error(diagnostics::InvalidCharacter(
                    c,
                    Span::new(start, start + c.len_utf8() as u32),
                ));
                return Kind::Undetermined;
            }
            None => {
                self.error(diagnostics::UnexpectedEnd(Span::new(start, start)));
                return Kind::Undetermined;
            }
        }
        self.identifier_tail(builder);
        Kind::PrivateIdentifier
    }

    fn identifier_tail(&mut self, mut builder: AutoCow<'a>) -> &'a str {
        // ident tail
        while let Some(c) = self.peek() {
            if !is_identifier_part(c) {
                if c == '\\' {
                    self.consume_char();
                    builder.force_allocation_without_current_ascii_char(self);
                    self.identifier_unicode_escape_sequence(&mut builder, false);
                    continue;
                }
                break;
            }
            self.consume_char();
            builder.push_matching(c);
        }
        let has_escape = builder.has_escape();
        let text = builder.finish(self);
        self.save_string(has_escape, text);
        text
    }
}
