use memchr::memmem::Finder;

use oxc_ast::CommentKind;
use oxc_data_structures::branch_hints::cold_path;
use oxc_syntax::line_terminator::is_line_terminator;

use crate::{config::LexerConfig as Config, diagnostics};

use super::{
    Kind, Lexer,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
    source::SourcePosition,
    trivia_builder::contains_license_or_preserve_comment,
};

// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA9];

static LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\r' | b'\n' | LS_OR_PS_FIRST));

static MULTILINE_COMMENT_START_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'*' | b'\r' | b'\n' | LS_OR_PS_FIRST));

/// Number of times to search a multi-line comment for `/` or `@`, before searching for its end
/// with a search which other `/` and `@` in the comment do not interrupt.
///
/// Each search which stops short of the end of the comment costs as much as searching a few dozen
/// more bytes. Most comments contain no more than one `/` or `@` before their end.
const MAX_SEARCHES_FOR_COMMENT_END: u32 = 2;

/// Length below which it is cheaper to check each byte for a line break
/// than to start a search for one.
const MIN_LEN_TO_SEARCH_FOR_LINE_BREAK: usize = 16;

/// Returns `true` if `bytes` contains a line terminator.
#[inline]
fn contains_line_break(mut bytes: &[u8]) -> bool {
    if bytes.len() < MIN_LEN_TO_SEARCH_FOR_LINE_BREAK {
        for (index, &byte) in bytes.iter().enumerate() {
            if LINE_BREAK_TABLE.matches(byte)
                && (byte != LS_OR_PS_FIRST || starts_with_ls_or_ps_tail(&bytes[index + 1..]))
            {
                return true;
            }
        }
        return false;
    }

    loop {
        let Some(index) = memchr::memchr3(b'\r', b'\n', LS_OR_PS_FIRST, bytes) else {
            return false;
        };
        if bytes[index] != LS_OR_PS_FIRST {
            return true;
        }
        // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
        // Either way, Unicode is uncommon, so make this a cold branch.
        cold_path();
        // Skip only that byte. If it starts some other Unicode char,
        // the other bytes of that char will not match the search.
        bytes = &bytes[index + 1..];
        if starts_with_ls_or_ps_tail(bytes) {
            return true;
        }
    }
}

/// Returns `true` if `bytes` starts with the 2nd and 3rd bytes of LS or PS.
#[inline]
fn starts_with_ls_or_ps_tail(bytes: &[u8]) -> bool {
    bytes.starts_with(&LS_BYTES_2_AND_3) || bytes.starts_with(&PS_BYTES_2_AND_3)
}

impl<'a, C: Config> Lexer<'a, C> {
    /// Section 12.4 Single Line Comment
    pub(super) fn skip_single_line_comment(&mut self, kind: CommentKind) -> Kind {
        let start = self.source.position();
        let remaining = self.source.remaining().as_bytes();
        let Some(index) = memchr::memchr3(b'\r', b'\n', LS_OR_PS_FIRST, remaining) else {
            self.source.advance_to_end();
            self.trivia_builder.add_line_comment(
                self.token_start,
                self.offset(),
                kind,
                self.source.whole(),
                &mut self.pending,
            );
            return Kind::Skip;
        };
        // SAFETY: The match is an in-bounds ASCII byte or a UTF-8 leading byte.
        let end = unsafe { start.add(index) };
        if remaining[index] == LS_OR_PS_FIRST {
            // Keep the scalar scanner for Unicode, where repeated matches can be very close.
            self.source.set_position(end);
            return self.skip_single_line_comment_unicode(kind);
        }
        self.trivia_builder.add_line_comment(
            self.token_start,
            self.source.offset_of(end),
            kind,
            self.source.whole(),
            &mut self.pending,
        );
        // SAFETY: Consume the matching ASCII line break.
        self.source.set_position(unsafe { end.add(1) });
        self.pending.set_is_on_new_line();
        Kind::Skip
    }

