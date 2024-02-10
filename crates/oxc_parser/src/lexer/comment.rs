use super::{
    cold_branch,
    search::{byte_search, safe_byte_match_table, SafeByteMatchTable},
    Kind, Lexer,
};
use crate::diagnostics;

use oxc_syntax::identifier::is_line_terminator;

const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA9];

static LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\r' | b'\n' | LS_OR_PS_FIRST));

impl<'a> Lexer<'a> {
    /// Section 12.4 Single Line Comment
    pub(super) fn skip_single_line_comment(&mut self) -> Kind {
        // SAFETY: Requirement not to alter `pos` if return `true` from `if_continue` is satisfied
        unsafe {
            byte_search! {
                lexer: self,
                table: LINE_BREAK_TABLE,
                continue_if: |next_byte, pos| {
                    // Match found. Decide whether to continue searching.
                    // If this is end of comment, create trivia, and advance `pos` to after line break.
                    // Do that here rather than in `handle_match`, to avoid branching twice on value of
                    // the matched byte.
                    #[allow(clippy::if_not_else)]
                    if next_byte != LS_OR_PS_FIRST {
                        // `\r` or `\n`
                        self.trivia_builder
                            .add_single_line_comment(self.token.start, self.source.offset_of(pos));
                        // SAFETY: Safe to consume `\r` or `\n` as both are ASCII
                        pos = pos.add(1);
                        // We've found the end. Do not continue searching.
                        false
                    } else {
                        // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
                        // Either way, Unicode is uncommon, so make this a cold branch.
                        cold_branch(|| {
                            // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                            // So safe to advance `pos` by 1 and read 2 bytes.
                            let next2 = pos.add(1).read2();
                            if next2 == LS_BYTES_2_AND_3 || next2 == PS_BYTES_2_AND_3 {
                                // Irregular line break
                                self.trivia_builder
                                    .add_single_line_comment(self.token.start, self.source.offset_of(pos));
                                // Advance `pos` to after this char.
                                // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                                // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                                pos = pos.add(3);
                                // We've found the end. Do not continue searching.
                                false
                            } else {
                                // Some other Unicode char beginning with `0xE2`. Continue searching.
                                true
                            }
                        })
                    }
                },
                handle_match: |_next_byte, _start| {
                    self.token.is_on_new_line = true;
                    Kind::Skip
                },
                handle_eof: |_start| {
                    self.trivia_builder.add_single_line_comment(self.token.start, self.offset());
                    Kind::Skip
                },
            };
        }
    }

    /// Section 12.4 Multi Line Comment
    pub(super) fn skip_multi_line_comment(&mut self) -> Kind {
        while let Some(c) = self.next_char() {
            if c == '*' && self.next_eq('/') {
                self.trivia_builder.add_multi_line_comment(self.token.start, self.offset());
                return Kind::Skip;
            }
            if is_line_terminator(c) {
                self.token.is_on_new_line = true;
            }
        }
        self.error(diagnostics::UnterminatedMultiLineComment(self.unterminated_range()));
        Kind::Eof
    }

    /// Section 12.5 Hashbang Comments
    pub(super) fn read_hashbang_comment(&mut self) -> Kind {
        while let Some(c) = self.next_char().as_ref() {
            if is_line_terminator(*c) {
                break;
            }
        }
        self.token.is_on_new_line = true;
        Kind::HashbangComment
    }
}
