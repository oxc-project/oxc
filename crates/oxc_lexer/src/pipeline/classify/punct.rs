//! Details of single-byte punctuators, and lookup tables which map them to their `TokenKind`s.
//!
//! The lookup tables (`PH_A`, `PH_B` and `PH_T`) use a perfect hash.
//!
//! `vpunct1` does the lookup for 32 bytes at once, with 4 `vpshufb`s. For a byte `c`:
//!
//! 1. `h = PH_A[c & 15] ^ PH_B[c >> 4]` hashes `c` to one of 32 slots.
//!    All entries in `PH_A` and `PH_B` are less than 32, so `h` is too.
//! 2. The kind is `PH_T[h]`.
//!    `vpunct1` looks up the 2 halves of `PH_T` separately, because a `vpshufb` lookup
//!    can only index 16 entries.
//!
//! The hash is perfect for the bytes which matter:
//!
//! - Each punctuator in `PUNCT1` lands in a slot holding its kind.
//! - Every other byte in 0x20 - 0x7F lands in a slot holding `Invalid` (255),
//!   except identifier chars and whitespace. They can land anywhere,
//!   because the SIMD implementations of `classify` overwrite their kind afterwards.
//!
//! Bytes below 0x20 and from 0x80 upwards don't use the hash. `vpunct1` gives them `Invalid` directly.
//!
//! `test_punct1_hash` checks the tables against `PUNCT1`.
//! Nothing generates them, so a change to `PUNCT1` needs new values found for them.

use crate::{
    simd::{VecData128, VecData256},
    token::TokenKind,
};

/// Single-byte punctuator and its [`TokenKind`].
///
/// Only exists at build time and in tests, not referenced from any runtime code.
pub(super) struct Punct1 {
    pub byte: u8,
    pub kind: TokenKind,
}

impl Punct1 {
    const fn new(c: char, kind: TokenKind) -> Self {
        assert!(c.is_ascii());
        Self { byte: c as u8, kind }
    }
}

/// Single-byte punctuators and their [`TokenKind`]s.
/// `#` maps to `Invalid` - a bare `#` is invalid on its own
/// (private names and hashbangs are resolved earlier).
///
/// Does not exist at runtime - used only at compile time and in tests.
pub(super) const PUNCT1: [Punct1; 26] = [
    Punct1::new('(', TokenKind::LParen),
    Punct1::new(')', TokenKind::RParen),
    Punct1::new('[', TokenKind::LBracket),
    Punct1::new(']', TokenKind::RBracket),
    Punct1::new('{', TokenKind::LBrace),
    Punct1::new('}', TokenKind::RBrace),
    Punct1::new(';', TokenKind::Semi),
    Punct1::new(',', TokenKind::Comma),
    Punct1::new('.', TokenKind::Dot),
    Punct1::new('<', TokenKind::Lt),
    Punct1::new('>', TokenKind::Gt),
    Punct1::new('+', TokenKind::Plus),
    Punct1::new('-', TokenKind::Minus),
    Punct1::new('*', TokenKind::Star),
    Punct1::new('/', TokenKind::Slash),
    Punct1::new('%', TokenKind::Percent),
    Punct1::new('&', TokenKind::Amp),
    Punct1::new('|', TokenKind::Pipe),
    Punct1::new('^', TokenKind::Caret),
    Punct1::new('!', TokenKind::Bang),
    Punct1::new('~', TokenKind::Tilde),
    Punct1::new('?', TokenKind::Question),
    Punct1::new(':', TokenKind::Colon),
    Punct1::new('=', TokenKind::Eq),
    Punct1::new('@', TokenKind::At),
    Punct1::new('#', TokenKind::Invalid),
];

pub(super) const PH_A: VecData128 =
    VecData128::from_bytes([4, 13, 19, 20, 0, 14, 7, 8, 10, 26, 22, 0, 29, 23, 3, 2]);
pub(super) const PH_B: VecData128 =
    VecData128::from_bytes([24, 26, 2, 16, 31, 25, 19, 30, 0, 0, 0, 0, 0, 0, 0, 0]);

pub(super) const PH_T: VecData256 = VecData256::from_bytes([
    68, 38, 58, 76, 255, 72, 42, 52, 34, 33, 255, 255, 70, 48, 37, 55, // Low half
    40, 255, 43, 50, 64, 61, 255, 255, 35, 36, 80, 89, 255, 82, 32, 41, // High half
]);

#[cfg(test)]
mod tests {
    use crate::cpu::BASELINE;

    use crate::pipeline::bytes::{is_word, is_ws};

    use super::*;

    #[test]
    fn test_punct1_hash() {
        let mut kinds = [TokenKind::Invalid; 256];
        for punct in &PUNCT1 {
            assert!(
                punct.byte >= 0x20,
                "{:?} is a control char, which SIMD classify maps to Invalid",
                punct.byte as char
            );
            kinds[punct.byte as usize] = punct.kind;
        }

        for byte in 0..=255 {
            // `is_word` returns `true` for all values >= 128, so non-ASCII bytes are skipped
            if is_word(byte, BASELINE) || is_ws(byte, BASELINE) || byte < 0x20 {
                continue;
            }
            assert!(
                punct1_hash(byte) == kinds[byte as usize] as u8,
                "PH_A / PH_B / PH_T wrong at byte {byte:#04x}"
            );
        }
    }

    fn punct1_hash(byte: u8) -> u8 {
        let h = PH_A[(byte & 15) as usize] ^ PH_B[((byte >> 4) & 15) as usize];
        PH_T[h as usize]
    }
}
