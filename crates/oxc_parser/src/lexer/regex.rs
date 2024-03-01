use oxc_index::const_assert_eq;

use super::{
    cold_branch,
    search::{byte_search, safe_byte_match_table, SafeByteMatchTable},
    Kind, Lexer, RegExpFlags, Token,
};
use crate::diagnostics;

// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA9];

static REGEX_END_TABLE: SafeByteMatchTable = safe_byte_match_table!(|b| matches!(
    b,
    b'/' | b'[' | b']' | b'\\' | b'\r' | b'\n' | LS_OR_PS_FIRST
));

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum FlagMatch {
    FlagG = 0,
    FlagI = 1,
    FlagM = 2,
    FlagS = 3,
    FlagU = 4,
    FlagY = 5,
    FlagD = 6,
    FlagV = 7,
    AsciiId = 8,
    End = 9,
}

const_assert_eq!(1u8 << (FlagMatch::FlagG as u8), RegExpFlags::G.bits());
const_assert_eq!(1u8 << (FlagMatch::FlagI as u8), RegExpFlags::I.bits());
const_assert_eq!(1u8 << (FlagMatch::FlagM as u8), RegExpFlags::M.bits());
const_assert_eq!(1u8 << (FlagMatch::FlagS as u8), RegExpFlags::S.bits());
const_assert_eq!(1u8 << (FlagMatch::FlagU as u8), RegExpFlags::U.bits());
const_assert_eq!(1u8 << (FlagMatch::FlagY as u8), RegExpFlags::Y.bits());
const_assert_eq!(1u8 << (FlagMatch::FlagD as u8), RegExpFlags::D.bits());
const_assert_eq!(1u8 << (FlagMatch::FlagV as u8), RegExpFlags::V.bits());

#[repr(C, align(64))]
struct FlagMatchTable([FlagMatch; 256]);

static FLAG_MATCH_TABLE: FlagMatchTable = {
    let mut table = FlagMatchTable([FlagMatch::End; 256]);
    let mut b = 0u8;
    loop {
        table.0[b as usize] = match b {
            b'g' => FlagMatch::FlagG,
            b'i' => FlagMatch::FlagI,
            b'm' => FlagMatch::FlagM,
            b's' => FlagMatch::FlagS,
            b'u' => FlagMatch::FlagU,
            b'y' => FlagMatch::FlagY,
            b'd' => FlagMatch::FlagD,
            b'v' => FlagMatch::FlagV,
            _ => {
                if b.is_ascii_alphanumeric() || matches!(b, b'$' | b'_') {
                    FlagMatch::AsciiId
                } else {
                    FlagMatch::End
                }
            }
        };
        if b == 255 {
            break;
        }
        b += 1;
    }
    table
};

impl FlagMatch {
    #[inline]
    fn from_byte(b: u8) -> Self {
        FLAG_MATCH_TABLE.0[b as usize]
    }

    #[inline]
    fn as_flags(self) -> RegExpFlags {
        RegExpFlags::from_bits(1u8 << (self as u8)).unwrap()
    }
}

impl<'a> Lexer<'a> {
    /// Re-tokenize the current `/` or `/=` and return `RegExp`
    /// See Section 12:
    ///   The `InputElementRegExp` goal symbol is used in all syntactic grammar contexts
    ///   where a `RegularExpressionLiteral` is permitted
    /// Which means the parser needs to re-tokenize on `PrimaryExpression`,
    /// `RegularExpressionLiteral` only appear on the right hand side of `PrimaryExpression`
    pub(crate) fn next_regex(&mut self, kind: Kind) -> (Token, u32, RegExpFlags) {
        self.token.start = self.offset()
            - match kind {
                Kind::Slash => 1,
                Kind::SlashEq => 2,
                _ => unreachable!(),
            };
        let (pattern_end, flags) = self.read_regex();
        self.lookahead.clear();
        let token = self.finish_next(Kind::RegExp);
        (token, pattern_end, flags)
    }

