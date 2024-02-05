use super::{Kind, Lexer};
use crate::diagnostics;

use oxc_syntax::identifier::is_line_terminator;

impl<'a> Lexer<'a> {
    /// Section 12.4 Single Line Comment
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn skip_single_line_comment(&mut self) -> Kind {
        let start = self.token.start;

        // The first byte of the UTF-8 encoding of U+2028 and U+2029.
        let ps_ls_start_byte = 0xE2;

        while let Some(byte) = self.source.eat_until_byte3(b'\n', b'\r', ps_ls_start_byte) {
            // Handle ambiguity between PS/LS and some other UTF-8 character.
            if byte == ps_ls_start_byte && !matches!(self.peek(), Some('\u{2028}' | '\u{2029}')) {
                self.consume_char();
                continue;
            }

            self.token.is_on_new_line = true;
            self.trivia_builder.add_single_line_comment(start, self.offset());
            self.consume_char();
            return Kind::Skip;
        }

        // EOF
        self.source.set_eof();
        self.trivia_builder.add_single_line_comment(start, self.offset());
        Kind::Skip
    }

    /// Section 12.4 Multi Line Comment
    pub(super) fn skip_multi_line_comment(&mut self) -> Kind {
        while let Some(c) = self.next_char() {
            if c == '*' && self.next_eq('/') {
                self.trivia_builder.add_multi_line_comment(self.token.start, self.offset());
                return Kind::Skip;
            }
            if is_line_terminator(c) {
                self.token.is_on_new_line = true;
            }
        }
        self.error(diagnostics::UnterminatedMultiLineComment(self.unterminated_range()));
        Kind::Eof
    }

    /// Section 12.5 Hashbang Comments
    pub(super) fn read_hashbang_comment(&mut self) -> Kind {
        while let Some(c) = self.next_char().as_ref() {
            if is_line_terminator(*c) {
                break;
            }
        }
        self.token.is_on_new_line = true;
        Kind::HashbangComment
    }
}
