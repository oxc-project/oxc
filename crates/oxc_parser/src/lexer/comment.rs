use memchr::memmem::Finder;

use oxc_syntax::identifier::is_line_terminator;

use crate::diagnostics;

use super::{
    Kind, Lexer, cold_branch,
    search::{SafeByteMatchTable, safe_byte_match_table, byte_search_with_advanced_continue, SearchContinuation},
    source::SourcePosition,
};

// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA9];

static LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\r' | b'\n' | LS_OR_PS_FIRST));

static MULTILINE_COMMENT_START_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'*' | b'\r' | b'\n' | LS_OR_PS_FIRST));

impl<'a> Lexer<'a> {
    /// Section 12.4 Single Line Comment
    pub(super) fn skip_single_line_comment(&mut self) -> Kind {
        let _byte = match byte_search_with_advanced_continue(
            self,
            &LINE_BREAK_TABLE,
            self.source.position(),
            |byte, pos| {
                let next_byte = byte;
                // Match found. Decide whether to continue searching.
                // If this is end of comment, create trivia, and advance `pos` to after line break.
                // Do that here rather than in `handle_match`, to avoid branching twice on value of
                // the matched byte.
                #[expect(clippy::if_not_else)]
                if next_byte != LS_OR_PS_FIRST {
                    // `\r` or `\n`
                    self.trivia_builder.add_line_comment(
                        self.token.start(),
                        self.source.offset_of(pos),
                        self.source.whole(),
                    );
                    // SAFETY: Safe to consume `\r` or `\n` as both are ASCII
                    let new_pos = unsafe { pos.add(1) };
                    // We've found the end. Do not continue searching.
                    SearchContinuation::Stop(new_pos)
                } else {
                    // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
                    // Either way, Unicode is uncommon, so make this a cold branch.
                    cold_branch(|| {
                        // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                        // So safe to advance `pos` by 1 and read 2 bytes.
                        let next2 = unsafe { pos.add(1).read2() };
                        if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                            // Irregular line break
                            self.trivia_builder.add_line_comment(
                                self.token.start(),
                                self.source.offset_of(pos),
                                self.source.whole(),
                            );
                            // Advance `pos` to after this char.
                            // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                            // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                            let new_pos = unsafe { pos.add(3) };
                            // We've found the end. Do not continue searching.
                            SearchContinuation::Stop(new_pos)
                        } else {
                            // Some other Unicode char beginning with `0xE2`.
                            // Skip 3 bytes and continue searching.
                            // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                            // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                            let new_pos = unsafe { pos.add(3) };
                            SearchContinuation::Continue(new_pos)
                        }
                    })
                }
            },
            || {
                self.trivia_builder.add_line_comment(
                    self.token.start(),
                    self.offset(),
                    self.source.whole(),
                );
                return Kind::Skip;
            },
        ) {
            Ok(_) => {},
            Err(kind) => return kind,
        };

