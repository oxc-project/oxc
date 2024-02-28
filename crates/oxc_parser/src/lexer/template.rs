use super::{
    cold_branch,
    search::{byte_search, safe_byte_match_table, SafeByteMatchTable},
    Kind, Lexer, SourcePosition, Token,
};
use crate::diagnostics;

use std::cmp::max;

use oxc_allocator::String;

const MIN_ESCAPED_TEMPLATE_LIT_LEN: usize = 16;

static TEMPLATE_LITERAL_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'$' | b'`' | b'\r' | b'\\'));

impl<'a> Lexer<'a> {
    /// 12.8.6 Template Literal Lexical Components

    /// Read template literal component.
    pub(super) fn read_template_literal(&mut self, substitute: Kind, tail: Kind) -> Kind {
        byte_search! {
            lexer: self,
            table: TEMPLATE_LITERAL_TABLE,
            continue_if: |next_byte, pos| {
                match next_byte {
                    b'$' => {
                        // SAFETY: Next byte is `$` which is ASCII, so after it is a UTF-8 char boundary
                        let after_dollar = unsafe { pos.add(1) };
                        if after_dollar.addr() < self.source.end_addr() {
                            // If `${`, exit.
                            // SAFETY: Have checked there's at least 1 further byte to read.
                            if unsafe { after_dollar.read() } == b'{' {
                                // Consume `${` and exit.
                                // SAFETY: Consuming `${` leaves `pos` on a UTF-8 char boundary.
                                self.source.set_position(unsafe { after_dollar.add(1) });
                                return substitute;
                            }
                            // Not `${`. Continue searching.
                            true
                        } else {
                            // This is last byte in file. Continue to `handle_eof`.
                            // This is illegal in valid JS, so mark this branch cold.
                            cold_branch(|| true)
                        }
                    },
                    b'`' => {
                        // Consume b'`' and exit.
                        // SAFETY: Char at `pos` is '`', so `pos + 1` is a UTF-8 char boundary.
                        let after_backtick = unsafe { pos.add(1) };
                        self.source.set_position(after_backtick);
                        return tail;
                    },
                    b'\r' => {
                        // SAFETY: Byte at `pos` is `\r`.
                        // `pos` has only been advanced relative to `self.source.position()`.
                        return unsafe { self.template_literal_carriage_return(pos, substitute, tail) };
                    }
                    _ => {
                        // `TEMPLATE_LITERAL_TABLE` only matches `$`, '`', `\r` and `\`
                        debug_assert!(next_byte == b'\\');
                        // SAFETY: Byte at `pos` is `\`.
                        // `pos` has only been advanced relative to `self.source.position()`.
                        return unsafe { self.template_literal_backslash(pos, substitute, tail) };
                    }
                }
            },
            handle_match: |_next_byte, _start| {
                // TODO: This should be `unreachable!()`
                Kind::Undetermined
            },
            handle_eof: |_start| {
                self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                Kind::Undetermined
            },
        };
    }

    /// Consume rest of template literal after a `\r` is found.
    ///
    /// # SAFETY
    /// * Byte at `pos` must be `\r`.
    /// * `pos` must not be before `self.source.position()`.
    unsafe fn template_literal_carriage_return(
        &mut self,
        mut pos: SourcePosition<'a>,
        substitute: Kind,
        tail: Kind,
    ) -> Kind {
        // Create arena string to hold modified template literal, containing up to before `\r`.
        // SAFETY: Caller guarantees `pos` is not before `self.source.position()`.
        let str = self.template_literal_create_string(pos);

        // Skip `\r`.
        // SAFETY: Caller guarantees byte at `pos` is `\r`, so `pos + 1` is a UTF-8 char boundary.
        pos = pos.add(1);

        // If at EOF, exit. This illegal in valid JS, so cold branch.
        if pos.addr() == self.source.end_addr() {
            return cold_branch(|| {
                self.source.advance_to_end();
                self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                Kind::Undetermined
            });
        }

        // Start next chunk after `\r`
        let chunk_start = pos;

        // If next char is `\n`, start next search after it.
        // `\n` is first char of next chunk, so it'll get added to `str` when chunk is pushed.
        // SAFETY: Have checked not at EOF.
        if pos.read() == b'\n' {
            // SAFETY: `\n` is ASCII, so advancing past it leaves `pos` on a UTF-8 char boundary
            pos = pos.add(1);
        }

        self.template_literal_different(str, pos, chunk_start, true, substitute, tail)
    }

