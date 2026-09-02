use crate::{config::LexerConfig as Config, diagnostics};
use oxc_allocator::ArenaStringBuilder;
use oxc_str::JSStrBuilder;
use oxc_syntax::{
    identifier::{
        FF, TAB, VT, is_identifier_part, is_identifier_start, is_identifier_start_unicode,
        is_irregular_whitespace,
    },
    line_terminator::{CR, LF, LS, PS, is_irregular_line_terminator},
};

use super::{Kind, Lexer, Span};

/// A Unicode escape sequence.
///
/// `\u Hex4Digits`, `\u Hex4Digits \u Hex4Digits`, or `\u{ HexDigits }`.
enum UnicodeEscape {
    // `\u Hex4Digits` or `\u{ HexDigits }`, which forms a valid Unicode code point.
    // Char cannot be in range 0xD800..=0xDFFF.
    CodePoint(char),
    // `\u Hex4Digits \u Hex4Digits`, which forms a valid Unicode astral code point.
    // Char is in the range 0x10000..=0x10FFFF.
    SurrogatePair(char),
    // `\u Hex4Digits` or `\u{ HexDigits }`, which forms an invalid Unicode code point.
    // Code unit is in the range 0xD800..=0xDFFF.
    LoneSurrogate(u32),
}

impl<'a, C: Config> Lexer<'a, C> {
    pub(super) fn unicode_char_handler(&mut self) -> Kind {
        let c = self.peek_char().unwrap();
        match c {
            // U+FFFD (replacement character) appears when a binary file is decoded as UTF-8.
            // This is likely a binary file that cannot be parsed.
            // <https://github.com/microsoft/TypeScript/blob/main/src/compiler/scanner.ts>
            '\u{FFFD}' => self.handle_binary_file(),
            c if is_identifier_start_unicode(c) => {
                let start_pos = self.source.position();
                self.consume_char();
                self.identifier_tail_after_unicode(start_pos);
                Kind::Ident
            }
            c if is_irregular_whitespace(c) => self.handle_irregular_whitespace(c),
            c if is_irregular_line_terminator(c) => self.handle_irregular_line_terminator(c),
            _ => self.handle_invalid_unicode_char(c),
        }
    }

    #[cold]
    fn handle_binary_file(&mut self) -> Kind {
        self.error(diagnostics::file_appears_to_be_binary());
        self.source.advance_to_end();
        Kind::Eof
    }

    #[cold]
    fn handle_irregular_whitespace(&mut self, _c: char) -> Kind {
        self.consume_char();
        self.trivia_builder.add_irregular_whitespace(self.token.start(), self.offset());
        Kind::Skip
    }

    #[cold]
    fn handle_irregular_line_terminator(&mut self, _c: char) -> Kind {
        self.consume_char();
        self.token.set_is_on_new_line(true);
        self.trivia_builder.add_irregular_whitespace(self.token.start(), self.offset());
        Kind::Skip
    }

    #[cold]
    fn handle_invalid_unicode_char(&mut self, c: char) -> Kind {
        self.consume_char();
        self.error(diagnostics::invalid_character(c, self.unterminated_range()));
        Kind::Undetermined
    }

    /// Identifier `UnicodeEscapeSequence`
    ///   \u `Hex4Digits`
    ///   \u{ `CodePoint` }
    pub(super) fn identifier_unicode_escape_sequence(
        &mut self,
        str: &mut ArenaStringBuilder<'a>,
        check_identifier_start: bool,
    ) {
        let start = self.offset();
        if self.peek_byte() == Some(b'u') {
            self.consume_char();
        } else {
            self.next_char();
            let range = Span::new(start, self.offset());
            self.error(diagnostics::unicode_escape_sequence(range));
            return;
        }

        let value = match self.peek_byte() {
            Some(b'{') => {
                self.consume_char();
                self.unicode_code_point()
            }
            _ => self.unicode_code_unit(),
        };

        let Some(value) = value else {
            let range = Span::new(start, self.offset());
            self.error(diagnostics::unicode_escape_sequence(range));
            return;
        };

        // For Identifiers, surrogate pair is an invalid grammar, e.g. `var \uD800\uDEA7`.
        let ch = match value {
            UnicodeEscape::CodePoint(ch) => ch,
            UnicodeEscape::SurrogatePair(_) | UnicodeEscape::LoneSurrogate(_) => {
                let range = Span::new(start, self.offset());
                self.error(diagnostics::unicode_escape_sequence(range));
                return;
            }
        };

        let is_valid =
            if check_identifier_start { is_identifier_start(ch) } else { is_identifier_part(ch) };

        if !is_valid {
            self.error(diagnostics::invalid_character(ch, self.current_offset()));
            return;
        }

        str.push(ch);
    }

