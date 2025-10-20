use std::{borrow::Cow, fmt::Write};

use cow_utils::CowUtils;

use oxc_allocator::StringBuilder;
use oxc_syntax::identifier::{
    CR, FF, LF, LS, PS, TAB, VT, is_identifier_part, is_identifier_start,
    is_identifier_start_unicode,
};

use crate::diagnostics;

use super::{Kind, Lexer, Span};

// UTF-8 byte sequences for irregular line terminators
const LS_BYTES: [u8; 3] = [0xE2, 0x80, 0xA8]; // U+2028 LINE SEPARATOR
const PS_BYTES: [u8; 3] = [0xE2, 0x80, 0xA9]; // U+2029 PARAGRAPH SEPARATOR

// UTF-8 byte sequences for common irregular whitespace
const OGHAM_SPACE_BYTES: [u8; 3] = [0xE1, 0x9A, 0x80]; // U+1680 OGHAM SPACE MARK
const ZWNBSP_BYTES: [u8; 3] = [0xEF, 0xBB, 0xBF]; // U+FEFF ZERO WIDTH NO-BREAK SPACE

// UTF-8 byte sequences for En Quad through Zero Width Space (U+2000 to U+200A)
const EN_QUAD_BYTES: [u8; 3] = [0xE2, 0x80, 0x80]; // U+2000
const EM_QUAD_BYTES: [u8; 3] = [0xE2, 0x80, 0x81]; // U+2001
const EN_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x82]; // U+2002
const EM_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x83]; // U+2003
const THREE_PER_EM_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x84]; // U+2004
const FOUR_PER_EM_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x85]; // U+2005
const SIX_PER_EM_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x86]; // U+2006
const FIGURE_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x87]; // U+2007
const PUNCTUATION_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x88]; // U+2008
const THIN_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x89]; // U+2009
const HAIR_SPACE_BYTES: [u8; 3] = [0xE2, 0x80, 0x8A]; // U+200A

const NNBSP_BYTES: [u8; 3] = [0xE2, 0x80, 0xAF]; // U+202F NARROW NO-BREAK SPACE
const MMSP_BYTES: [u8; 3] = [0xE2, 0x81, 0x9F]; // U+205F MEDIUM MATHEMATICAL SPACE
const IDEOGRAPHIC_SPACE_BYTES: [u8; 3] = [0xE3, 0x80, 0x80]; // U+3000 IDEOGRAPHIC SPACE

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

impl<'a> Lexer<'a> {
    /// Check if the next bytes match a specific byte sequence.
    /// Returns `true` if the bytes match, `false` otherwise.
    #[inline]
    fn check_byte_sequence(&self, sequence: &[u8]) -> bool {
        let Some(first_byte) = self.peek_byte() else {
            return false;
        };

        if first_byte != sequence[0] {
            return false;
        }

        if sequence.len() == 1 {
            return true;
        }

        // For multi-byte sequences, we need to peek additional bytes
        if sequence.len() == 2 {
            if let Some([b1, b2]) = self.peek_2_bytes() {
                return b1 == sequence[0] && b2 == sequence[1];
            }
            return false;
        }

        // For 3-byte sequences, check first 2 bytes, then check 3rd
        if sequence.len() == 3
            && let Some([b1, b2]) = self.peek_2_bytes()
        {
            if b1 != sequence[0] || b2 != sequence[1] {
                return false;
            }
            // Need to check 3rd byte
            // Create a position 2 bytes ahead and read from it
            let pos = self.source.position();
            // SAFETY: We just checked that there are at least 2 bytes available
            let third_pos = unsafe { pos.add(2) };
            if !third_pos.is_end_of(&self.source) {
                // SAFETY: We checked that third_pos is not at end of source
                let third_byte = unsafe { third_pos.read() };
                return third_byte == sequence[2];
            }
        }

        false
    }

    /// Check if next bytes represent an irregular line terminator (LS or PS).
    /// Uses byte-based fast path before falling back to char decoding.
    #[inline]
    fn is_next_irregular_line_terminator(&self) -> bool {
        self.check_byte_sequence(&LS_BYTES) || self.check_byte_sequence(&PS_BYTES)
    }

    /// Check if next bytes represent irregular whitespace.
    /// Uses byte-based fast path for common cases before falling back to char decoding.
    #[inline]
    fn is_next_irregular_whitespace(&self) -> bool {
        let Some(first_byte) = self.peek_byte() else {
            return false;
        };

        // Fast path for 2-byte sequences starting with 0xC2
        if first_byte == 0xC2 {
            if let Some([_, second]) = self.peek_2_bytes() {
                // Check for NBSP (0xA0), or other 2-byte irregular whitespace
                if second == 0xA0 {
                    return true; // NBSP
                }
            }
            return false;
        }

        // Fast path for 3-byte sequences starting with 0xE2
        if first_byte == 0xE2 {
            return self.check_byte_sequence(&EN_QUAD_BYTES)
                || self.check_byte_sequence(&EM_QUAD_BYTES)
                || self.check_byte_sequence(&EN_SPACE_BYTES)
                || self.check_byte_sequence(&EM_SPACE_BYTES)
                || self.check_byte_sequence(&THREE_PER_EM_SPACE_BYTES)
                || self.check_byte_sequence(&FOUR_PER_EM_SPACE_BYTES)
                || self.check_byte_sequence(&SIX_PER_EM_SPACE_BYTES)
                || self.check_byte_sequence(&FIGURE_SPACE_BYTES)
                || self.check_byte_sequence(&PUNCTUATION_SPACE_BYTES)
                || self.check_byte_sequence(&THIN_SPACE_BYTES)
                || self.check_byte_sequence(&HAIR_SPACE_BYTES)
                || self.check_byte_sequence(&NNBSP_BYTES)
                || self.check_byte_sequence(&MMSP_BYTES);
        }

        // Fast path for other 3-byte sequences
        if first_byte == 0xE1 && self.check_byte_sequence(&OGHAM_SPACE_BYTES) {
            return true;
        }

        if first_byte == 0xE3 && self.check_byte_sequence(&IDEOGRAPHIC_SPACE_BYTES) {
            return true;
        }

        if first_byte == 0xEF && self.check_byte_sequence(&ZWNBSP_BYTES) {
            return true;
        }

        false
    }

