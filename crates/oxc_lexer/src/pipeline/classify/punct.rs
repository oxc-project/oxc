//! Details of single-byte punctuators, and lookup tables which map them to their `TokenKind`s.
//!
//! The lookup tables (`PUNCT_HASH_LO`, `PUNCT_HASH_HI` and `PUNCT_HASH_KINDS`) use a perfect hash.
//!
//! AVX2's `punct_kinds` does the lookup for 32 bytes at once, with 4 `vpshufb`s. For a byte `c`:
//!
//! 1. `h = PUNCT_HASH_LO[c & 15] ^ PUNCT_HASH_HI[c >> 4]` hashes `c` to one of 32 slots.
//!    All entries in `PUNCT_HASH_LO` and `PUNCT_HASH_HI` are less than 32, so `h` is too.
//! 2. The kind is `PUNCT_HASH_KINDS[h]`.
//!    AVX2's `punct_kinds` looks up the 2 halves of `PUNCT_HASH_KINDS` separately,
//!    because a `vpshufb` lookup can only index 16 entries.
//!
//! The hash is perfect for the bytes which matter:
//!
//! - Each punctuator in `PUNCT_CHARS` lands in a slot holding its kind.
//! - Every other byte in 0x1F - 0x7F lands in a slot holding `Invalid` (255),
//!   except identifier chars and whitespace. They can land anywhere,
//!   because the SIMD implementations of `classify` overwrite their kind afterwards.
//!
//! Bytes below 0x20 and from 0x80 upwards don't use the hash.
//! `punct_kinds` gives them `Invalid` directly.
//!
//! Except: AVX2's `punct_kinds` compares against 0x1F, not 0x20, so LLVM can reuse the splat
//! of `WORD_MASK` (also 0x1F). So on AVX2, control char 0x1F uses the hash too.
//!
//! Neon's `punct_kinds` uses `PUNCT_HASH_HI_NEON` and `PUNCT_HASH_KINDS_NEON` instead,
//! which are derived from `PUNCT_HASH_HI` and `PUNCT_HASH_KINDS`. They give control chars
//! `Invalid` without a compare:
//!
//! - `PUNCT_HASH_HI_NEON` is 0x80 for rows 0 and 1, so a control char's `h` is 0x80 or more.
//!   `tbl` gives 0 for an index of 32 or more.
//! - `PUNCT_HASH_KINDS_NEON` holds each kind + 1, and `punct_kinds` subtracts 1 after the
//!   lookup. That gives back the kind, and turns the 0 for a control char into `Invalid` (255).
//!
//! `test_punct_hash` and `test_punct_hash_neon` check the tables against `PUNCT_CHARS`.
//! Nothing generates them, so a change to `PUNCT_CHARS` needs new values found for them.

use oxc_simd::{VecData128, VecData256};

use crate::token::TokenKind;

/// Single-byte punctuator and its [`TokenKind`].
///
/// Only exists at build time and in tests, not referenced from any runtime code.
pub(super) struct PunctChar {
    pub byte: u8,
    pub kind: TokenKind,
}

impl PunctChar {
    const fn new(c: char, kind: TokenKind) -> Self {
        assert!(c.is_ascii());
        Self { byte: c as u8, kind }
    }
}

/// Single-byte punctuators and their [`TokenKind`]s.
/// `#` maps to `Invalid` - a bare `#` is invalid on its own
/// (private names and hashbangs are resolved later).
///
/// Does not exist at runtime - used only at compile time and in tests.
pub(super) const PUNCT_CHARS: [PunctChar; 26] = [
    PunctChar::new('(', TokenKind::LParen),
    PunctChar::new(')', TokenKind::RParen),
    PunctChar::new('[', TokenKind::LBracket),
    PunctChar::new(']', TokenKind::RBracket),
    PunctChar::new('{', TokenKind::LBrace),
    PunctChar::new('}', TokenKind::RBrace),
    PunctChar::new(';', TokenKind::Semi),
    PunctChar::new(',', TokenKind::Comma),
    PunctChar::new('.', TokenKind::Dot),
    PunctChar::new('<', TokenKind::Lt),
    PunctChar::new('>', TokenKind::Gt),
    PunctChar::new('+', TokenKind::Plus),
    PunctChar::new('-', TokenKind::Minus),
    PunctChar::new('*', TokenKind::Star),
    PunctChar::new('/', TokenKind::Slash),
    PunctChar::new('%', TokenKind::Percent),
    PunctChar::new('&', TokenKind::Amp),
    PunctChar::new('|', TokenKind::Pipe),
    PunctChar::new('^', TokenKind::Caret),
    PunctChar::new('!', TokenKind::Bang),
    PunctChar::new('~', TokenKind::Tilde),
    PunctChar::new('?', TokenKind::Question),
    PunctChar::new(':', TokenKind::Colon),
    PunctChar::new('=', TokenKind::Eq),
    PunctChar::new('@', TokenKind::At),
    PunctChar::new('#', TokenKind::Invalid),
];

