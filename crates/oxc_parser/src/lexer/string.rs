use std::cmp::max;

use oxc_str::{JSStr, JSStrBuilder};

use crate::{config::LexerConfig as Config, diagnostics};

use super::{
    FinishTokenMode, Kind, Lexer, SourcePosition, Span, Token, cold_branch,
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
        let pos = $lexer.source.position();
        match handle_plain_string_literal!($lexer, pos, $delimiter, $table) {
            Ok(_) => Kind::Str,
            // An escape, or an unterminated string, which is invalid JS. Either way, a cold path.
            // Building the diagnostic out-of-line keeps the `OxcDiagnostic` return buffer out of
            // this handler's stack frame.
            Err(next_byte) => cold_branch(|| {
                handle_string_literal_rest!($lexer, pos, $delimiter, $table, next_byte)
            }),
        }
    }};
}

/// Macro to read a string literal up to its closing quote, or to the first thing in it which is not
/// plain text. The opening quote is at `$pos`.
///
/// Evaluates to `Ok(end)` if the string contains no escapes. `end` is the position after the closing
/// quote, and position of `lexer.source` has been set to it.
///
/// Otherwise evaluates to `Err(byte)`, and `handle_string_literal_rest!` handles the rest of the string:
/// * `byte` is `\`, `\r` or `\n`: position of `lexer.source` is on that byte.
/// * `byte` is 0: the source ended. Position of `lexer.source` is the end of the source.
///
/// # SAFETY
/// `$delimiter` must be an ASCII byte.
/// `$pos` must be the position of a `$delimiter` byte.
/// `$table` must be a `SafeByteMatchTable`.
/// `$table` must only match `$delimiter`, '\', '\r' or '\n'.
macro_rules! handle_plain_string_literal {
    ($lexer:ident, $pos:ident, $delimiter:literal, $table:ident) => {{
        debug_assert!($delimiter.is_ascii());

        // Skip opening quote.
        // SAFETY: Caller guarantees byte at `$pos` is ASCII, so safe to advance past it.
        let after_opening_quote = $pos.add(1);

        // Consume bytes which are part of string
        let next_byte = byte_search! {
            lexer: $lexer,
            table: $table,
            start: after_opening_quote,
            // `$table` does not match 0, so the search only produces it at EOF
            handle_eof: 0,
        };

        if next_byte == $delimiter {
            // SAFETY: Macro user guarantees delimiter is ASCII, so consuming it cannot move
            // `lexer.source` off a UTF-8 character boundary.
            let end = $lexer.source.position().add(1);
            $lexer.source.set_position(end);
            Ok(end)
        } else {
            Err(next_byte)
        }
    }};
}

/// Macro to handle the rest of a string literal whose opening quote is at `$pos`,
/// after `handle_plain_string_literal!` evaluated to `Err($next_byte)` for it.
///
/// Evaluates to the `Kind` of the token. May also return it from the enclosing function.
///
/// # SAFETY
/// As for `handle_plain_string_literal!`.
/// Position of `lexer.source` must be where `handle_plain_string_literal!` left it.
macro_rules! handle_string_literal_rest {
    ($lexer:ident, $pos:ident, $delimiter:literal, $table:ident, $next_byte:ident) => {{
        match $next_byte {
            b'\\' => {
                // SAFETY: Macro user guarantees byte at `$pos` is ASCII, so safe to advance past it.
                let after_opening_quote = $pos.add(1);
                handle_string_literal_escape!($lexer, $delimiter, $table, after_opening_quote)
            }
            0 => {
                // End of source
                $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                Kind::Undetermined
            }
            _ => {
                // Line break
                debug_assert!(matches!($next_byte, b'\r' | b'\n'));
                $lexer.consume_char();
                $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                Kind::Undetermined
            }
        }
    }};
}