    #[cold]
    fn skip_single_line_comment_unicode(&mut self, kind: CommentKind) -> Kind {
        byte_search! {
            lexer: self,
            table: LINE_BREAK_TABLE,
            continue_if: (next_byte, pos) {
                // Match found. Decide whether to continue searching.
                // If this is end of comment, create trivia, and advance `pos` to after line break.
                // Do that here rather than in `handle_match`, to avoid branching twice on value of
                // the matched byte.
                #[expect(clippy::if_not_else)]
                if next_byte != LS_OR_PS_FIRST {
                    // `\r` or `\n`
                    self.trivia_builder
                        .add_line_comment(self.token_start, self.source.offset_of(pos), kind, self.source.whole(), &mut self.pending);
                    // SAFETY: Safe to consume `\r` or `\n` as both are ASCII
                    pos = unsafe { pos.add(1) };
                    // We've found the end. Do not continue searching.
                    false
                } else {
                    // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
                    // Either way, Unicode is uncommon, so make this a cold branch.
                    cold_path();
                    // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                    // So safe to advance `pos` by 1 and read 2 bytes.
                    let next2 = unsafe { pos.add(1).read2() };
                    if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                        // Irregular line break
                        self.trivia_builder
                            .add_line_comment(self.token_start, self.source.offset_of(pos), kind, self.source.whole(), &mut self.pending);
                        // Advance `pos` to after this char.
                        // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                        // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                        pos = unsafe { pos.add(3) };
                        // We've found the end. Do not continue searching.
                        false
                    } else {
                        // Some other Unicode char beginning with `0xE2`.
                        // Skip 3 bytes (macro skips 1 already, so skip 2 here), and continue searching.
                        // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                        // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                        pos = unsafe { pos.add(2) };
                        true
                    }
                }
            },
            handle_eof: {
                self.trivia_builder.add_line_comment(self.token_start, self.offset(), kind, self.source.whole(), &mut self.pending);
                return Kind::Skip;
            },
        };

