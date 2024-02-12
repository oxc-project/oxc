use super::{
    search::{byte_search, simd_byte_match_table, SimdByteMatchTable},
    Kind, Lexer,
};
use once_cell::sync::Lazy;

static NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE: Lazy<SimdByteMatchTable> =
    simd_byte_match_table!(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'), true);

impl<'a> Lexer<'a> {
    pub(super) fn line_break_handler(&mut self) -> Kind {
        self.token.is_on_new_line = true;

        // Indentation is common after a line break.
        // Consume it, along with any further line breaks.
        // Irregular line breaks and whitespace are not consumed.
        // They're uncommon, so leave them for the next call to `handle_byte` to take care of.
        byte_search! {
            lexer: self,
            table: NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE,
            handle_match: |_next_byte, _start| {
                Kind::Skip
            },
            handle_eof: |_start| {
                Kind::Skip
            },
        };
    }
}