macro_rules! handle_string_literal_escape {
    ($lexer:ident, $delimiter:literal, $table:ident, $after_opening_quote:ident) => {{
        let so_far = $lexer.source.str_from_pos_to_current($after_opening_quote);
        if so_far.is_empty() && $lexer.read_single_character_escape::<$delimiter>() {
            return Kind::Str;
        }

        // Decode the first escape before allocating. Each escape produces at most one
        // code point (or no text for a line continuation).
        let escape_start_offset = $lexer.offset();
        $lexer.consume_char();
        let after_backslash = $lexer.source.position();
        let mut is_valid_escape_sequence = true;
        let first = $lexer.decode_string_escape_sequence(false, &mut is_valid_escape_sequence);
        if !is_valid_escape_sequence {
            let range = Span::new(escape_start_offset, $lexer.offset());
            $lexer.error(diagnostics::invalid_escape_sequence(range));
        }

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
        let chunk = $lexer.source.str_from_pos_to_current(chunk_start);
        if b != $delimiter && b != b'\\' {
            return cold_branch(|| {
                debug_assert!(matches!(b, b'\r' | b'\n'));
                $lexer.consume_char();
                $lexer.error(diagnostics::unterminated_string($lexer.unterminated_range()));
                Kind::Undetermined
            });
        }

        if b == $delimiter && is_valid_escape_sequence {
            // Identity escapes and one-sided line continuations can leave the decoded
            // value in one contiguous source span.
            let borrowed = match first {
                None if so_far.is_empty() => Some(chunk),
                None if chunk.is_empty() => Some(so_far),
                Some(value) if so_far.is_empty() => value.to_char().and_then(|c| {
                    let source_value = $lexer.source.str_from_pos_to_current(after_backslash);
                    let escape_len = source_value.len() - chunk.len();
                    // Check the whole escape payload, so `\u0075` cannot borrow `u0075`.
                    (escape_len == c.len_utf8() && source_value.starts_with(c))
                        .then_some(source_value)
                }),
                _ => None,
            };
            if let Some(value) = borrowed {
                // SAFETY: The closing quote is ASCII, so consuming it preserves a UTF-8 boundary.
                $lexer.source.next_byte_unchecked();
                $lexer.save_js_string(value.into());
                return Kind::Str;
            }
        }

        // With no further escape, the final size is known. The source chunks are UTF-8,
        // so neither can form a surrogate pair with the decoded escape.
        let capacity = if b == $delimiter {
            so_far.len() + first.map_or(0, |c| c.len_bytes()) + chunk.len()
        } else {
            max(so_far.len() * 2, MIN_ESCAPED_STR_LEN)
        };
        let mut str = JSStrBuilder::with_capacity_in(capacity, &$lexer.allocator);
        str.push_str(so_far);
        if let Some(first) = first {
            if let Some(c) = first.to_char() {
                str.push(c);
            } else {
                str.push_js_char(first);
            }
        }
        str.push_str(chunk);

        if b == $delimiter {
            // SAFETY: The closing quote is ASCII, so consuming it preserves a UTF-8 boundary.
            $lexer.source.next_byte_unchecked();
            $lexer.save_js_string(str.into_js_str());
            return Kind::Str;
        }
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
    /// Decode a single common escape without growing an arena string.
    #[inline]
    fn read_single_character_escape<const DELIMITER: u8>(&mut self) -> bool {
        let remaining = self.source.remaining();
        let [b'\\', escape, delimiter, ..] = remaining.as_bytes() else {
            return false;
        };
        if *delimiter != DELIMITER {
            return false;
        }
        let value = match escape {
            b'\'' => "'",
            b'\"' => "\"",
            b'\\' => "\\",
            b'b' => "\u{8}",
            b'f' => "\u{c}",
            b'n' => "\n",
            b'r' => "\r",
            b't' => "\t",
            b'v' => "\u{b}",
            // The closing quote guarantees this cannot start an octal escape.
            b'0' => "\0",
            _ => return false,
        };
        // Raw transfer requires nonempty strings to reside in the source or arena.
        let value = if matches!(escape, b'\'' | b'"' | b'\\') {
            JSStr::from(&remaining[1..2])
        } else {
            JSStr::from_str_in(value, &self.allocator)
        };
        // SAFETY: The matched escape and closing quote are three in-bounds ASCII bytes.
        let end = unsafe { self.source.position().add(3) };
        self.source.set_position(end);
        self.save_js_string(value);
        true
    }

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

    /// Read string literal delimited with `"`, up to its closing quote, or to the first thing in it
    /// which is not plain text.
    ///
    /// Returns `Ok(end)` if the string contains no escapes. `end` is the position after the closing
    /// quote, which position of source has been set to.
    ///
    /// Otherwise returns `Err(next_byte)`. `string_token_rest_double_quote` reads the rest.
    ///
    /// # SAFETY
    /// `pos` must be the position of a `"`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) unsafe fn read_plain_string_literal_double_quote(
        &mut self,
        pos: SourcePosition<'a>,
    ) -> Result<SourcePosition<'a>, u8> {
        // SAFETY: Caller guarantees byte at `pos` is `"`, which is ASCII.
        // `DOUBLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        unsafe { handle_plain_string_literal!(self, pos, b'"', DOUBLE_QUOTE_STRING_END_TABLE) }
    }

    /// Read string literal delimited with `'`, up to its closing quote, or to the first thing in it
    /// which is not plain text.
    ///
    /// Returns `Ok(end)` if the string contains no escapes. `end` is the position after the closing
    /// quote, which position of source has been set to.
    ///
    /// Otherwise returns `Err(next_byte)`. `string_token_rest_single_quote` reads the rest.
    ///
    /// # SAFETY
    /// `pos` must be the position of a `'`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) unsafe fn read_plain_string_literal_single_quote(
        &mut self,
        pos: SourcePosition<'a>,
    ) -> Result<SourcePosition<'a>, u8> {
        // SAFETY: Caller guarantees byte at `pos` is `'`, which is ASCII.
        // `SINGLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        unsafe { handle_plain_string_literal!(self, pos, b'\'', SINGLE_QUOTE_STRING_END_TABLE) }
    }

    /// Read the rest of the string literal whose opening `"` is at `pos`, and finish the token,
    /// after `read_plain_string_literal_double_quote` returned `Err(next_byte)` for it.
    ///
    /// The string contains an escape, or is unterminated. Neither is common.
    /// `pending` must reflect the trivia seen since the previous token.
    ///
    /// # SAFETY
    /// * `pos` must be the position of a `"`.
    /// * Position of source must be where `read_plain_string_literal_double_quote` left it.
    #[cold]
    #[inline(never)]
    pub(super) unsafe fn string_token_rest_double_quote(
        &mut self,
        pos: SourcePosition<'a>,
        next_byte: u8,
    ) -> Token {
        // Escaped strings and diagnostics are recorded against `token_start`
        self.token_start = self.source.offset_of(pos);
        // SAFETY: Caller guarantees byte at `pos` is `"`, which is ASCII, and that position
        // of source is as `handle_plain_string_literal!` left it.
        // `DOUBLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        let kind = cold_branch(|| unsafe {
            handle_string_literal_rest!(self, pos, b'"', DOUBLE_QUOTE_STRING_END_TABLE, next_byte)
        });
        self.finish_next_inner(kind, self.source.position(), FinishTokenMode::Push)
    }

    /// Read the rest of the string literal whose opening `'` is at `pos`, and finish the token,
    /// after `read_plain_string_literal_single_quote` returned `Err(next_byte)` for it.
    ///
    /// The string contains an escape, or is unterminated. Neither is common.
    /// `pending` must reflect the trivia seen since the previous token.
    ///
    /// # SAFETY
    /// * `pos` must be the position of a `'`.
    /// * Position of source must be where `read_plain_string_literal_single_quote` left it.
    #[cold]
    #[inline(never)]
    pub(super) unsafe fn string_token_rest_single_quote(
        &mut self,
        pos: SourcePosition<'a>,
        next_byte: u8,
    ) -> Token {
        // Escaped strings and diagnostics are recorded against `token_start`
        self.token_start = self.source.offset_of(pos);
        // SAFETY: Caller guarantees byte at `pos` is `'`, which is ASCII, and that position
        // of source is as `handle_plain_string_literal!` left it.
        // `SINGLE_QUOTE_STRING_END_TABLE` is a `SafeByteMatchTable`.
        let kind = cold_branch(|| unsafe {
            handle_string_literal_rest!(self, pos, b'\'', SINGLE_QUOTE_STRING_END_TABLE, next_byte)
        });
        self.finish_next_inner(kind, self.source.position(), FinishTokenMode::Push)
    }

    fn save_js_string(&mut self, value: JSStr<'a>) {
        self.escaped_strings.insert(self.token_start, value);
        self.pending.set_escaped();
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
        self.escaped_strings.insert(self.token_start, s.into());
        self.pending.set_escaped();
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
