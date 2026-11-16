use crate::config::LexerConfig as Config;

use super::{
    Kind, Lexer,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
};

static NOT_REGULAR_WHITESPACE_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !matches!(b, b' ' | b'\t'));

static NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'));

impl<C: Config> Lexer<'_, C> {
    pub(super) fn whitespace_handler(&mut self) -> Kind {
        // Single spaces between tokens are consumed directly by `read_next_token`.
        // Scan additional spaces and tabs in one pass to avoid repeated token-loop
        // checks, token-start updates, and indirect byte-handler calls.
        // Leave line breaks and irregular whitespace to their handlers, which
        // update newline state and record trivia.
        byte_search! {
            lexer: self,
            table: NOT_REGULAR_WHITESPACE_TABLE,
        };

        Kind::Skip
    }

    /// Skip regular whitespace after a line break, returning the next byte.
    ///
    /// At EOF, returns 0 - the first byte of the padding which follows the source text.
    /// The byte handler for NUL checks for EOF.
    pub(super) fn line_break_handler(&mut self) -> u8 {
        self.pending.set_is_on_new_line();
        self.trivia_builder.handle_newline(&mut self.pending);

        // Indentation is common after a line break.
        // Consume it, along with any further line breaks.
        // Irregular line breaks and whitespace are not consumed.
        // They're uncommon, so leave them for the next call to `handle_byte` to take care of.
        byte_search! {
            lexer: self,
            table: NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE,
        }
    }
}
