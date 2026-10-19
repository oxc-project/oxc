//! Lookup tables which map single-byte punctuators to their `TokenKind`, using a perfect hash.
//!
//! `vpunct1` does the lookup for 32 bytes at once, with 4 `vpshufb`s. For a byte `c`:
//!
//! 1. `h = PH_A[c & 15] ^ PH_B[c >> 4]` hashes `c` to one of 32 slots.
//!    All entries in `PH_A` and `PH_B` are less than 32, so `h` is too.
//! 2. The kind is `PH_T0[h]` if `h < 16`, and `PH_T1[h - 16]` otherwise.
//!    The table is split in two because a `vpshufb` lookup can only index 16 entries.
//!
//! The hash is perfect for the bytes which matter:
//!
//! * Each punctuator in `PUNCT1` lands in a slot holding its kind.
//! * Every other byte in 0x20 - 0x7F lands in a slot holding `Invalid` (255),
//!   except identifier chars and whitespace. They can land anywhere,
//!   because `classify_impl` overwrites their kind afterwards.
//!
//! Bytes below 0x20 and from 0x80 upwards don't use the hash. `vpunct1` gives them `Invalid` directly.
//!
//! `test_punct1_hash` checks the tables against `PUNCT1`.
//! Nothing generates them, so a change to `PUNCT1` needs new values found for them.

pub(super) const PH_A: [u8; 16] = [4, 13, 19, 20, 0, 14, 7, 8, 10, 26, 22, 0, 29, 23, 3, 2];
pub(super) const PH_B: [u8; 16] = [24, 26, 2, 16, 31, 25, 19, 30, 0, 0, 0, 0, 0, 0, 0, 0];
pub(super) const PH_T0: [u8; 16] =
    [68, 38, 58, 76, 255, 72, 42, 52, 34, 33, 255, 255, 70, 48, 37, 55];
pub(super) const PH_T1: [u8; 16] =
    [40, 255, 43, 50, 64, 61, 255, 255, 35, 36, 80, 89, 255, 82, 32, 41];

#[cfg(test)]
mod tests {
    use crate::{
        pipeline::{
            bytes::{is_word, is_ws},
            classify::punct1::PUNCT1,
        },
        token::tk,
    };

    use super::*;

    #[test]
    fn test_punct1_hash() {
        let mut punct1_ord = [tk!(Invalid); 256];
        for i in 0..PUNCT1.len() {
            punct1_ord[PUNCT1[i].byte as usize] = PUNCT1[i].kind as u8;
        }

        for c in 0..256usize {
            let cb = c as u8;
            if is_word(cb) || is_ws(cb) {
                continue;
            }
            assert!(punct1_hash(cb) == punct1_ord[c], "PH_A/B/T wrong at byte {c:#04x}");
        }
    }

    fn punct1_hash(c: u8) -> u8 {
        if c < 0x20 {
            return tk!(Invalid);
        }
        let h = (PH_A[(c & 15) as usize] ^ PH_B[((c >> 4) & 15) as usize]) & 31;
        if h < 16 { PH_T0[h as usize] } else { PH_T1[(h & 15) as usize] }
    }
}