    /// Consume rest of template literal after a `\` escape is found.
    ///
    /// # SAFETY
    /// * Byte at `pos` must be `\`.
    /// * `pos` must not be before `self.source.position()`.
    unsafe fn template_literal_backslash(
        &mut self,
        pos: SourcePosition<'a>,
        substitute: Kind,
        tail: Kind,
    ) -> Kind {
        // Create arena string to hold modified template literal, containing up to before `\`.
        // SAFETY: Caller guarantees `pos` is not before `self.source.position()`.
        let mut str = self.template_literal_create_string(pos);

        // Decode escape sequence into `str`.
        // `read_string_escape_sequence` expects `self.source` to be positioned after `\`.
        // SAFETY: Caller guarantees next byte is `\`, which is ASCII, so `pos + 1` is UTF-8 char boundary.
        let after_backslash = pos.add(1);
        self.source.set_position(after_backslash);

        let mut is_valid_escape_sequence = true;
        self.read_string_escape_sequence(&mut str, true, &mut is_valid_escape_sequence);

        // Continue search after escape
        let after_escape = self.source.position();
        // SAFETY: `pos` and `chunk_start` are the same
        self.template_literal_different(
            str,
            after_escape,
            after_escape,
            is_valid_escape_sequence,
            substitute,
            tail,
        )
    }

