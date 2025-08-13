use super::{
    Kind, Lexer,
    search::{SafeByteMatchTable, byte_search_raw, safe_byte_match_table},
};

static NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'));

impl Lexer<'_> {
    pub(super) fn line_break_handler(&mut self) -> Kind {
        self.token.set_is_on_new_line(true);
        self.trivia_builder.handle_newline();

        // Indentation is common after a line break.
        // Consume it, along with any further line breaks.
        // Irregular line breaks and whitespace are not consumed.
        // They're uncommon, so leave them for the next call to `handle_byte` to take care of.
        // Use raw search function to avoid borrowing issues
        if let Some((_byte, pos)) = byte_search_raw(
            self,
            &NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE,
            self.source.position(),
        ) {
            self.source.set_position(pos);
        } else {
            // EOF
            self.source.advance_to_end();
        }

        Kind::Skip
    }
}
