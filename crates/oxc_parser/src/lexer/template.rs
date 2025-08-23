use std::{cmp::max, str};

use oxc_allocator::StringBuilder;

use crate::diagnostics;

use super::{
    Kind, Lexer, SourcePosition, Token, cold_branch,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
};

const MIN_ESCAPED_TEMPLATE_LIT_LEN: usize = 16;

/// Convert `char` to UTF-8 bytes array.
const fn to_bytes<const N: usize>(ch: char) -> [u8; N] {
    assert!(ch.len_utf8() == N);
    let mut bytes = [0u8; N];
    ch.encode_utf8(&mut bytes);
    bytes
}

/// Lossy replacement character (U+FFFD) as UTF-8 bytes.
const LOSSY_REPLACEMENT_CHAR_BYTES: [u8; 3] = to_bytes('\u{FFFD}');
const LOSSY_REPLACEMENT_CHAR_FIRST_BYTE: u8 = LOSSY_REPLACEMENT_CHAR_BYTES[0];
const _: () = assert!(LOSSY_REPLACEMENT_CHAR_FIRST_BYTE == 0xEF);

static TEMPLATE_LITERAL_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'$' | b'`' | b'\r' | b'\\'));

// Same as above, but with 1st byte of lossy replacement character added
static TEMPLATE_LITERAL_ESCAPED_MATCH_TABLE: SafeByteMatchTable = safe_byte_match_table!(
    |b| matches!(b, b'$' | b'`' | b'\r' | b'\\' | LOSSY_REPLACEMENT_CHAR_FIRST_BYTE)
);

