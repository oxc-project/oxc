use memchr::memmem::Finder;

use oxc_ast::CommentKind;
use oxc_data_structures::branch_hints::cold_path;
use oxc_syntax::line_terminator::is_line_terminator;

use crate::{config::LexerConfig as Config, diagnostics};

use super::{
    Kind, Lexer,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
    source::SourcePosition,
};

// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA9];

static LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\r' | b'\n' | LS_OR_PS_FIRST));

/// Find a line break candidate, or a possible block-comment terminator.
/// The first 0xE2 byte is checked by the caller for Unicode LS/PS.
#[inline]
fn find_comment_special<const BLOCK: bool>(bytes: &[u8]) -> Option<usize> {
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;

    let mut offset = 0;
    while offset + 8 <= bytes.len() {
        let word = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        let mask = |byte| {
            let value = word ^ (LO * u64::from(byte));
            value.wrapping_sub(LO) & !value & HI
        };
        let mut matches = mask(b'\r') | mask(b'\n') | mask(LS_OR_PS_FIRST);
        if BLOCK {
            matches |= mask(b'*');
        }
        if matches != 0 {
            return Some(offset + matches.trailing_zeros() as usize / 8);
        }
        offset += 8;
    }
    bytes[offset..]
        .iter()
        .position(|&b| matches!(b, b'\r' | b'\n' | LS_OR_PS_FIRST) || (BLOCK && b == b'*'))
        .map(|index| offset + index)
}

impl<'a, C: Config> Lexer<'a, C> {
    /// Section 12.4 Single Line Comment
    pub(super) fn skip_single_line_comment(&mut self, kind: CommentKind) -> Kind {
        let start = self.source.position();
        let remaining = self.source.remaining().as_bytes();
        let Some(index) = memchr::memchr3(b'\r', b'\n', LS_OR_PS_FIRST, remaining) else {
            self.source.advance_to_end();
            self.trivia_builder.add_line_comment(
                self.token.start(),
                self.offset(),
                kind,
                self.source.whole(),
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
            self.token.start(),
            self.source.offset_of(end),
            kind,
            self.source.whole(),
        );
        // SAFETY: Consume the matching ASCII line break.
        self.source.set_position(unsafe { end.add(1) });
        self.token.set_is_on_new_line(true);
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
                        .add_line_comment(self.token.start(), self.source.offset_of(pos), kind, self.source.whole());
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
                            .add_line_comment(self.token.start(), self.source.offset_of(pos), kind, self.source.whole());
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
                self.trivia_builder.add_line_comment(self.token.start(), self.offset(), kind, self.source.whole());
                return Kind::Skip;
            },
        };

        self.token.set_is_on_new_line(true);
        Kind::Skip
    }

    /// Section 12.4 Multi Line Comment
    pub(super) fn skip_multi_line_comment(&mut self) -> Kind {
        let mut pos = self.source.position();
        loop {
            let remaining = self.source.str_from_pos_to_end(pos).as_bytes();
            let Some(index) = find_comment_special::<true>(remaining) else {
                self.source.advance_to_end();
                self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
                return Kind::Eof;
            };
            // SAFETY: The matching byte is ASCII or the UTF-8 leading byte 0xE2.
            pos = unsafe { pos.add(index) };
            match remaining[index] {
                b'*' => {
                    if remaining.get(index + 1) == Some(&b'/') {
                        // SAFETY: Both bytes of the closing delimiter are present.
                        self.source.set_position(unsafe { pos.add(2) });
                        break;
                    }
                    // SAFETY: The matching ASCII byte is present, including at EOF.
                    pos = unsafe { pos.add(1) };
                }
                LS_OR_PS_FIRST => {
                    cold_path();
                    // SAFETY: 0xE2 starts a three-byte character in valid UTF-8.
                    let next2 = unsafe { pos.add(1).read2() };
                    // SAFETY: Consuming the full character preserves its UTF-8 boundary.
                    pos = unsafe { pos.add(3) };
                    if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                        self.token.set_is_on_new_line(true);
                        return self.skip_multi_line_comment_after_line_break(pos);
                    }
                }
                _ => {
                    self.token.set_is_on_new_line(true);
                    // SAFETY: The matching CR or LF is present and is ASCII.
                    let after_line_break = unsafe { pos.add(1) };
                    return self.skip_multi_line_comment_after_line_break(after_line_break);
                }
            }
        }
        self.trivia_builder.add_block_comment(
            self.token.start(),
            self.offset(),
            CommentKind::SingleLineBlock,
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
                CommentKind::MultiLineBlock,
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

#[cfg(test)]
mod tests {
    use super::find_comment_special;

    #[test]
    fn swar_comment_search_matches_scalar() {
        fn check<const BLOCK: bool>() {
            let expected = |bytes: &[u8]| {
                bytes
                    .iter()
                    .position(|&b| matches!(b, b'\r' | b'\n' | 0xE2) || (BLOCK && b == b'*'))
            };
            for len in 0..=33 {
                let mut bytes = vec![b'x'; len];
                for index in 0..len {
                    for byte in 0..=u8::MAX {
                        bytes[index] = byte;
                        assert_eq!(find_comment_special::<BLOCK>(&bytes), expected(&bytes));
                    }
                    bytes[index] = b'x';
                }
            }
            // Borrow propagation may mark a byte after the actual match, but must
            // never change the position of the first match.
            for special in [b'\r', b'\n', 0xE2, b'*'] {
                let bytes = [b'x', b'x', special, special ^ 1, b'x', b'x', b'x', b'x'];
                assert_eq!(find_comment_special::<BLOCK>(&bytes), expected(&bytes));
            }
            let mut seed = 0x1234_5678_u32;
            for len in 0..=96 {
                for _ in 0..32 {
                    let bytes: Vec<_> = std::iter::repeat_with(|| {
                        seed ^= seed << 13;
                        seed ^= seed >> 17;
                        seed ^= seed << 5;
                        seed.to_le_bytes()[0]
                    })
                    .take(len)
                    .collect();
                    assert_eq!(find_comment_special::<BLOCK>(&bytes), expected(&bytes));
                }
            }
        }
        check::<false>();
        check::<true>();
    }
}
