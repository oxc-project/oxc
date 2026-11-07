use std::cmp::max;

use oxc_str::{JSStr, JSStrBuilder};

use crate::{config::LexerConfig as Config, diagnostics};

use super::{
    Kind, Lexer, Span, Token, cold_branch,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
};

const MIN_ESCAPED_STR_LEN: usize = 16;

static DOUBLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'"' | b'\r' | b'\n' | b'\\'));

static SINGLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\'' | b'\r' | b'\n' | b'\\'));

/// Find the next quote, escape, or line break after an escape sequence.
#[inline(always)]
fn find_string_chunk_end<const DELIMITER: u8>(
    bytes: &[u8],
    table: &SafeByteMatchTable,
) -> Option<usize> {
    const PREFIX_LEN: usize = 16;
    // Handle short chunks inline before paying for the two bulk searches.
    let Some(prefix) = bytes.get(..PREFIX_LEN) else {
        return bytes.iter().position(|&byte| table.matches(byte));
    };
    for (index, &byte) in prefix.iter().enumerate() {
        if table.matches(byte) {
            return Some(index);
        }
    }

    let remaining = &bytes[PREFIX_LEN..];
    let end = memchr::memchr3(DELIMITER, b'\\', b'\n', remaining);
    // A carriage return before the other delimiters ends the chunk first.
    memchr::memchr(b'\r', &remaining[..end.unwrap_or(remaining.len())])
        .or(end)
        .map(|index| PREFIX_LEN + index)
}

/// Macro to handle a string literal.
///
/// # SAFETY
/// `$delimiter` must be an ASCII byte.
/// Next char in `lexer.source` must be ASCII.
/// `$table` must be a `SafeByteMatchTable`.
/// `$table` must only match `$delimiter`, '\', '\r' or '\n'.
macro_rules! handle_string_literal {
    ($lexer:ident, $delimiter:literal, $table:ident) => {{
        debug_assert!($delimiter.is_ascii());

        // Skip opening quote.
        // SAFETY: Caller guarantees next byte is ASCII, so safe to advance past it.
        let after_opening_quote = $lexer.source.position().add(1);

        // Consume bytes which are part of string
        let next_byte = byte_search! {
            lexer: $lexer,
            table: $table,
            start: after_opening_quote,
            handle_eof: {
                // Unterminated string is invalid JS, so a cold path. Building the diagnostic
                // out-of-line keeps the `OxcDiagnostic` return buffer out of this handler's
                // stack frame (matching the sibling `\\` and line-break arms below).
                return cold_branch(|| {
                    $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                    Kind::Undetermined
                });
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
                handle_string_literal_escape!($lexer, $delimiter, $table, after_opening_quote)
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
        let mut str = JSStrBuilder::with_capacity_in(capacity, &$lexer.allocator);

        // Push chunk before `\` into `str`.
        str.push_str(so_far);

        loop {
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
            let chunk_start = $lexer.source.position();
            let remaining = $lexer.source.remaining().as_bytes();
            let Some(end) = find_string_chunk_end::<$delimiter>(remaining, &$table) else {
                $lexer.source.advance_to_end();
                $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                return Kind::Undetermined;
            };
            let b = remaining[end];
            // SAFETY: The match is an in-bounds ASCII byte, hence a UTF-8 character boundary.
            $lexer.source.set_position(chunk_start.add(end));
            match b {
                $delimiter => {
                    // End of string found. Push last chunk to `str`.
                    let chunk = $lexer.source.str_from_pos_to_current(chunk_start);
                    str.push_str(chunk);

                    // Consume closing quote.
                    // SAFETY: Caller guarantees delimiter is ASCII, so consuming it cannot move
                    // `lexer.source` off a UTF-8 character boundary
                    $lexer.source.next_byte_unchecked();
                    break;
                }
                b'\\' => {
                    // Another escape found. Push last chunk to `str`, and loop back to handle escape.
                    let chunk = $lexer.source.str_from_pos_to_current(chunk_start);
                    str.push_str(chunk);
                }
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

        $lexer.save_js_string(str.into_js_str());

        Kind::Str
    }};
}

/// 12.9.4 String Literals
impl<'a, C: Config> Lexer<'a, C> {
    /// Read string literal delimited with `"`.
    /// # SAFETY
    /// Next character must be `"`.
    pub(super) unsafe fn read_string_literal_double_quote(&mut self) -> Kind {
        // SAFETY: Caller guarantees next char is `"`, which is ASCII.
        // b'"' is an ASCII byte. `DOUBLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        unsafe { handle_string_literal!(self, b'"', DOUBLE_QUOTE_STRING_END_TABLE) }
    }

    /// Read string literal delimited with `'`.
    /// # SAFETY
    /// Next character must be `'`.
    pub(super) unsafe fn read_string_literal_single_quote(&mut self) -> Kind {
        // SAFETY: Caller guarantees next char is `'`, which is ASCII.
        // b'\'' is an ASCII byte. `SINGLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        unsafe { handle_string_literal!(self, b'\'', SINGLE_QUOTE_STRING_END_TABLE) }
    }

    fn save_js_string(&mut self, value: JSStr<'a>) {
        self.escaped_strings.insert(self.token.start(), value);
        self.token.set_escaped(true);
    }

    pub(crate) fn get_js_string(&self, token: Token) -> JSStr<'a> {
        debug_assert_eq!(token.kind(), Kind::Str);
        if token.escaped() {
            self.escaped_strings[&token.start()]
        } else {
            // Exclude the quotes before widening offsets, so JSStr's length stays bounded by u32.
            let start = (token.start() + 1) as usize;
            let end = (token.end() - 1) as usize;
            JSStr::from(&self.source.whole()[start..end])
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
        self.escaped_strings.insert(self.token.start(), s.into());
        self.token.set_escaped(true);
    }

    pub(crate) fn get_string(&self, token: Token) -> &'a str {
        if token.escaped() {
            // Identifiers only save validated Unicode scalar values.
            // Decoded string literals and templates use `get_js_string` instead.
            return self.escaped_strings[&token.start()]
                .as_str()
                .expect("identifier escape must be valid UTF-8");
        }

        let source_text = self.source.whole();
        let mut start = token.start() as usize;
        let mut end = token.end() as usize;
        match token.kind() {
            Kind::Str => {
                // Omit surrounding quotes
                start += 1;
                end -= 1;
            }
            Kind::PrivateIdentifier => {
                // Omit leading `#`
                start += 1;
            }
            _ => {}
        }
        &source_text[start..end]
    }
}