/// 12.8.6 Template Literal Lexical Components
impl<'a> Lexer<'a> {
    /// Read template literal component.
    ///
    /// This function handles the common case where template contains no escapes or `\r` characters
    /// and so does not require saving to `lexer.escaped_templates`.
    /// If an escape or `\r` is found, control is passed to `template_literal_escaped` which builds
    /// the unescaped string. This division keeps the path for common case as fast as possible.
    pub(super) fn read_template_literal(&mut self, substitute: Kind, tail: Kind) -> Kind {
        let mut ret = substitute;

        byte_search! {
            lexer: self,
            table: TEMPLATE_LITERAL_TABLE,
            continue_if: (next_byte, pos) {
                match next_byte {
                    b'$' => {
                        // SAFETY: Next byte is `$` which is ASCII, so after it is a UTF-8 char boundary
                        let after_dollar = unsafe { pos.add(1) };
                        if after_dollar.is_not_end_of(&self.source) {
                            // If `${`, exit.
                            // SAFETY: Have checked there's at least 1 further byte to read.
                            if unsafe { after_dollar.read() } == b'{' {
                                // Skip `${` and stop searching.
                                // SAFETY: Consuming `${` leaves `pos` on a UTF-8 char boundary.
                                pos = unsafe { pos.add(2) };
                                false
                            } else {
                                // Not `${`. Continue searching.
                                true
                            }
                        } else {
                            // This is last byte in file. Continue to `handle_eof`.
                            // This is illegal in valid JS, so mark this branch cold.
                            cold_branch(|| true)
                        }
                    },
                    b'`' => {
                        // Skip '`' and stop searching.
                        // SAFETY: Char at `pos` is '`', so `pos + 1` is a UTF-8 char boundary.
                        pos = unsafe { pos.add(1) };
                        ret = tail;
                        false
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
            handle_eof: {
                self.error(diagnostics::unterminated_string(self.unterminated_range()));
                return Kind::Undetermined;
            },
        };

        ret
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
        let mut str = unsafe { self.template_literal_create_string(pos) };

        // Skip `\r`.
        // SAFETY: Caller guarantees byte at `pos` is `\r`, so `pos + 1` is a UTF-8 char boundary.
        pos = unsafe { pos.add(1) };

        // If at EOF, exit. This illegal in valid JS, so cold branch.
        if pos.is_end_of(&self.source) {
            return cold_branch(|| {
                self.source.advance_to_end();
                self.error(diagnostics::unterminated_string(self.unterminated_range()));
                Kind::Undetermined
            });
        }

        // Start next chunk after `\r`
        let chunk_start = pos;

        // Either `\r` alone or `\r\n` needs to be converted to `\n`.
        // SAFETY: Have checked not at EOF.
        if unsafe { pos.read() } == b'\n' {
            // We have `\r\n`.
            // Start next search after the `\n`.
            // `chunk_start` is before the `\n`, so no need to push an `\n` to `str` here.
            // The `\n` is first char of next chunk, so it'll get pushed to `str` later on
            // when that next chunk is pushed.
            // SAFETY: `\n` is ASCII, so advancing past it leaves `pos` on a UTF-8 char boundary.
            pos = unsafe { pos.add(1) };
        } else {
            // We have a lone `\r`.
            // Convert it to `\n` by pushing an `\n` to `str`.
            // `chunk_start` is *after* the `\r`, so the `\r` is not included in next chunk,
            // so it will not also get included in `str` when that next chunk is pushed.
            str.push('\n');
        }

        // SAFETY: `chunk_start` is not after `pos`
        unsafe { self.template_literal_escaped(str, pos, chunk_start, true, substitute, tail) }
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
        let mut str = unsafe { self.template_literal_create_string(pos) };

        // Decode escape sequence into `str`.
        // `read_string_escape_sequence` expects `self.source` to be positioned after `\`.
        // SAFETY: Caller guarantees next byte is `\`, which is ASCII, so `pos + 1` is UTF-8 char boundary.
        let after_backslash = unsafe { pos.add(1) };
        self.source.set_position(after_backslash);

        let mut is_valid_escape_sequence = true;
        self.read_string_escape_sequence(&mut str, true, &mut is_valid_escape_sequence);

        // Continue search after escape
        let after_escape = self.source.position();
        // SAFETY: `pos` and `chunk_start` are the same
        unsafe {
            self.template_literal_escaped(
                str,
                after_escape,
                after_escape,
                is_valid_escape_sequence,
                substitute,
                tail,
            )
        }
    }

    /// Create arena string for modified template literal, containing the template literal up to `pos`.
    ///
    /// # SAFETY
    /// `pos` must not be before `self.source.position()`
    unsafe fn template_literal_create_string(&self, pos: SourcePosition<'a>) -> StringBuilder<'a> {
        // Create arena string to hold modified template literal.
        // We don't know how long template literal will end up being. Take a guess that total length
        // will be double what we've seen so far, or `MIN_ESCAPED_TEMPLATE_LIT_LEN` minimum.
        // SAFETY: Caller guarantees `pos` is not before `self.source.position()`.
        let so_far = unsafe { self.source.str_from_current_to_pos_unchecked(pos) };
        let capacity = max(so_far.len() * 2, MIN_ESCAPED_TEMPLATE_LIT_LEN);
        let mut str = StringBuilder::with_capacity_in(capacity, self.allocator);
        str.push_str(so_far);
        str
    }

    /// Process template literal after `\r` or `\` found.
    ///
    /// # SAFETY
    /// `chunk_start` must not be after `pos`.
    unsafe fn template_literal_escaped(
        &mut self,
        mut str: StringBuilder<'a>,
        pos: SourcePosition<'a>,
        mut chunk_start: SourcePosition<'a>,
        mut is_valid_escape_sequence: bool,
        substitute: Kind,
        tail: Kind,
    ) -> Kind {
        let mut ret = substitute;

        byte_search! {
            lexer: self,
            table: TEMPLATE_LITERAL_ESCAPED_MATCH_TABLE,
            start: pos,
            continue_if: (next_byte, pos) {
                if next_byte == b'$' {
                    // SAFETY: Next byte is `$` which is ASCII, so after it is a UTF-8 char boundary
                    let after_dollar = unsafe {pos.add(1)};
                    if after_dollar.is_not_end_of(&self.source) {
                        // If `${`, exit.
                        // SAFETY: Have checked there's at least 1 further byte to read.
                        if unsafe {after_dollar.read()} == b'{' {
                            // Add last chunk to `str`.
                            // SAFETY: Caller guarantees `chunk_start` is not after `pos` at start of
                            // this function. `pos` only increases during searching.
                            // Where `chunk_start` is updated, it's always before or equal to `pos`.
                            // So `chunk_start` cannot be after `pos`.
                            let chunk = unsafe {self.source.str_between_positions_unchecked(chunk_start, pos)};
                            str.push_str(chunk);

                            // Skip `${` and stop searching.
                            // SAFETY: Consuming `${` leaves `pos` on a UTF-8 char boundary.
                            pos = unsafe {pos.add(2)};
                            false
                        } else {
                            // Not `${`. Continue searching.
                            true
                        }
                    } else {
                        // This is last byte in file. Continue to `handle_eof`.
                        // This is illegal in valid JS, so mark this branch cold.
                        cold_branch(|| true)
                    }
                } else {
                    // Next byte is '`', `\r`, `\`, or first byte of lossy replacement character.
                    // Add chunk up to before this char to `str`.
                    // SAFETY: Caller guarantees `chunk_start` is not after `pos` at start of
                    // this function. `pos` only increases during searching.
                    // Where `chunk_start` is updated, it's always before or equal to `pos`.
                    // So `chunk_start` cannot be after `pos`.
                    let chunk = unsafe {self.source.str_between_positions_unchecked(chunk_start, pos)};
                    str.push_str(chunk);

                    match next_byte {
                        b'`' => {
                            // Skip '`' and stop searching.
                            // SAFETY: Byte at `pos` is '`' (ASCII), so `pos + 1` is a UTF-8 char boundary.
                            pos = unsafe {pos.add(1)};
                            ret = tail;
                            false
                        }
                        b'\r' => {
                            // Set next chunk to start after `\r`.
                            // SAFETY: Next byte is `\r` which is ASCII, so after it is a UTF-8 char boundary.
                            // This temporarily puts `chunk_start` 1 byte after `pos`, but `byte_search!` macro
                            // increments `pos` when return `true` from `continue_if`, so `pos` will be
                            // brought up to `chunk_start` again.
                            chunk_start = unsafe {pos.add(1)};

                            if chunk_start.is_not_end_of(&self.source) {
                                // Either `\r` alone or `\r\n` needs to be converted to `\n`.
                                // SAFETY: Have checked not at EOF.
                                if unsafe {chunk_start.read()} == b'\n' {
                                    // We have `\r\n`.
                                    // Start next search after the `\n`.
                                    // `chunk_start` is before the `\n`, so no need to push an `\n`
                                    // to `str` here. The `\n` is first char of next chunk, so it'll get
                                    // pushed to `str` later on when that next chunk is pushed.
                                    // Note: `byte_search!` macro already advances `pos` by 1, so only
                                    // advance by 1 here, so that in total we skip 2 bytes for `\r\n`.
                                    pos = chunk_start;
                                } else {
                                    // We have a lone `\r`.
                                    // Convert it to `\n` by pushing an `\n` to `str`.
                                    // `chunk_start` is *after* the `\r`, so the `\r` is not included in
                                    // next chunk, so it will not also get included in `str` when that
                                    // next chunk is pushed.
                                    // Note: `byte_search!` macro already advances `pos` by 1,
                                    // which steps past the `\r`, so don't advance `pos` here.
                                    str.push('\n');
                                }
                            } else {
                                // This is last byte in file. Continue to `handle_eof`.
                                // This is illegal in valid JS, so mark this branch cold.
                                cold_branch(|| {});
                            }

                            // Continue searching
                            true
                        }
                        b'\\' => {
                            // Decode escape sequence into `str`.
                            // `read_string_escape_sequence` expects `self.source` to be positioned after `\`.
                            // SAFETY: Next byte is `\`, which is ASCII, so `pos + 1` is UTF-8 char boundary.
                            let after_backslash = unsafe {pos.add(1)};
                            self.source.set_position(after_backslash);
                            self.read_string_escape_sequence(&mut str, true, &mut is_valid_escape_sequence);

                            // Start next chunk after escape sequence
                            chunk_start = self.source.position();
                            assert!(chunk_start >= after_backslash);

                            // Continue search after escape sequence.
                            // NB: `byte_search!` macro increments `pos` when return `true`,
                            // so need to subtract 1 here to counteract that.
                            // SAFETY: Added 1 to `pos` above, and checked `chunk_start` hasn't moved
                            // backwards from that, so subtracting 1 again is within bounds.
                            pos = unsafe {chunk_start.sub(1)};

                            // Continue searching
                            true
                        }
                        _ => {
                            // `TEMPLATE_LITERAL_ESCAPED_MATCH_TABLE` only matches `$`, '`', `\r`, `\`,
                            // or first byte of lossy replacement character
                            debug_assert!(next_byte == LOSSY_REPLACEMENT_CHAR_FIRST_BYTE);

                            // SAFETY: 0xEF is always first byte of a 3-byte UTF-8 character,
                            // so there must be 2 more bytes to read
                            let next2 = unsafe { pos.add(1).read2() };
                            if next2 == [LOSSY_REPLACEMENT_CHAR_BYTES[1], LOSSY_REPLACEMENT_CHAR_BYTES[2]]
                                && self.token.lone_surrogates()
                            {
                                str.push_str("\u{FFFD}fffd");
                            } else {
                                let bytes = [LOSSY_REPLACEMENT_CHAR_FIRST_BYTE, next2[0], next2[1]];
                                // SAFETY: 0xEF is always first byte of a 3-byte UTF-8 character,
                                // so these 3 bytes must comprise a valid UTF-8 string
                                let s = unsafe { str::from_utf8_unchecked(&bytes) };
                                str.push_str(s);
                            }

                            // Advance past this character.
                            // SAFETY: Character is 3 bytes, so `pos + 2` is in bounds.
                            // Note: `byte_search!` macro already advances `pos` by 1, so only
                            // advance by 2 here, so that in total we skip 3 bytes.
                            pos = unsafe { pos.add(2) };

                            // Set next chunk to start after this character.
                            // SAFETY: It's a 3 byte character, and we added 2 to `pos` above,
                            // so `pos + 1` must be a UTF-8 char boundary.
                            // This temporarily puts `chunk_start` 1 byte after `pos`, but `byte_search!` macro
                            // increments `pos` when return `true` from `continue_if`, so `pos` will be
                            // brought up to `chunk_start` again.
                            chunk_start = unsafe { pos.add(1) };

                            // Continue searching
                            true
                        }
                    }
                }
            },
            handle_eof: {
                self.error(diagnostics::unterminated_string(self.unterminated_range()));
                return Kind::Undetermined;
            },
        };

        self.save_template_string(is_valid_escape_sequence, str.into_str());

        ret
    }

    /// Re-tokenize the current `}` token for `TemplateSubstitutionTail`
    /// See Section 12, the parser needs to re-tokenize on `TemplateSubstitutionTail`,
    pub(crate) fn next_template_substitution_tail(&mut self) -> Token {
        self.token.set_start(self.offset() - 1);
        let kind = self.read_template_literal(Kind::TemplateMiddle, Kind::TemplateTail);
        self.finish_next(kind)
    }

    /// Save escaped template string
    fn save_template_string(&mut self, is_valid_escape_sequence: bool, s: &'a str) {
        self.escaped_templates.insert(self.token.start(), is_valid_escape_sequence.then_some(s));
        self.token.set_escaped(true);
    }

    pub(crate) fn get_template_string(&self, span_start: u32) -> Option<&'a str> {
        self.escaped_templates[&span_start]
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_span::SourceType;

    use super::super::{Kind, Lexer, UniquePromise};

    #[test]
    fn template_literal_linebreaks() {
        // Note: These cases don't include all `\n`s because that requires no unescaping
        let escapes = [
            // 1 return
            ("\r", "\n"),
            ("\r\n", "\n"),
            // 2 returns
            ("\r\r", "\n\n"),
            ("\r\r\n", "\n\n"),
            ("\r\n\r", "\n\n"),
            ("\r\n\n", "\n\n"),
            ("\n\r", "\n\n"),
            ("\n\r\n", "\n\n"),
            ("\r\n\r\n", "\n\n"),
            // 3 returns
            ("\r\r\r", "\n\n\n"),
            ("\n\r\r", "\n\n\n"),
            ("\n\n\r", "\n\n\n"),
            ("\r\n\r\r", "\n\n\n"),
            ("\r\r\n\r", "\n\n\n"),
            ("\r\r\r\n", "\n\n\n"),
            ("\r\n\r\n\r", "\n\n\n"),
            ("\r\r\n\r\n", "\n\n\n"),
            ("\r\n\r\n\r\n", "\n\n\n"),
        ];

        #[expect(clippy::items_after_statements, clippy::needless_pass_by_value)]
        fn run_test(source_text: String, expected_escaped: String, is_only_part: bool) {
            let allocator = Allocator::default();
            let unique = UniquePromise::new_for_tests_and_benchmarks();
            let mut lexer = Lexer::new(&allocator, &source_text, SourceType::default(), unique);
            let token = lexer.next_token();
            assert_eq!(
                token.kind(),
                if is_only_part { Kind::NoSubstitutionTemplate } else { Kind::TemplateHead }
            );
            let escaped = lexer.escaped_templates[&token.start()];
            assert_eq!(escaped, Some(expected_escaped.as_str()));
        }

        for (source_fragment, escaped_fragment) in escapes {
            run_test(format!("`{source_fragment}`"), escaped_fragment.to_string(), true);
            run_test(format!("`{source_fragment}${{x}}`"), escaped_fragment.to_string(), false);
            run_test(format!("`{source_fragment}abc`"), format!("{escaped_fragment}abc"), true);
            run_test(
                format!("`{source_fragment}abc${{x}}`"),
                format!("{escaped_fragment}abc"),
                false,
            );
            run_test(format!("`abc{source_fragment}`"), format!("abc{escaped_fragment}"), true);
            run_test(
                format!("`abc{source_fragment}${{x}}`"),
                format!("abc{escaped_fragment}"),
                false,
            );
            run_test(
                format!("`abc{source_fragment}def{source_fragment}ghi`"),
                format!("abc{escaped_fragment}def{escaped_fragment}ghi"),
                true,
            );
            run_test(
                format!("`abc{source_fragment}def{source_fragment}ghi${{x}}`"),
                format!("abc{escaped_fragment}def{escaped_fragment}ghi"),
                false,
            );
            run_test(
                format!("`{source_fragment}abc{source_fragment}def{source_fragment}`"),
                format!("{escaped_fragment}abc{escaped_fragment}def{escaped_fragment}"),
                true,
            );
            run_test(
                format!("`{source_fragment}abc{source_fragment}def{source_fragment}${{x}}`"),
                format!("{escaped_fragment}abc{escaped_fragment}def{escaped_fragment}"),
                false,
            );
        }
    }
}
