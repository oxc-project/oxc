use crate::{config::LexerConfig as Config, diagnostics};

use super::{
    Kind, Lexer, RegExpFlags, Token,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
};

// 0xE2 is the first byte of the Unicode line terminators U+2028 and U+2029.
static REGEX_SPECIAL_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'/' | b'[' | b']' | b'\\' | b'\r' | b'\n' | 0xE2));

impl<C: Config> Lexer<'_, C> {
    /// Re-tokenize the current `/` or `/=` and return `RegExp`
    /// See Section 12:
    ///   The `InputElementRegExp` goal symbol is used in all syntactic grammar contexts
    ///   where a `RegularExpressionLiteral` is permitted
    /// Which means the parser needs to re-tokenize on `PrimaryExpression`,
    /// `RegularExpressionLiteral` only appear on the right hand side of `PrimaryExpression`
    pub(crate) fn next_regex(&mut self, kind: Kind) -> (Token, u32, RegExpFlags) {
        self.token.set_start(
            self.offset()
                - match kind {
                    Kind::Slash => 1,
                    Kind::SlashEq => 2,
                    _ => unreachable!(),
                },
        );
        let (pattern_end, flags) = self.read_regex();
        let token = self.finish_next_retokenized(Kind::RegExp);
        (token, pattern_end, flags)
    }

    /// 12.9.5 Regular Expression Literals
    fn read_regex(&mut self) -> (u32, RegExpFlags) {
        self.read_regex_pattern();

        let pattern_end = self.offset() - 1; // -1 to exclude `/`
        let mut flags = RegExpFlags::empty();

        while let Some(b @ (b'$' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')) =
            self.peek_byte()
        {
            self.consume_char();
            let Ok(flag) = RegExpFlags::try_from(b) else {
                self.error(diagnostics::reg_exp_flag(
                    b as char,
                    self.current_offset().expand_left(1),
                ));
                continue;
            };
            if flags.contains(flag) {
                self.error(diagnostics::reg_exp_flag_twice(
                    b as char,
                    self.current_offset().expand_left(1),
                ));
                continue;
            }
            flags |= flag;
        }

        (pattern_end, flags)
    }

    /// Skip ordinary bytes in batches, interpreting only delimiters, escapes and line terminators.
    fn read_regex_pattern(&mut self) {
        let mut in_character_class = false;

        byte_search! {
            lexer: self,
            table: REGEX_SPECIAL_TABLE,
            continue_if: (byte, pos) {
                match byte {
                    b'/' if !in_character_class => {
                        // SAFETY: `/` is ASCII, so the next byte is a UTF-8 boundary.
                        pos = unsafe { pos.add(1) };
                        false
                    }
                    b'[' => {
                        in_character_class = true;
                        true
                    }
                    b']' => {
                        in_character_class = false;
                        true
                    }
                    b'/' => true,
                    b'\\' => {
                        // Consume the escaped character without interpreting it as a delimiter.
                        // SAFETY: `\\` is ASCII, so the next byte is a UTF-8 boundary.
                        let after_backslash = unsafe { pos.add(1) };
                        let next = self.source.str_from_pos_to_end(after_backslash).chars().next();
                        if let Some(c) = next {
                            // The macro advances one more byte after this block, placing `pos`
                            // after the escaped character on a UTF-8 boundary.
                            // SAFETY: The escaped character is present in the source.
                            pos = unsafe { pos.add(c.len_utf8()) };
                            if matches!(c, '\r' | '\n' | '\u{2028}' | '\u{2029}') {
                                // SAFETY: This is just after the escaped character.
                                self.source.set_position(unsafe { pos.add(1) });
                                self.error(diagnostics::unterminated_reg_exp(self.unterminated_range()));
                                self.advance_to_end();
                                return;
                            }
                        }
                        true
                    }
                    _ => {
                        let line_break_len = if byte == 0xE2 {
                            // SAFETY: 0xE2 starts a three-byte UTF-8 character.
                            let next2 = unsafe { pos.add(1).read2() };
                            matches!(next2, [0x80, 0xA8 | 0xA9]).then_some(3)
                        } else {
                            Some(1)
                        };
                        if let Some(len) = line_break_len {
                            // SAFETY: `len` is the length of this line terminator.
                            self.source.set_position(unsafe { pos.add(len) });
                            self.error(diagnostics::unterminated_reg_exp(self.unterminated_range()));
                            self.advance_to_end();
                            return;
                        }
                        // SAFETY: This is a non-terminating three-byte UTF-8 character.
                        // The macro skips its final byte, reaching the next UTF-8 boundary.
                        pos = unsafe { pos.add(2) };
                        true
                    }
                }
            },
            handle_eof: {
                self.error(diagnostics::unterminated_reg_exp(self.unterminated_range()));
                return;
            },
        };
    }
}