pub(super) const PUNCT_HASH_LO: VecData128 =
    VecData128::from_bytes([4, 13, 19, 20, 0, 14, 7, 8, 10, 26, 22, 0, 29, 23, 3, 2]);
pub(super) const PUNCT_HASH_HI: VecData128 =
    VecData128::from_bytes([24, 6, 2, 16, 31, 25, 19, 30, 0, 0, 0, 0, 0, 0, 0, 0]);

// Every entry must be a declared TokenKind, which Kinds::new relies on.
pub(super) const PUNCT_HASH_KINDS: VecData256 = VecData256::from_bytes([
    68, 38, 58, 76, 255, 72, 42, 52, 34, 33, 255, 255, 70, 48, 37, 55, // Low half
    40, 255, 43, 50, 64, 61, 255, 255, 35, 36, 80, 89, 255, 82, 32, 41, // High half
]);

/// [`PUNCT_HASH_HI`] for Neon, with rows 0 and 1 (control chars) set to 0x80.
pub(super) const PUNCT_HASH_HI_NEON: VecData128 = {
    let mut hi = *PUNCT_HASH_HI.as_bytes();
    hi[0] = 0x80;
    hi[1] = 0x80;
    VecData128::from_bytes(hi)
};

/// [`PUNCT_HASH_KINDS`] for Neon, holding each kind + 1 (wrapping, so `Invalid` is stored as 0).
pub(super) const PUNCT_HASH_KINDS_NEON: VecData256 = {
    let kinds = PUNCT_HASH_KINDS.as_bytes();
    let mut entries = [0; 32];
    let mut i = 0;
    while i < kinds.len() {
        entries[i] = kinds[i].wrapping_add(1);
        i += 1;
    }
    VecData256::from_bytes(entries)
};

#[cfg(test)]
mod tests {
    use oxc_simd::BASELINE;

    use crate::pipeline::bytes::{is_word, is_ws};

    use super::*;

    #[test]
    fn test_punct_hash() {
        let kinds = expected_kinds();
        for byte in 0..=255 {
            // `is_word` returns `true` for all values >= 128, so non-ASCII bytes are skipped.
            // 0x1F is checked, because AVX2's `punct_kinds` sends it through the hash.
            if is_word(byte, BASELINE) || is_ws(byte, BASELINE) || byte < 0x1F {
                continue;
            }
            assert!(
                punct_hash(byte) == kinds[byte as usize] as u8,
                "PUNCT_HASH_LO / PUNCT_HASH_HI / PUNCT_HASH_KINDS wrong at byte {byte:#04x}"
            );
        }
    }

    #[test]
    fn test_punct_hash_neon() {
        let kinds = expected_kinds();
        for byte in 0..=255 {
            // `is_word` returns `true` for all values >= 128, so non-ASCII bytes are skipped.
            // Control chars are checked, because Neon's `punct_kinds` sends them through the hash.
            if is_word(byte, BASELINE) || is_ws(byte, BASELINE) {
                continue;
            }
            assert!(
                punct_hash_neon(byte) == kinds[byte as usize] as u8,
                "Neon punct hash tables wrong at byte {byte:#04x}"
            );
        }
    }

    /// Get the kind of every byte, according to `PUNCT_CHARS`. Bytes not in it are `Invalid`.
    fn expected_kinds() -> [TokenKind; 256] {
        let mut kinds = [TokenKind::Invalid; 256];
        for punct in &PUNCT_CHARS {
            assert!(
                punct.byte >= 0x20,
                "{:?} is a control char, which SIMD classify maps to Invalid",
                punct.byte as char
            );
            kinds[punct.byte as usize] = punct.kind;
        }
        kinds
    }

    /// Look up the kind of `byte` the same way as AVX2's `punct_kinds`, for an ASCII byte >= 0x1F.
    fn punct_hash(byte: u8) -> u8 {
        let h = PUNCT_HASH_LO[(byte & 15) as usize] ^ PUNCT_HASH_HI[((byte >> 4) & 15) as usize];
        PUNCT_HASH_KINDS[h as usize]
    }

    /// Look up the kind of `byte` the same way as Neon's `punct_kinds`, for an ASCII byte.
    fn punct_hash_neon(byte: u8) -> u8 {
        let h =
            PUNCT_HASH_LO[(byte & 15) as usize] ^ PUNCT_HASH_HI_NEON[((byte >> 4) & 15) as usize];
        // `tbl` gives 0 for an index of 32 or more
        let entry = if h < 32 { PUNCT_HASH_KINDS_NEON[h as usize] } else { 0 };
        entry.wrapping_sub(1)
    }
}