    /// 12.9.5 Regular Expression Literals
    fn read_regex(&mut self) -> (u32, RegExpFlags) {
        let mut in_character_class = false;

        byte_search! {
            lexer: self,
            table: REGEX_END_TABLE,
            continue_if: (next_byte, pos) {
                // Match found. Decide whether to continue searching.
                match next_byte {
                    b'/' => {
                        if in_character_class {
                            true
                        } else {
                            let pattern_end = self.source.offset_of(pos);
                            // SAFETY: Next byte is `/`, so `pos + 1` is UTF-8 char boundary
                            self.source.set_position(unsafe { pos.add(1) });
                            let flags = self.read_regex_flags();
                            return (pattern_end, flags);
                        }
                    },
                    b'[' => {
                        in_character_class = true;
                        true
                    }
                    b']' => {
                        in_character_class = false;
                        true
                    }
                    b'\\' => {
                        // SAFETY: Next byte is `\` which is ASCII, so +1 byte is a UTF-8 char boundary
                        let after_backslash = unsafe { pos.add(1) };
                        if after_backslash.addr() < self.source.end_addr() {
                            // SAFETY: Have checked not at EOF, so safe to read a byte
                            let after_backslash_byte = unsafe { after_backslash.read() };
                            if matches!(after_backslash_byte, b'\r' | b'\n' | LS_OR_PS_FIRST) {
                                // `\r`, `\n`, or first byte of PS/LS after backslash.
                                // Continue search, so that if it is a line break (at present could be
                                // some other Unicode char starting with same byte as PS/LS),
                                // then next turn of search will raise an error.
                                // If it's not a line break, search will continue.
                                // Line breaks are illegal in valid JS, and Unicode chars are rare,
                                // so cold branch.
                                cold_branch(|| true)
                            } else {
                                // Skip next byte.
                                // Macro will already advance 1 byte, so this advances 2 bytes total,
                                // past the `\` and the next byte. This may place `pos` in middle of
                                // a multi-byte Unicode character, but `REGEX_END_TABLE` doesn't match
                                // any UTF-8 continuation characters, so in this case `pos` will end up
                                // on a UTF-8 char boundary again after next turn of the search.
                                pos = after_backslash;
                                true
                            }
                        } else {
                            // This is last byte in file. Continue to `handle_eof`.
                            // This is illegal in valid JS, so mark this branch cold.
                            cold_branch(|| true)
                        }
                    },
                    _ => cold_branch(|| {
                        // Likely line break.
                        // Line breaks are illegal in valid JS, and Unicode is also rare, so cold branch.
                        // SAFETY: This may place `pos` in middle of a UTF-8 char, but if so that's
                        // fixed below.
                        pos = unsafe { pos.add(1) };
                        if next_byte == LS_OR_PS_FIRST {
                            // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                            // So safe to read 2 bytes (we already skipped the `0xE2` byte).
                            let next2 = unsafe { pos.read2() };
                            if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                                // Irregular line break. Consume it and stop searching.
                                // SAFETY: Irregular line breaks are 3-byte chars. We consumed 1 byte already.
                                pos = unsafe { pos.add(2) };
                                false
                            } else {
                                // Some other Unicode char beginning with `0xE2`, not a line break.
                                // Skip 3 bytes (already skipped 1, and macro skips 1 more, so skip 1 more
                                // here to make 3), and continue searching.
                                // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                                // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                                pos = unsafe { pos.add(1) };
                                true
                            }
                        } else {
                            // Regular line break. Stop searching, so fall through to `handle_match`
                            // which raises an error. Already consumed the line break.
                            debug_assert!(matches!(next_byte, b'\r' | b'\n'));
                            false
                        }
                    })
                }
            },
            handle_eof: 0, // Fall through to below
        };

        // Line break found (legal end is handled above)
        self.error(diagnostics::UnterminatedRegExp(self.unterminated_range()));
        (self.offset(), RegExpFlags::empty())
    }

    /// Read regex flags.
    #[inline]
    fn read_regex_flags(&mut self) -> RegExpFlags {
        let mut flags = RegExpFlags::empty();
        while let Some(b) = self.source.peek_byte() {
            let maybe_flag = FlagMatch::from_byte(b);
            if maybe_flag == FlagMatch::End {
                break;
            }

            // SAFETY: `FlagMatch::End` covers all Unicode bytes, so consuming 1 byte
            // will leave `source` on a UTF-8 char boundary
            unsafe { self.source.next_byte_unchecked() };

            if maybe_flag == FlagMatch::AsciiId {
                self.error(diagnostics::RegExpFlag(b as char, self.current_offset()));
                continue;
            }

            let flag = maybe_flag.as_flags();
            if flags.contains(flag) {
                self.error(diagnostics::RegExpFlagTwice(b as char, self.current_offset()));
                continue;
            }
            flags |= flag;
        }

        flags
    }
}