    /// String `UnicodeEscapeSequence`
    ///   \u `Hex4Digits`
    ///   \u `Hex4Digits` \u `Hex4Digits`
    ///   \u{ `CodePoint` }
    fn string_unicode_escape_sequence(
        &mut self,
        text: &mut JSStrBuilder<'a>,
        is_valid_escape_sequence: &mut bool,
    ) {
        let value = match self.peek_byte() {
            Some(b'{') => {
                self.consume_char();
                self.unicode_code_point()
            }
            _ => self.unicode_code_unit(),
        };

        let Some(value) = value else {
            // error raised within the parser by `diagnostics::template_literal`
            *is_valid_escape_sequence = false;
            return;
        };

        // For strings and templates, surrogate pairs are valid grammar, e.g. `"\uD83D\uDE00" === 😀`.
        match value {
            UnicodeEscape::CodePoint(ch) => text.push_char(ch),
            UnicodeEscape::SurrogatePair(ch) => {
                // Surrogate pair is always >= 0x10000, so cannot be 0xFFFD
                text.push_char(ch);
            }
            UnicodeEscape::LoneSurrogate(code_point) => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "lone surrogates are always within u16"
                )]
                text.push_code_unit(code_point as u16);
            }
        }
    }

    /// Decode unicode code point (`\u{ HexBytes }`).
    ///
    /// The opening `\u{` must already have been consumed before calling this method.
    fn unicode_code_point(&mut self) -> Option<UnicodeEscape> {
        let value = self.code_point()?;
        if !self.next_ascii_byte_eq(b'}') {
            return None;
        }
        Some(value)
    }

    fn hex_4_digits(&mut self) -> Option<u32> {
        let mut value = 0;
        for _ in 0..4 {
            value = (value << 4) | self.hex_digit()?;
        }
        Some(value)
    }

    fn hex_digit(&mut self) -> Option<u32> {
        let b = self.peek_byte()?;

        // Reduce instructions and remove 1 branch by comparing against `A-F` and `a-f` simultaneously
        // https://godbolt.org/z/9caMMzvP3
        let value = if b.is_ascii_digit() {
            b - b'0'
        } else {
            // Match `A-F` or `a-f`. `b | 32` converts uppercase letters to lowercase,
            // but leaves lowercase as they are
            let lower_case = b | 32;
            if matches!(lower_case, b'a'..=b'f') {
                lower_case + 10 - b'a'
            } else {
                return None;
            }
        };

        // Because of `b | 32` above, compiler cannot deduce that next byte is definitely ASCII
        // so `next_byte_unchecked` is necessary to produce compact assembly, rather than `consume_char`.
        // SAFETY: This code is only reachable if there is a byte remaining, and it's ASCII.
        // Therefore it's safe to consume that byte, and will leave position on a UTF-8 char boundary.
        unsafe { self.source.next_byte_unchecked() };

        Some(u32::from(value))
    }

    fn code_point(&mut self) -> Option<UnicodeEscape> {
        let mut value = self.hex_digit()?;
        while let Some(next) = self.hex_digit() {
            value = (value << 4) | next;
            if value > 0x0010_FFFF {
                return None;
            }
        }

        match char::from_u32(value) {
            Some(ch) => Some(UnicodeEscape::CodePoint(ch)),
            None => Some(UnicodeEscape::LoneSurrogate(value)),
        }
    }

    /// Unicode code unit (`\uXXXX`).
    ///
    /// The opening `\u` must already have been consumed before calling this method.
    ///
    /// See background info on surrogate pairs:
    ///   * `https://mathiasbynens.be/notes/javascript-encoding#surrogate-formulae`
    ///   * `https://mathiasbynens.be/notes/javascript-identifiers-es6`
    fn unicode_code_unit(&mut self) -> Option<UnicodeEscape> {
        const MIN_HIGH: u32 = 0xD800;
        const MAX_HIGH: u32 = 0xDBFF;
        const MIN_LOW: u32 = 0xDC00;
        const MAX_LOW: u32 = 0xDFFF;

        // `https://tc39.es/ecma262/#sec-utf16decodesurrogatepair`
        #[inline]
        const fn pair_to_code_point(high: u32, low: u32) -> u32 {
            (high - 0xD800) * 0x400 + low - 0xDC00 + 0x10000
        }

        const _: () = {
            assert!(char::from_u32(pair_to_code_point(MIN_HIGH, MIN_LOW)).is_some());
            assert!(char::from_u32(pair_to_code_point(MIN_HIGH, MAX_LOW)).is_some());
            assert!(char::from_u32(pair_to_code_point(MAX_HIGH, MIN_LOW)).is_some());
            assert!(char::from_u32(pair_to_code_point(MAX_HIGH, MAX_LOW)).is_some());
        };

        let high = self.hex_4_digits()?;
        if let Some(ch) = char::from_u32(high) {
            return Some(UnicodeEscape::CodePoint(ch));
        }

        // The first code unit of a surrogate pair is always in the range from 0xD800 to 0xDBFF,
        // and is called a high surrogate or a lead surrogate.
        // Note: `high` must be >= `MIN_HIGH`, otherwise `char::from_u32` would have returned `Some`,
        // and already exited.
        debug_assert!(high >= MIN_HIGH);
        let is_pair = high <= MAX_HIGH && self.peek_2_bytes() == Some([b'\\', b'u']);
        if !is_pair {
            return Some(UnicodeEscape::LoneSurrogate(high));
        }

        let before_second = self.source.position();

        // SAFETY: We checked above that next 2 chars are `\u`
        unsafe {
            self.source.next_byte_unchecked();
            self.source.next_byte_unchecked();
        }

        // The second code unit of a surrogate pair is always in the range from 0xDC00 to 0xDFFF,
        // and is called a low surrogate or a trail surrogate.
        if let Some(low) = self.hex_4_digits()
            && (MIN_LOW..=MAX_LOW).contains(&low)
        {
            let code_point = pair_to_code_point(high, low);
            // SAFETY: `high` and `low` have been checked to be in ranges which always yield a `code_point`
            // which is a valid `char`
            let ch = unsafe { char::from_u32_unchecked(code_point) };
            return Some(UnicodeEscape::SurrogatePair(ch));
        }

        // Not a valid surrogate pair.
        // Rewind to before the 2nd, and return the first only.
        // The 2nd could be the first part of a valid pair, or a `\u{...}` escape.
        self.source.set_position(before_second);
        Some(UnicodeEscape::LoneSurrogate(high))
    }

    // EscapeSequence ::
    pub(super) fn read_string_escape_sequence(
        &mut self,
        text: &mut JSStrBuilder<'a>,
        in_template: bool,
        is_valid_escape_sequence: &mut bool,
    ) {
        self.read_string_escape_sequence_impl(text, in_template, is_valid_escape_sequence);
    }

    fn read_string_escape_sequence_impl(
        &mut self,
        text: &mut JSStrBuilder<'a>,
        in_template: bool,
        is_valid_escape_sequence: &mut bool,
    ) {
        match self.next_char() {
            None => {
                self.error(diagnostics::unterminated_string(self.unterminated_range()));
            }
            Some(c) => match c {
                // \ LineTerminatorSequence
                // LineTerminatorSequence ::
                // <LF>
                // <CR> [lookahead ≠ <LF>]
                // <LS>
                // <PS>
                // <CR> <LF>
                LF | LS | PS => {}
                CR => {
                    self.next_ascii_byte_eq(b'\n');
                }
                // SingleEscapeCharacter :: one of
                //   ' " \ b f n r t v
                '\'' | '"' | '\\' => text.push_char(c),
                'b' => text.push_char('\u{8}'),
                'f' => text.push_char(FF),
                'n' => text.push_char(LF),
                'r' => text.push_char(CR),
                't' => text.push_char(TAB),
                'v' => text.push_char(VT),
                // HexEscapeSequence
                'x' => {
                    self.hex_digit()
                        .and_then(|value1| {
                            let value2 = self.hex_digit()?;
                            Some((value1, value2))
                        })
                        .map(|(value1, value2)| (value1 << 4) | value2)
                        .and_then(|value| char::try_from(value).ok())
                        .map_or_else(
                            || {
                                *is_valid_escape_sequence = false;
                            },
                            |c| {
                                text.push_char(c);
                            },
                        );
                }
                // UnicodeEscapeSequence
                'u' => {
                    self.string_unicode_escape_sequence(text, is_valid_escape_sequence);
                }
                // 0 [lookahead ∉ DecimalDigit]
                '0' if !self.peek_byte().is_some_and(|b| b.is_ascii_digit()) => {
                    text.push_char('\0');
                }
                // Section 12.9.4 String Literals
                // LegacyOctalEscapeSequence
                // NonOctalDecimalEscapeSequence
                c @ '0'..='7' if !in_template => {
                    let first_digit = c as u8 - b'0';
                    let mut value = first_digit;

                    if matches!(self.peek_byte(), Some(b'0'..=b'7')) {
                        let digit = self.consume_char() as u8 - b'0';
                        value = value * 8 + digit;
                        if first_digit < 4 && matches!(self.peek_byte(), Some(b'0'..=b'7')) {
                            let digit = self.consume_char() as u8 - b'0';
                            value = value * 8 + digit;

                            if value >= 128 {
                                text.push_char(char::from(value));
                                return;
                            }
                        }
                    }

                    text.push_char(char::from(value));
                }
                '0' if in_template && self.peek_byte().is_some_and(|b| b.is_ascii_digit()) => {
                    self.consume_char();
                    // error raised within the parser by `diagnostics::template_literal`
                    *is_valid_escape_sequence = false;
                }
                // NotEscapeSequence :: DecimalDigit but not 0
                '1'..='9' if in_template => {
                    // error raised within the parser by `diagnostics::template_literal`
                    *is_valid_escape_sequence = false;
                }
                other => {
                    // NonOctalDecimalEscapeSequence \8 \9 in strict mode
                    text.push_char(other);
                }
            },
        }
    }
}
