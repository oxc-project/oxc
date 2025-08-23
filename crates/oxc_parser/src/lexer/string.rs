use std::cmp::max;

use oxc_allocator::StringBuilder;

use crate::diagnostics;

use super::{
    Kind, Lexer, LexerContext, Span, Token, cold_branch,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
};

/// Convert `char` to UTF-8 bytes array.
const fn to_bytes<const N: usize>(ch: char) -> [u8; N] {
    assert!(ch.len_utf8() == N);
    let mut bytes = [0u8; N];
    ch.encode_utf8(&mut bytes);
    bytes
}

/// Lossy replacement character (U+FFFD) as UTF-8 bytes.
const LOSSY_REPLACEMENT_CHAR_BYTES: [u8; 3] = to_bytes('\u{FFFD}');
const LOSSY_REPLACEMENT_CHAR_FIRST_BYTE: u8 = LOSSY_REPLACEMENT_CHAR_BYTES[0];
const _: () = assert!(LOSSY_REPLACEMENT_CHAR_FIRST_BYTE == 0xEF);

const MIN_ESCAPED_STR_LEN: usize = 16;

static DOUBLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'"' | b'\r' | b'\n' | b'\\'));

static SINGLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\'' | b'\r' | b'\n' | b'\\'));

// Same as above, but with 1st byte of lossy replacement character added
static DOUBLE_QUOTE_ESCAPED_MATCH_TABLE: SafeByteMatchTable = safe_byte_match_table!(|b| matches!(
    b,
    b'"' | b'\r' | b'\n' | b'\\' | LOSSY_REPLACEMENT_CHAR_FIRST_BYTE
));

static SINGLE_QUOTE_ESCAPED_MATCH_TABLE: SafeByteMatchTable = safe_byte_match_table!(|b| matches!(
    b,
    b'\'' | b'\r' | b'\n' | b'\\' | LOSSY_REPLACEMENT_CHAR_FIRST_BYTE
));

/// Macro to handle a string literal.
///
/// # SAFETY
/// `$delimiter` must be an ASCII byte.
/// Next char in `lexer.source` must be ASCII.
/// `$table` must be a `SafeByteMatchTable`.
/// `$table` must only match `$delimiter`, '\', '\r' or '\n'.
macro_rules! handle_string_literal {
    ($lexer:ident, $delimiter:literal, $table:ident, $escaped_table:ident) => {{
        debug_assert!($delimiter.is_ascii());

        if $lexer.context == LexerContext::JsxAttributeValue {
            // SAFETY: Caller guarantees `$delimiter` is ASCII, and next char is ASCII
            return $lexer.read_jsx_string_literal($delimiter);
        }

        // Skip opening quote.
        // SAFETY: Caller guarantees next byte is ASCII, so safe to advance past it.
        let after_opening_quote = $lexer.source.position().add(1);

        // Consume bytes which are part of string
        let next_byte = byte_search! {
            lexer: $lexer,
            table: $table,
            start: after_opening_quote,
            handle_eof: {
                $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                return Kind::Undetermined;
            },
        };

        // Found a matching byte.
        // Either end of string found, or a line break, or `\` escape.
        match next_byte {
            $delimiter => {
                // SAFETY: Macro user guarantees delimiter is ASCII, so consuming it cannot move
                // `lexer.source` off a UTF-8 character boundary.
                $lexer.source.next_byte_unchecked();
                Kind::Str
            }
            b'\\' => cold_branch(|| {
                handle_string_literal_escape!(
                    $lexer,
                    $delimiter,
                    $escaped_table,
                    after_opening_quote
                )
            }),
            _ => {
                // Line break. This is impossible in valid JS, so cold path.
                cold_branch(|| {
                    debug_assert!(matches!(next_byte, b'\r' | b'\n'));
                    $lexer.consume_char();
                    $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                    Kind::Undetermined
                })
            }
        }
    }};
}