        self.pending.set_is_on_new_line();
        Kind::Skip
    }

    /// Section 12.4 Multi Line Comment
    ///
    /// Position of source must be after the opening `/*`.
    pub(super) fn skip_multi_line_comment(&mut self) -> Kind {
        // We need to identify if comment contains line breaks or not
        // (`CommentKind::SingleLineBlock` or `CommentKind::MultiLineBlock`),
        // even if `Token`'s `is_on_new_line` flag is already set.
        //
        // How to find the end of the comment fastest depends on what kind of comment it is,
        // and its first bytes say what kind it is likely to be.
        // First, find where to search for the end of the comment from,
        // and whether there is a line break in the comment before that position.
        let start = self.source.position();
        let (pos, has_line_break) = match self.source.remaining().as_bytes() {
            // A comment which spans lines is usually a JSDoc comment, which breaks straight after
            // its opening `/**`
            [b'*', b'\r' | b'\n', ..] => {
                // SAFETY: `*` and the line break are ASCII, so after them is in bounds,
                // and on a UTF-8 char boundary
                (unsafe { start.add(2) }, true)
            }
            // JSDoc comment with text on its first line. That is usually a sentence or more,
            // which is faster to search than to check byte by byte.
            [b'*', second, ..] if !matches!(second, b'*' | b'/') => (start, false),
            // Other comments are mostly short e.g. `/* @__PURE__ */` or `/******/`.
            // Starting a search costs more than checking each byte of one of those.
            _ => match self.skip_multi_line_comment_first_line() {
                Ok(after_line_break) => (after_line_break, true),
                Err(kind) => return kind,
            },
        };

        // The comment does not end before `pos`. It ends at the first `/` which follows a `*`.
        // `*` starts every line of a JSDoc comment, whereas `/` is uncommon in comments,
        // so search for `/`.
        // Search for `@` in the same pass, to find out whether the comment contains `@license`
        // or `@preserve`. `TriviaBuilder` needs to know that to classify the comment,
        // and would otherwise search the whole comment again.
        let rest = self.source.str_from_pos_to_end(pos).as_bytes();
        let mut has_legal_tag = false;
        let mut unsearched = rest;
        let mut searches_left = MAX_SEARCHES_FOR_COMMENT_END;
        let len = loop {
            if searches_left == 0 {
                // Cache `Finder` instance on `Lexer` as there's a significant cost to creating it.
                // `Finder::new` isn't a const function, so can't make it a `static`, and
                // `lazy_static!` has a cost each time it's deref-ed. Creating `Finder`
                // unconditionally in `Lexer::new` would be efficient for files containing
                // multi-line comments, but would impose pointless cost on files which don't.
                let finder =
                    self.multi_line_comment_end_finder.get_or_insert_with(|| Finder::new("*/"));
                let Some(index) = finder.find(unsearched) else {
                    return self.unterminated_multi_line_comment(rest, has_line_break);
                };
                // The rest of the comment has not been searched for the tags.
                // Skip that if the parser rewound and this comment was added before,
                // as `TriviaBuilder` does not classify it again.
                if !has_legal_tag && !self.trivia_builder.is_seen_comment(self.token_start) {
                    has_legal_tag = contains_license_or_preserve_comment(&unsearched[..index]);
                }
                break rest.len() - unsearched.len() + index;
            }
            searches_left -= 1;

            let Some(index) = memchr::memchr2(b'/', b'@', unsearched) else {
                return self.unterminated_multi_line_comment(rest, has_line_break);
            };
            let (before_match, from_match) = unsearched.split_at(index);
            unsearched = &from_match[1..];

            if from_match[0] == b'/' {
                // `before_match` is empty where `/` is at `pos`, or straight after another `/`
                // or `@`. `/` does not end the comment in any of those places.
                if before_match.last() == Some(&b'*') {
                    break rest.len() - from_match.len() - 1;
                }
            } else if !has_legal_tag {
                // Neither tag contains `*`, so if either is here, it is before the end of the comment
                has_legal_tag =
                    unsearched.starts_with(b"license") || unsearched.starts_with(b"preserve");
            }
        };

        // The search above started at `pos`. Where that is after the first line of the comment,
        // the first line has not been searched for the tags. It is usually only `*`.
        if !has_legal_tag {
            let first_line_start = self.token_start as usize + 2;
            let first_line_end = self.source.offset_of(pos) as usize;
            let first_line = &self.source.whole().as_bytes()[first_line_start..first_line_end];
            has_legal_tag = contains_license_or_preserve_comment(first_line);
        }

        let kind = if has_line_break || contains_line_break(&rest[..len]) {
            self.pending.set_is_on_new_line();
            CommentKind::MultiLineBlock
        } else {
            CommentKind::SingleLineBlock
        };

        // SAFETY: `pos + len` is the position of the closing `*/`, so `pos + len + 2` is in bounds,
        // and on a UTF-8 char boundary
        self.source.set_position(unsafe { pos.add(len + 2) });
        self.trivia_builder.add_block_comment(
            self.token_start,
            self.offset(),
            kind,
            has_legal_tag,
            self.source.whole(),
            &mut self.pending,
        );
        Kind::Skip
    }

    /// Find the end of the first line of a multi-line comment, by checking each byte.
    ///
    /// Returns `Ok` with the position after the first line break in the comment.
    /// Returns `Err` with the `Kind` to return, if the comment ends before a line break,
    /// or has neither a line break nor an end.
    #[inline]
    fn skip_multi_line_comment_first_line(&mut self) -> Result<SourcePosition<'a>, Kind> {
        byte_search! {
            lexer: self,
            table: MULTILINE_COMMENT_START_TABLE,
            continue_if: (next_byte, pos) {
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
                            pos = unsafe { pos.add(2) };
                            false
                        } else {
                            true
                        }
                    } else {
                        // This is last byte in file. Continue to `handle_eof`.
                        // This is illegal in valid JS, so mark this branch cold.
                        cold_path();
                        true
                    }
                } else if next_byte == LS_OR_PS_FIRST {
                    // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
                    // Either way, Unicode is uncommon, so make this a cold branch.
                    cold_path();
                    // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                    // So safe to advance `pos` by 1 and read 2 bytes.
                    let next2 = unsafe { pos.add(1).read2() };
                    if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                        // Irregular line break
                        // SAFETY: Consuming this 3-byte UTF-8 char leaves a UTF-8 char boundary.
                        let after_line_break = unsafe { pos.add(3) };
                        return Ok(after_line_break);
                    }
                    // Some other Unicode char beginning with `0xE2`.
                    // Skip 3 bytes (macro skips 1 already, so skip 2 here), and continue searching.
                    // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                    // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                    pos = unsafe { pos.add(2) };
                    true
                } else {
                    // Regular line break.
                    // No need to look for more line breaks, so switch to faster search for the end.
                    // SAFETY: Regular line breaks are ASCII, so skipping 1 byte is a UTF-8 char boundary.
                    let after_line_break = unsafe { pos.add(1) };
                    return Ok(after_line_break);
                }
            },
            handle_eof: {
                self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
                return Err(Kind::Eof);
            },
        };

        self.trivia_builder.add_unsearched_block_comment(
            self.token_start,
            self.offset(),
            CommentKind::SingleLineBlock,
            self.source.whole(),
            &mut self.pending,
        );
        Err(Kind::Skip)
    }

    /// Handle a multi-line comment which has no end.
    ///
    /// `rest` is the part of the source which has not been searched for line breaks,
    /// and `has_line_break` is whether there is a line break in the comment before it.
    #[cold]
    fn unterminated_multi_line_comment(&mut self, rest: &[u8], has_line_break: bool) -> Kind {
        if has_line_break || contains_line_break(rest) {
            self.pending.set_is_on_new_line();
        }
        self.source.advance_to_end();
        self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
        Kind::Eof
    }

    /// Section 12.5 Hashbang Comments.
    ///
    /// # SAFETY
    /// Next 2 bytes must be `#!`.
    pub(super) unsafe fn read_hashbang_comment(&mut self) -> Kind {
        debug_assert_eq!(self.peek_2_bytes(), Some([b'#', b'!']));

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
