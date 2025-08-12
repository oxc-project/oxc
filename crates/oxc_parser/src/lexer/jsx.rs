use memchr::memchr;

use oxc_span::Span;
use oxc_syntax::identifier::is_identifier_part;

use crate::diagnostics;

use super::{
    Kind, Lexer, Token, cold_branch,
    search::{SEARCH_BATCH_SIZE, SafeByteMatchTable, safe_byte_match_table},
};

static NOT_ASCII_JSX_ID_CONTINUE_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'-')));

static JSX_CHILD_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| b == b'{' || b == b'}' || b == b'>' || b == b'<');

/// `JSXDoubleStringCharacters` ::
///   `JSXDoubleStringCharacter` `JSXDoubleStringCharactersopt`
/// `JSXDoubleStringCharacter` ::
///   `JSXStringCharacter` but not "
/// `JSXSingleStringCharacters` ::
///   `JSXSingleStringCharacter` `JSXSingleStringCharactersopt`
/// `JSXSingleStringCharacter` ::
///   `JSXStringCharacter` but not '
/// `JSXStringCharacter` ::
///   `SourceCharacter` but not one of `HTMLCharacterReference`
impl Lexer<'_> {
    /// Read JSX string literal.
    /// # SAFETY
    /// * `delimiter` must be an ASCII character.
    /// * Next char in `lexer.source` must be ASCII.
    pub(super) unsafe fn read_jsx_string_literal(&mut self, delimiter: u8) -> Kind {
        debug_assert!(delimiter.is_ascii());

        // Skip opening quote
        // SAFETY: Caller guarantees next byte is ASCII, so `.add(1)` is a UTF-8 char boundary
        let after_opening_quote = unsafe { self.source.position().add(1) };
        let remaining = self.source.str_from_pos_to_end(after_opening_quote);

        let len = memchr(delimiter, remaining.as_bytes());
        if let Some(len) = len {
            // SAFETY: `after_opening_quote` + `len` is position of delimiter.
            // Caller guarantees delimiter is ASCII, so 1 byte after it is a UTF-8 char boundary.
            let after_closing_quote = unsafe { after_opening_quote.add(len + 1) };
            self.source.set_position(after_closing_quote);
            Kind::Str
        } else {
            self.source.advance_to_end();
            self.error(diagnostics::unterminated_string(self.unterminated_range()));
            Kind::Eof
        }
    }

    pub(crate) fn next_jsx_child(&mut self) -> Token {
        self.token.set_start(self.offset());
        let kind = self.read_jsx_child();
        self.finish_next(kind)
    }

    /// [`JSXChild`](https://facebook.github.io/jsx/#prod-JSXChild)
    /// `JSXChild` :
    /// `JSXText`
    /// `JSXElement`
    /// `JSXFragment`
    /// { `JSXChildExpressionopt` }
    fn read_jsx_child(&mut self) -> Kind {
        match self.peek_byte() {
            Some(b'<') => {
                self.consume_char();
                Kind::LAngle
            }
            Some(b'{') => {
                self.consume_char();
                Kind::LCurly
            }
            Some(_) => {
                let next_byte = {
                    // Inlined byte_search! macro
                    #[allow(clippy::unnecessary_safety_comment, clippy::allow_attributes)]
                    JSX_CHILD_END_TABLE.use_table();

                    let mut pos = self.source.position();
                    // Silence warnings if macro called in unsafe code
                    #[allow(
                        unused_unsafe,
                        clippy::unnecessary_safety_comment,
                        clippy::allow_attributes
                    )]
                    'outer: loop {
                        let byte = if pos.can_read_batch_from(&self.source) {
                            // Search a batch of `SEARCH_BATCH_SIZE` bytes.
                            //
                            // `'inner: loop {}` is not a real loop - it always exits on first turn.
                            // Only using `loop {}` so that can use `break 'inner` to get out of it.
                            // This allows complex logic of `$should_continue` and `$match_handler` to be
                            // outside the `for` loop, keeping it as minimal as possible, to encourage
                            // compiler to unroll it.
                            //
                            // SAFETY:
                            // `$pos.can_read_batch_from(&$lexer.source)` check above ensures there are
                            // at least `SEARCH_BATCH_SIZE` bytes remaining in `lexer.source`.
                            // So `$pos.add()` in this loop cannot go out of bounds.
                            let batch = unsafe { pos.slice(SEARCH_BATCH_SIZE) };
                            'inner: loop {
                                for (i, &byte) in batch.iter().enumerate() {
                                    if JSX_CHILD_END_TABLE.matches(byte) {
                                        // SAFETY: Cannot go out of bounds (see above).
                                        // Also see above about UTF-8 character boundaries invariant.
                                        pos = unsafe { pos.add(i) };
                                        break 'inner byte;
                                    }
                                }
                                // No match in batch - search next batch.
                                // SAFETY: Cannot go out of bounds (see above).
                                // Also see above about UTF-8 character boundaries invariant.
                                pos = unsafe { pos.add(SEARCH_BATCH_SIZE) };
                                continue 'outer;
                            }
                        } else {
                            // Not enough bytes remaining for a batch. Process byte-by-byte.
                            // Same as above, `'inner: loop {}` is not a real loop here - always exits on first turn.
                            'inner: loop {
                                // SAFETY: `$pos` is before or equal to end of source
                                let remaining = unsafe {
                                    let remaining_len = self.source.end().offset_from(pos);
                                    pos.slice(remaining_len)
                                };
                                for (i, &byte) in remaining.iter().enumerate() {
                                    if JSX_CHILD_END_TABLE.matches(byte) {
                                        // SAFETY: `i` is less than number of bytes remaining after `$pos`,
                                        // so `$pos + i` cannot be out of bounds
                                        pos = unsafe { pos.add(i) };
                                        break 'inner byte;
                                    }
                                }

                                // EOF.
                                // Advance `lexer.source`'s position to end of file.
                                self.source.advance_to_end();

                                // Avoid lint errors if `$eof_handler` contains `return` statement
                                #[allow(
                                    unused_variables,
                                    unreachable_code,
                                    clippy::diverging_sub_expression,
                                    clippy::allow_attributes
                                )]
                                {
                                    let eof_ret = {
                                        return Kind::Eof;
                                    };
                                    break 'outer eof_ret;
                                }
                            }
                        };

                        // Found match. Check if should continue.
                        if false {
                            // continue_if: (byte, pos) false
                            // Not a match after all - continue searching.
                            // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
                            // See above about UTF-8 character boundaries invariant.
                            pos = unsafe { pos.add(1) };
                            continue;
                        }

                        // Match confirmed.
                        // Advance `lexer.source`'s position up to `$pos`, consuming unmatched bytes.
                        // SAFETY: See above about UTF-8 character boundaries invariant.
                        self.source.set_position(pos);

                        break byte;
                    }
                };

                if matches!(next_byte, b'<' | b'{') {
                    Kind::JSXText
                } else {
                    cold_branch(|| {
                        let start = self.offset();
                        self.error(diagnostics::unexpected_jsx_end(
                            Span::empty(start),
                            next_byte as char,
                            if next_byte == b'}' { "rbrace" } else { "gt" },
                        ));
                        Kind::Eof
                    })
                }
            }
            None => Kind::Eof,
        }
    }

    /// Expand the current `Ident` token for `JSXIdentifier`
    ///
    /// The current character is at `Ident`, continue reading for `JSXIdentifier` if it has a `-`
    ///
    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    pub(crate) fn continue_lex_jsx_identifier(&mut self) -> Option<Token> {
        if self.peek_byte() != Some(b'-') {
            return None;
        }
        self.consume_char();

        // Consume bytes which are part of identifier tail
        let next_byte = {
            // Inlined byte_search! macro
            #[allow(clippy::unnecessary_safety_comment, clippy::allow_attributes)]
            NOT_ASCII_JSX_ID_CONTINUE_TABLE.use_table();

            let mut pos = self.source.position();
            // Silence warnings if macro called in unsafe code
            #[allow(unused_unsafe, clippy::unnecessary_safety_comment, clippy::allow_attributes)]
            'outer: loop {
                let byte = if pos.can_read_batch_from(&self.source) {
                    // Search a batch of `SEARCH_BATCH_SIZE` bytes.
                    //
                    // `'inner: loop {}` is not a real loop - it always exits on first turn.
                    // Only using `loop {}` so that can use `break 'inner` to get out of it.
                    // This allows complex logic of `$should_continue` and `$match_handler` to be
                    // outside the `for` loop, keeping it as minimal as possible, to encourage
                    // compiler to unroll it.
                    //
                    // SAFETY:
                    // `$pos.can_read_batch_from(&$lexer.source)` check above ensures there are
                    // at least `SEARCH_BATCH_SIZE` bytes remaining in `lexer.source`.
                    // So `$pos.add()` in this loop cannot go out of bounds.
                    let batch = unsafe { pos.slice(SEARCH_BATCH_SIZE) };
                    'inner: loop {
                        for (i, &byte) in batch.iter().enumerate() {
                            if NOT_ASCII_JSX_ID_CONTINUE_TABLE.matches(byte) {
                                // SAFETY: Cannot go out of bounds (see above).
                                // Also see above about UTF-8 character boundaries invariant.
                                pos = unsafe { pos.add(i) };
                                break 'inner byte;
                            }
                        }
                        // No match in batch - search next batch.
                        // SAFETY: Cannot go out of bounds (see above).
                        // Also see above about UTF-8 character boundaries invariant.
                        pos = unsafe { pos.add(SEARCH_BATCH_SIZE) };
                        continue 'outer;
                    }
                } else {
                    // Not enough bytes remaining for a batch. Process byte-by-byte.
                    // Same as above, `'inner: loop {}` is not a real loop here - always exits on first turn.
                    'inner: loop {
                        // SAFETY: `$pos` is before or equal to end of source
                        let remaining = unsafe {
                            let remaining_len = self.source.end().offset_from(pos);
                            pos.slice(remaining_len)
                        };
                        for (i, &byte) in remaining.iter().enumerate() {
                            if NOT_ASCII_JSX_ID_CONTINUE_TABLE.matches(byte) {
                                // SAFETY: `i` is less than number of bytes remaining after `$pos`,
                                // so `$pos + i` cannot be out of bounds
                                pos = unsafe { pos.add(i) };
                                break 'inner byte;
                            }
                        }

                        // EOF.
                        // Advance `lexer.source`'s position to end of file.
                        self.source.advance_to_end();

                        // Avoid lint errors if `$eof_handler` contains `return` statement
                        #[allow(
                            unused_variables,
                            unreachable_code,
                            clippy::diverging_sub_expression,
                            clippy::allow_attributes
                        )]
                        {
                            let eof_ret = {
                                return Some(self.finish_next(Kind::Ident));
                            };
                            break 'outer eof_ret;
                        }
                    }
                };

                // Found match. Check if should continue.
                if false {
                    // continue_if: (byte, pos) false
                    // Not a match after all - continue searching.
                    // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
                    // See above about UTF-8 character boundaries invariant.
                    pos = unsafe { pos.add(1) };
                    continue;
                }

                // Match confirmed.
                // Advance `lexer.source`'s position up to `$pos`, consuming unmatched bytes.
                // SAFETY: See above about UTF-8 character boundaries invariant.
                self.source.set_position(pos);

                break byte;
            }
        };

        // Found a matching byte.
        // Either end of identifier found, or a Unicode char.
        if !next_byte.is_ascii() {
            // Unicode chars are rare in identifiers, so cold branch to keep common path for ASCII
            // as fast as possible
            cold_branch(|| {
                while let Some(c) = self.peek_char() {
                    if c == '-' || is_identifier_part(c) {
                        self.consume_char();
                    } else {
                        break;
                    }
                }
            });
        }

        Some(self.finish_next(Kind::Ident))
    }
}