    pub(super) fn unicode_char_handler(&mut self) -> Kind {
        // Try byte-based fast paths first before decoding the full character

        // Fast path 1: Check for irregular line terminators (LS, PS)
        if self.is_next_irregular_line_terminator() {
            self.consume_char();
            self.token.set_is_on_new_line(true);
            self.trivia_builder.add_irregular_whitespace(self.token.start(), self.offset());
            return Kind::Skip;
        }

        // Fast path 2: Check for irregular whitespace
        if self.is_next_irregular_whitespace() {
            self.consume_char();
            self.trivia_builder.add_irregular_whitespace(self.token.start(), self.offset());
            return Kind::Skip;
        }

        // Slow path: Decode the character and check if it's an identifier start or error
        let c = self.peek_char().unwrap();

        if is_identifier_start_unicode(c) {
            let start_pos = self.source.position();
            self.consume_char();
            self.identifier_tail_after_unicode(start_pos);
            Kind::Ident
        } else {
            // Not an irregular whitespace/line terminator or identifier start
            // Must be an invalid character
            self.handle_invalid_unicode_char(c)
        }
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
        str: &mut StringBuilder<'a>,
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
        text: &mut StringBuilder<'a>,
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
            UnicodeEscape::CodePoint(ch) => {
                if ch == '\u{FFFD}' && self.token.lone_surrogates() {
                    // Lossy replacement character is being used as an escape marker. Escape it.
                    text.push_str("\u{FFFD}fffd");
                } else {
                    text.push(ch);
                }
            }
            UnicodeEscape::SurrogatePair(ch) => {
                // Surrogate pair is always >= 0x10000, so cannot be 0xFFFD
                text.push(ch);
            }
            UnicodeEscape::LoneSurrogate(code_point) => {
                self.string_lone_surrogate(code_point, text);
            }
        }
    }

    /// Lone surrogate found in string.
    fn string_lone_surrogate(&mut self, code_point: u32, text: &mut StringBuilder<'a>) {
        debug_assert!(code_point <= 0xFFFF);

        if !self.token.lone_surrogates() {
            self.token.set_lone_surrogates(true);

            // We use `\u{FFFD}` (the lossy replacement character) as a marker indicating the start
            // of a lone surrogate. e.g. `\u{FFFD}d800` (which will be output as `\ud800`).
            // So we need to escape any actual lossy replacement characters in the string so far.
            //
            // This could be more efficient, avoiding allocating a temporary `String`.
            // But strings containing both lone surrogates and lossy replacement characters
            // should be vanishingly rare, so don't bother.
            if let Cow::Owned(replaced) = text.cow_replace("\u{FFFD}", "\u{FFFD}fffd") {
                *text = StringBuilder::from_str_in(&replaced, self.allocator);
            }
        }

        // Encode lone surrogate as `\u{FFFD}XXXX` where XXXX is the code point as hex
        write!(text, "\u{FFFD}{code_point:04x}").unwrap();
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
        text: &mut StringBuilder<'a>,
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
                '\'' | '"' | '\\' => text.push(c),
                'b' => text.push('\u{8}'),
                'f' => text.push(FF),
                'n' => text.push(LF),
                'r' => text.push(CR),
                't' => text.push(TAB),
                'v' => text.push(VT),
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
                                text.push(c);
                            },
                        );
                }
                // UnicodeEscapeSequence
                'u' => {
                    self.string_unicode_escape_sequence(text, is_valid_escape_sequence);
                }
                // 0 [lookahead ∉ DecimalDigit]
                '0' if !self.peek_byte().is_some_and(|b| b.is_ascii_digit()) => text.push('\0'),
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
                                // `value` is between 128 and 255. UTF-8 representation is:
                                // 128-191: `0xC2`, followed by code point value.
                                // 192-255: `0xC3`, followed by code point value - 64.
                                let bytes = [0xC0 + first_digit, value & 0b1011_1111];
                                // SAFETY: `bytes` is a valid 2-byte UTF-8 sequence
                                unsafe { text.push_bytes_unchecked(&bytes) };
                                return;
                            }
                        }
                    }

                    // SAFETY: `value` is in range 0 to `((1 * 8) + 7) * 8 + 7` (127) i.e. ASCII
                    unsafe { text.push_byte_unchecked(value) };
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
                    text.push(other);
                }
            },
        }
    }
}
