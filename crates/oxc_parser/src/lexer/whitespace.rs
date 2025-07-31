use super::{
    Kind, Lexer,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
    simd_search::{skip_ascii_whitespace_fallback, skip_ascii_whitespace_simd},
};

static NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'));

impl Lexer<'_> {
    pub(super) fn line_break_handler(&mut self) -> Kind {
        self.token.set_is_on_new_line(true);
        self.trivia_builder.handle_newline();

        // Use SIMD-optimized whitespace skipping for better performance
        let remaining = self.source.remaining().as_bytes();
        let consumed = if remaining.len() >= 32 {
            // Use SIMD for larger chunks
            skip_ascii_whitespace_simd(remaining)
        } else {
            // Use fallback for small chunks to avoid SIMD overhead
            skip_ascii_whitespace_fallback(remaining)
        };

        // Advance the source by the number of whitespace bytes consumed
        if consumed > 0 {
            for _ in 0..consumed {
                if self.source.next_char().is_none() {
                    break;
                }
            }
        }

        // Fall back to original implementation for any remaining irregular whitespace
        byte_search! {
            lexer: self,
            table: NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE,
            handle_eof: 0, // Fall through to below
        };

        Kind::Skip
    }
}