    /// Create arena string for modified template literal, containing the template literal up to `pos`.
    /// # SAFETY
    /// `pos` must not be before `self.source.position()`
    unsafe fn template_literal_create_string(&self, pos: SourcePosition) -> String<'a> {
        // Create arena string to hold modified template literal.
        // We don't know how long template literal will end up being. Take a guess that total length
        // will be double what we've seen so far, or `MIN_ESCAPED_TEMPLATE_LIT_LEN` minimum.
        // SAFETY: Caller guarantees `pos` is not before `self.source.position()`.
        let so_far = self.source.str_from_current_to_pos_unchecked(pos);
        let capacity = max(so_far.len() * 2, MIN_ESCAPED_TEMPLATE_LIT_LEN);
        let mut str = String::with_capacity_in(capacity, self.allocator);
        str.push_str(so_far);
        str
    }

    /// Process template literal after `\n` or `\` found.
    /// # SAFETY
    /// `chunk_start` must not be after `pos`.
    unsafe fn template_literal_different(
        &mut self,
        mut str: String<'a>,
        pos: SourcePosition<'a>,
        mut chunk_start: SourcePosition<'a>,
        mut is_valid_escape_sequence: bool,
        substitute: Kind,
        tail: Kind,
    ) -> Kind {
        byte_search! {
            lexer: self,
            table: TEMPLATE_LITERAL_TABLE,
            start: pos,
            continue_if: |next_byte, pos| {
                match next_byte {
                    b'$' => {
                        // SAFETY: Next byte is `$` which is ASCII, so after it is a UTF-8 char boundary
                        let after_dollar = pos.add(1);
                        if after_dollar.addr() < self.source.end_addr() {
                            // If `${`, exit.
                            // SAFETY: Have checked there's at least 1 further byte to read.
                            if unsafe { after_dollar.read() } == b'{' {
                                // Add last chunk to `str` and record string.
                                // SAFETY: TODO
                                let chunk = self.source.str_between_positions_unchecked(chunk_start, pos);
                                str.push_str(chunk);
                                self.save_template_string(
                                    is_valid_escape_sequence,
                                    str.into_bump_str(),
                                );

                                // Consume `${` and exit.
                                // SAFETY: Consuming `${` leaves `pos` on a UTF-8 char boundary.
                                self.source.set_position(unsafe { after_dollar.add(1) });
                                return substitute;
                            }
                            // Not `${`. Continue searching.
                            true
                        } else {
                            // This is last byte in file. Continue to `handle_eof`.
                            // This is illegal in valid JS, so mark this branch cold.
                            cold_branch(|| true)
                        }
                    },
                    b'`' => {
                        // Add last chunk to `str` and record string.
                        // SAFETY: TODO
                        let chunk = self.source.str_between_positions_unchecked(chunk_start, pos);
                        str.push_str(chunk);
                        self.save_template_string(
                            is_valid_escape_sequence,
                            str.into_bump_str(),
                        );

                        // Consume b'`' and exit.
                        // SAFETY: Byte at `pos` is '`', so `pos + 1` is a UTF-8 char boundary.
                        let after_backtick = pos.add(1);
                        self.source.set_position(after_backtick);
                        return tail;
                    },
                    b'\r' => {
                        // Add before `\r` to `str`.
                        // SAFETY: TODO
                        let chunk = self.source.str_between_positions_unchecked(chunk_start, pos);
                        str.push_str(chunk);

                        // Set next chunk to start after `\r`.
                        // SAFETY: TODO
                        chunk_start = pos.add(1);

                        if chunk_start.addr() < self.source.end_addr() {
                            // If next char is `\n`, start next search after it.
                            // NB: `byte_search!` macro already advances `pos` by 1, so only advance
                            // by 1 here, so that in total we skip 2 bytes for `\r\n`.
                            // No need to push `\n` to `str`, as it's 1st char of next chunk,
                            // and will be added to `str` when next chunk is pushed.
                            if chunk_start.read() == b'\n' {
                                // SAFETY: TODO
                                pos = chunk_start;
                            }
                            true
                        } else {
                            // This is last byte in file. Continue to `handle_eof`.
                            // This is illegal in valid JS, so mark this branch cold.
                            cold_branch(|| true)
                        }
                    }
                    _ => {
                        // `TEMPLATE_LITERAL_TABLE` only matches `$`, '`', `\r` and `\`
                        debug_assert!(next_byte == b'\\');

                        // Add chunk before escape to `str`.
                        // SAFETY: TODO
                        let chunk = self.source.str_between_positions_unchecked(chunk_start, pos);
                        str.push_str(chunk);

                        // Decode escape sequence into `str`.
                        // `read_string_escape_sequence` expects `self.source` to be positioned after `\`.
                        // SAFETY: Next byte is `\`, which is ASCII, so `pos + 1` is UTF-8 char boundary.
                        let after_backslash = pos.add(1);
                        self.source.set_position(after_backslash);
                        self.read_string_escape_sequence(&mut str, true, &mut is_valid_escape_sequence);

                        // Start next chunk after escape sequence
                        chunk_start = self.source.position();

                        // Continue search after escape sequence.
                        // NB: `byte_search!` macro increments `pos`, so need to subtract 1 here
                        // to counteract that.
                        // SAFETY: Added 1 to `pos` above, and `read_string_escape_sequence` only
                        // advances `self.source`, so subtracting 1 again is within bounds.
                        // TODO: This isn't good. It relies on behavior of `read_string_escape_sequence`,
                        // which makes no promise not to rewind `Source`.
                        pos = chunk_start.sub(1);

                        true
                    }
                }
            },
            handle_match: |_next_byte, _start| {
                // TODO: This should be `unreachable!()`
                Kind::Undetermined
            },
            handle_eof: |_start| {
                self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                Kind::Undetermined
            },
        };
    }

    /// Re-tokenize the current `}` token for `TemplateSubstitutionTail`
    /// See Section 12, the parser needs to re-tokenize on `TemplateSubstitutionTail`,
    pub(crate) fn next_template_substitution_tail(&mut self) -> Token {
        self.token.start = self.offset() - 1;
        let kind = self.read_template_literal(Kind::TemplateMiddle, Kind::TemplateTail);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Save escaped template string
    fn save_template_string(&mut self, is_valid_escape_sequence: bool, s: &'a str) {
        self.escaped_templates.insert(self.token.start, is_valid_escape_sequence.then_some(s));
        self.token.escaped = true;
    }

    pub(crate) fn get_template_string(&self, token: Token) -> Option<&'a str> {
        if token.escaped {
            return self.escaped_templates[&token.start];
        }
        let raw = &self.source.whole()[token.start as usize..token.end as usize];
        Some(match token.kind {
            Kind::NoSubstitutionTemplate | Kind::TemplateTail => {
                &raw[1..raw.len() - 1] // omit surrounding quotes or leading "}" and trailing "`"
            }
            Kind::TemplateHead | Kind::TemplateMiddle => {
                &raw[1..raw.len() - 2] // omit leading "`" or "}" and trailing "${"
            }
            _ => raw,
        })
    }
}