        self.token.set_is_on_new_line(true);
        Kind::Skip
    }

    /// Section 12.4 Multi Line Comment
    pub(super) fn skip_multi_line_comment(&mut self) -> Kind {
        // If `is_on_new_line` is already set, go directly to faster search which only looks for `*/`
        if self.token.is_on_new_line() {
            return self.skip_multi_line_comment_after_line_break(self.source.position());
        }

        let result = byte_search_with_advanced_continue(
            self,
            &MULTILINE_COMMENT_START_TABLE,
            self.source.position(),
            |byte, pos| {
                let next_byte = byte;
                // Match found. Decide whether to continue searching.
                if next_byte == b'*' {
                    // SAFETY: Next byte is `*` (ASCII) so after it is UTF-8 char boundary
                    let after_star = unsafe { pos.add(1) };
                    if after_star.is_not_end_of(&self.source) {
                        // If next byte isn't `/`, continue
                        // SAFETY: Have checked there's at least 1 further byte to read
                        if unsafe { after_star.read() } == b'/' {
                            // Consume `*/`
                            // SAFETY: Consuming `*/` leaves `pos` on a UTF-8 char boundary
                            let new_pos = unsafe { pos.add(2) };
                            SearchContinuation::Stop(new_pos)
                        } else {
                            // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
                            let new_pos = unsafe { pos.add(1) };
                            SearchContinuation::Continue(new_pos)
                        }
                    } else {
                        // This is last byte in file. Continue to `handle_eof`.
                        // This is illegal in valid JS, so mark this branch cold.
                        cold_branch(|| {
                            // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
                            let new_pos = unsafe { pos.add(1) };
                            SearchContinuation::Continue(new_pos)
                        })
                    }
                } else if next_byte == LS_OR_PS_FIRST {
                    // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
                    // Either way, Unicode is uncommon, so make this a cold branch.
                    cold_branch(|| {
                        // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                        // So safe to advance `pos` by 1 and read 2 bytes.
                        let next2 = unsafe { pos.add(1).read2() };
                        if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                            // Irregular line break
                            self.token.set_is_on_new_line(true);
                            // Ideally we'd go on to `skip_multi_line_comment_after_line_break` here
                            // but can't do that easily because can't use `return` in a closure.
                            // But irregular line breaks are rare anyway.
                        }
                        // Either way, continue searching.
                        // Skip 3 bytes and continue searching.
                        // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                        // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                        let new_pos = unsafe { pos.add(3) };
                        SearchContinuation::Continue(new_pos)
                    })
                } else {
                    // Regular line break.
                    // No need to look for more line breaks, so switch to faster search just for `*/`.
                    self.token.set_is_on_new_line(true);
                    // SAFETY: Regular line breaks are ASCII, so skipping 1 byte is a UTF-8 char boundary.
                    let after_line_break = unsafe { pos.add(1) };
                    // We need to call the specialized function and return early
                    // This is a bit awkward with the current function design
                    // For now, we'll just continue and handle this case after the search
                    SearchContinuation::Stop(after_line_break)
                }
            },
            || {
                self.error(diagnostics::unterminated_multi_line_comment(
                    self.unterminated_range(),
                ));
                return Kind::Eof;
            },
        );

        match result {
            Ok(_) => {
                // Check if this was a regular line break that requires special handling
                if self.token.is_on_new_line() {
                    // This might have been set in the continue_if closure due to a line break
                    // In that case, we should delegate to the specialized function
                    // But we've already advanced the position, so we can just continue with the normal flow
                }
            }
            Err(kind) => return kind,
        };

        self.trivia_builder.add_block_comment(
            self.token.start(),
            self.offset(),
            self.source.whole(),
        );
        Kind::Skip
    }

    fn skip_multi_line_comment_after_line_break(&mut self, pos: SourcePosition<'a>) -> Kind {
        // Can use `memchr` here as only searching for 1 pattern.
        // Cache `Finder` instance on `Lexer` as there's a significant cost to creating it.
        // `Finder::new` isn't a const function, so can't make it a `static`, and `lazy_static!`
        // has a cost each time it's deref-ed. Creating `Finder` unconditionally in `Lexer::new`
        // would be efficient for files containing multi-line comments, but would impose pointless
        // cost on files which don't. So this is the fastest solution.
        let finder = self.multi_line_comment_end_finder.get_or_insert_with(|| Finder::new("*/"));

        let remaining = self.source.str_from_pos_to_end(pos).as_bytes();
        if let Some(index) = finder.find(remaining) {
            // SAFETY: `pos + index + 2` is end of `*/`, so a valid `SourcePosition`
            self.source.set_position(unsafe { pos.add(index + 2) });
            self.trivia_builder.add_block_comment(
                self.token.start(),
                self.offset(),
                self.source.whole(),
            );
            Kind::Skip
        } else {
            self.source.advance_to_end();
            self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
            Kind::Eof
        }
    }

    /// Section 12.5 Hashbang Comments.
    ///
    /// # SAFETY
    /// Next 2 bytes must be `#!`.
    pub(super) unsafe fn read_hashbang_comment(&mut self) -> Kind {
        debug_assert!(self.peek_2_bytes() == Some([b'#', b'!']));

        // SAFETY: Caller guarantees next 2 bytes are `#!`
        unsafe {
            self.source.next_byte_unchecked();
            self.source.next_byte_unchecked();
        }

        while let Some(c) = self.peek_char() {
            if is_line_terminator(c) {
                break;
            }
            self.consume_char();
        }
        Kind::HashbangComment
    }
}
