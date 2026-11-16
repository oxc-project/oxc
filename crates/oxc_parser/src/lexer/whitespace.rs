use crate::config::LexerConfig as Config;

use super::{
    Kind, Lexer, PendingToken, SourcePosition, Token, byte_handlers,
    identifier::is_identifier_start_ascii_byte,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
};

static NOT_REGULAR_WHITESPACE_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !matches!(b, b' ' | b'\t'));

static NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'));

impl<'a, C: Config> Lexer<'a, C> {
    pub(super) fn whitespace_handler(&mut self) -> Kind {
        // Single spaces between tokens are consumed directly by `read_next_token`.
        // Scan additional spaces and tabs in one pass to avoid repeated token-loop
        // checks, token-start updates, and indirect byte-handler calls.
        // Leave line breaks and irregular whitespace to their handlers, which
        // update newline state and record trivia.
        byte_search! {
            lexer: self,
            table: NOT_REGULAR_WHITESPACE_TABLE,
            handle_eof: 0,
        };

        Kind::Skip
    }

    /// Skip regular whitespace after a line break, returning the next byte or `None` at EOF.
    pub(super) fn line_break_handler(&mut self) -> Option<u8> {
        self.pending.set_is_on_new_line();
        self.trivia_builder.handle_newline(&mut self.pending);

        // Indentation is common after a line break.
        // Consume it, along with any further line breaks.
        // Irregular line breaks and whitespace are not consumed.
        // They're uncommon, so leave them for the next call to `handle_byte` to take care of.
        let byte = byte_search! {
            lexer: self,
            table: NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE,
            handle_eof: return None,
        };

        Some(byte)
    }

    /// Read the next token, where the first trivia seen since the previous token is the line break
    /// at `pos`.
    ///
    /// A line break is usually followed by indentation, then a keyword, an identifier or `}`.
    /// Those tokens are finished here, without the work which `finish_next_inner` does
    /// for flags and comments. All that needs to be recorded is that they are on a new line.
    ///
    /// # SAFETY
    /// * `pos` must be the position of a `\r` or `\n` byte.
    /// * No trivia must have been seen since the previous token, other than one space.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) unsafe fn line_break_token(&mut self, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `pos` is the position of an ASCII byte
        let after_line_break = unsafe { pos.add(1) };

        // Consume indentation, along with any further line breaks
        let byte = byte_search! {
            lexer: self,
            table: NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE,
            start: after_line_break,
            // A byte which does not start a token, so that EOF takes the general path below
            handle_eof: b' ',
        };
        let pos = self.source.position();

        if is_identifier_start_ascii_byte(byte) {
            // Keywords are more common than identifiers at the start of a line,
            // so check for a keyword whatever the first letter is.
            // SAFETY: `byte` is the byte at `pos`, and it is ASCII
            return unsafe { self.identifier_token::<true, true>(pos) };
        }

        if let Some(kind) = byte_handlers::single_byte_token(byte) {
            // SAFETY: `byte` is the byte at `pos`, and single-byte tokens are ASCII
            let end = unsafe { pos.add(1) };
            self.source.set_position(end);
            return self.finish_fresh_token_on_new_line(kind, pos, end);
        }

        // Comments, strings, and everything else.
        // No trivia was seen before the line break, so this is all that needs to be recorded.
        self.pending = PendingToken::AFTER_LINE_BREAK;
        self.next_token_after_trivia()
    }
}