macro_rules! handle_string_literal_escape {
    ($lexer:ident, $delimiter:literal, $table:ident, $after_opening_quote:ident) => {{
        // Create arena string to hold unescaped string.
        // We don't know how long string will end up being. Take a guess that total length
        // will be double what we've seen so far, or `MIN_ESCAPED_STR_LEN` minimum.
        let so_far = $lexer.source.str_from_pos_to_current($after_opening_quote);
        let capacity = max(so_far.len() * 2, MIN_ESCAPED_STR_LEN);
        let mut str = StringBuilder::with_capacity_in(capacity, $lexer.allocator);

        // Push chunk before `\` into `str`.
        str.push_str(so_far);

        'outer: loop {
            // Consume `\`
            let escape_start_offset = $lexer.offset();
            $lexer.consume_char();

            // Consume escape sequence and add char to `str`
            let mut is_valid_escape_sequence = true;
            $lexer.read_string_escape_sequence(&mut str, false, &mut is_valid_escape_sequence);
            if !is_valid_escape_sequence {
                let range = Span::new(escape_start_offset, $lexer.offset());
                $lexer.error(diagnostics::invalid_escape_sequence(range));
            }

            // Consume bytes until reach end of string, line break, or another escape
            let mut chunk_start = $lexer.source.position();
            while let Some(b) = $lexer.peek_byte() {
                match b {
                    b if !$table.matches(b) => {
                        // SAFETY: A byte is available, as we just peeked it.
                        // This may put `source`'s position on a UTF-8 continuation byte, which violates
                        // `Source`'s invariant temporarily, but the guarantees of `SafeByteMatchTable`
                        // mean `!table.matches(b)` on this branch prevents exiting this loop until
                        // `source` is positioned on a UTF-8 character boundary again.
                        $lexer.source.next_byte_unchecked();
                    }
                    b if b == $delimiter => {
                        // End of string found. Push last chunk to `str`.
                        let chunk = $lexer.source.str_from_pos_to_current(chunk_start);
                        str.push_str(chunk);

                        // Consume closing quote.
                        // SAFETY: Caller guarantees delimiter is ASCII, so consuming it cannot move
                        // `lexer.source` off a UTF-8 character boundary
                        $lexer.source.next_byte_unchecked();
                        break 'outer;
                    }
                    b'\\' => {
                        // Another escape found. Push last chunk to `str`, and loop back to handle escape.
                        let chunk = $lexer.source.str_from_pos_to_current(chunk_start);
                        str.push_str(chunk);
                        continue 'outer;
                    }
                    LOSSY_REPLACEMENT_CHAR_FIRST_BYTE => cold_branch(|| {
                        // If the string contains lone surrogates, the lossy replacement character (U+FFFD)
                        // is used as start of an escape sequence.
                        // So an actual lossy escape character has to be escaped too.
                        // Output it as `\u{FFFD}fffd`.
                        // Cold branch because this should be very rare in real-world code.

                        // SAFETY: A byte is available, as we just peeked it, and it's 0xEF.
                        // 0xEF is always 1st byte of a 3-byte Unicode sequence, so safe to consume 3 bytes.
                        $lexer.source.next_byte_unchecked();
                        let next1 = $lexer.source.next_byte_unchecked();
                        let next2 = $lexer.source.next_byte_unchecked();
                        if $lexer.token.lone_surrogates()
                            && [next1, next2] == [LOSSY_REPLACEMENT_CHAR_BYTES[1], LOSSY_REPLACEMENT_CHAR_BYTES[2]]
                        {
                            let chunk = $lexer.source.str_from_pos_to_current(chunk_start);
                            str.push_str(chunk);
                            str.push_str("fffd");
                            chunk_start = $lexer.source.position();
                        }
                    }),
                    _ => {
                        // Line break. This is impossible in valid JS, so cold path.
                        return cold_branch(|| {
                            debug_assert!(matches!(b, b'\r' | b'\n'));
                            $lexer.consume_char();
                            $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                            Kind::Undetermined
                        });
                    }
                }
            }

            // EOF
            $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
            return Kind::Undetermined;
        }

        // Convert `str` to arena slice and save to `escaped_strings`
        $lexer.save_string(true, str.into_str());

        Kind::Str
    }};
}

/// 12.9.4 String Literals
impl<'a> Lexer<'a> {
    /// Read string literal delimited with `"`.
    /// # SAFETY
    /// Next character must be `"`.
    pub(super) unsafe fn read_string_literal_double_quote(&mut self) -> Kind {
        // SAFETY: Caller guarantees next char is `"`, which is ASCII.
        // b'"' is an ASCII byte. `DOUBLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        unsafe {
            handle_string_literal!(
                self,
                b'"',
                DOUBLE_QUOTE_STRING_END_TABLE,
                DOUBLE_QUOTE_ESCAPED_MATCH_TABLE
            )
        }
    }

    /// Read string literal delimited with `'`.
    /// # SAFETY
    /// Next character must be `'`.
    pub(super) unsafe fn read_string_literal_single_quote(&mut self) -> Kind {
        // SAFETY: Caller guarantees next char is `'`, which is ASCII.
        // b'\'' is an ASCII byte. `SINGLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        unsafe {
            handle_string_literal!(
                self,
                b'\'',
                SINGLE_QUOTE_STRING_END_TABLE,
                SINGLE_QUOTE_ESCAPED_MATCH_TABLE
            )
        }
    }

    /// Save the string if it is escaped
    /// This reduces the overall memory consumption while keeping the `Token` size small
    /// Strings without escaped values can be retrieved as is from the token span
    pub(super) fn save_string(&mut self, has_escape: bool, s: &'a str) {
        if has_escape {
            self.save_escaped_string(s);
        }
    }

    #[cold]
    fn save_escaped_string(&mut self, s: &'a str) {
        self.escaped_strings.insert(self.token.start(), s);
        self.token.set_escaped(true);
    }

    pub(crate) fn get_string(&self, token: Token) -> &'a str {
        if token.escaped() {
            return self.escaped_strings[&token.start()];
        }

        let raw = &self.source.whole()[token.start() as usize..token.end() as usize];
        match token.kind() {
            Kind::Str => {
                &raw[1..raw.len() - 1] // omit surrounding quotes
            }
            Kind::PrivateIdentifier => {
                &raw[1..] // omit leading `#`
            }
            _ => raw,
        }
    }
}
