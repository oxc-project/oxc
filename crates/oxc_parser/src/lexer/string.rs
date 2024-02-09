use super::{
    search::{byte_search, safe_byte_match_table, SafeByteMatchTable},
    source::SourcePosition,
    Kind, Lexer, LexerContext, Span, Token,
};
use crate::diagnostics;

use oxc_allocator::String;
use std::cmp::max;

const MIN_ESCAPED_STR_LEN: usize = 16;

static DOUBLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'"' | b'\r' | b'\n' | b'\\'));

static SINGLE_QUOTE_STRING_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\'' | b'\r' | b'\n' | b'\\'));

impl<'a> Lexer<'a> {
    /// 12.9.4 String Literals

    /// Read string literal delimited with `"`.
    /// # SAFETY
    /// Next character must be `"`.
    pub(super) unsafe fn read_string_literal_double_quote(&mut self) -> Kind {
        if self.context == LexerContext::JsxAttributeValue {
            self.consume_char();
            self.read_jsx_string_literal('"')
        } else {
            // SAFETY: `DOUBLE_QUOTE_STRING_END_TABLE` matches all non-ASCII bytes
            self.read_string_literal(b'"', &DOUBLE_QUOTE_STRING_END_TABLE)
        }
    }

    /// Read string literal delimited with `'`.
    /// # SAFETY
    /// Next character must be `'`.
    pub(super) unsafe fn read_string_literal_single_quote(&mut self) -> Kind {
        if self.context == LexerContext::JsxAttributeValue {
            self.consume_char();
            self.read_jsx_string_literal('\'')
        } else {
            // SAFETY: `SINGLE_QUOTE_STRING_END_TABLE` matches all non-ASCII bytes
            self.read_string_literal(b'\'', &SINGLE_QUOTE_STRING_END_TABLE)
        }
    }

    /// Read string literal.
    /// # SAFETY
    /// Next byte must be ASCII.
    unsafe fn read_string_literal(&mut self, delimiter: u8, table: &SafeByteMatchTable) -> Kind {
        // Skip opening quote.
        // SAFETY: Caller guarantees next byte is ASCII, so safe to advance past it.
        let after_opening_quote = unsafe { self.source.position().add(1) };

        // Consume bytes which are part of identifier
        byte_search! {
            lexer: self,
            table: table,
            start: after_opening_quote,
            handle_match: |next_byte| {
                // Found a matching byte.
                // Either end of string found, or a line break, or `\` escape.
                if next_byte == delimiter {
                    self.consume_char();
                    return Kind::Str;
                }

                if next_byte == b'\\' {
                    return self.string_literal_on_escape(delimiter, table, after_opening_quote);
                }

                debug_assert!(matches!(next_byte, b'\r' | b'\n'));
                self.consume_char();
                self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                Kind::Undetermined
            },
            handle_eof: || {
                self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                Kind::Undetermined
            },
        };
    }

    /// Process string literal when `\` escape found.
    #[cold]
    fn string_literal_on_escape(
        &mut self,
        delimiter: u8,
        table: &SafeByteMatchTable,
        after_opening_quote: SourcePosition,
    ) -> Kind {
        // Create arena string to hold unescaped string.
        // We don't know how long string will end up being. Take a guess that total length
        // will be double what we've seen so far, or `MIN_ESCAPED_STR_LEN` minimum.
        let so_far = self.source.str_from_pos_to_current(after_opening_quote);
        let capacity = max(so_far.len() * 2, MIN_ESCAPED_STR_LEN);
        let mut str = String::with_capacity_in(capacity, self.allocator);

        // Push chunk before `\` into `str`.
        // `bumpalo::collections::string::String::push_str` is currently expensive due to
        // inefficiency in bumpalo's implementation. But best we have right now.
        str.push_str(so_far);

        'outer: loop {
            // Consume `\`
            let escape_start_offset = self.offset();
            self.consume_char();

            // Consume escape sequence and add char to `str`
            let mut is_valid_escape_sequence = true;
            self.read_string_escape_sequence(&mut str, false, &mut is_valid_escape_sequence);
            if !is_valid_escape_sequence {
                let range = Span::new(escape_start_offset, self.offset());
                self.error(diagnostics::InvalidEscapeSequence(range));
            }

            // Consume bytes until reach end of string, line break, or another escape
            let chunk_start = self.source.position();
            while let Some(b) = self.source.peek_byte() {
                if !table.matches(b) {
                    // SAFETY: A byte is available, as we just peeked it.
                    // This may put `source`'s position on a UTF-8 continuation byte, which violates
                    // `Source`'s invariant temporarily, but the guarantees of `SafeByteMatchTable`
                    // mean `table.matches(b)` will always return `true` in a pattern where
                    // we can't exit this loop without `source` being positioned on a UTF-8 character
                    // boundary again.
                    unsafe { self.source.next_byte_unchecked() };
                    continue;
                }

                if b == delimiter {
                    // End of string found. Push last chunk to `str`, and consume closing quote.
                    let chunk = self.source.str_from_pos_to_current(chunk_start);
                    str.push_str(chunk);
                    self.consume_char();
                    break 'outer;
                }

                if b == b'\\' {
                    // Another escape found. Push last chunk to `str`, and loop back to handle escape.
                    let chunk = self.source.str_from_pos_to_current(chunk_start);
                    str.push_str(chunk);
                    continue 'outer;
                }

                debug_assert!(matches!(b, b'\r' | b'\n'));
                self.consume_char();
                break;
            }

            // EOF
            self.error(diagnostics::UnterminatedString(self.unterminated_range()));
            return Kind::Undetermined;
        }

        // Convert `str` to arena slice and save to `escaped_strings`
        self.save_string(true, str.into_bump_str());

        Kind::Str
    }

    /// Save the string if it is escaped
    /// This reduces the overall memory consumption while keeping the `Token` size small
    /// Strings without escaped values can be retrieved as is from the token span
    pub(super) fn save_string(&mut self, has_escape: bool, s: &'a str) {
        if !has_escape {
            return;
        }
        self.escaped_strings.insert(self.token.start, s);
        self.token.escaped = true;
    }

    pub(crate) fn get_string(&self, token: Token) -> &'a str {
        if token.escaped {
            return self.escaped_strings[&token.start];
        }

        let raw = &self.source.whole()[token.start as usize..token.end as usize];
        match token.kind {
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
