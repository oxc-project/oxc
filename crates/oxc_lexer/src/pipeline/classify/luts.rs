use oxc_simd::VecData128;

use crate::pipeline::keywords::{is_kw_init, is_kw_init_ts};

/// Nibble lookup tables which classify bytes as keyword-initial letters, operator chars, or `.`.
///
/// "Merged" because one lookup answers all three questions.
/// The SIMD implementations of `classify` look up each byte's low nibble in `lo`
/// and its high nibble in `hi`, and AND the results.
/// For an ASCII byte `b`, `lo[b & 15] & hi[b >> 4]` has these bits set:
///
/// - Bits 0-1: Keyword start.
/// - Bits 2-5: Operator char.
/// - Bit  7:   Dot (`.`).
///
/// Bit 6 is unused.
///
/// Dot is the top bit, so a movemask extracts it without a compare.
///
/// AVX2 detects operator chars with a signed `> KWINIT_MASK`, then adds `.` from the dot bit.
/// Neon detects them with an unsigned `> KWINIT_MASK`, which includes `.`.
/// Both rely on bit 6 staying unused.
///
/// Each bit covers only one row of 16 bytes which share a high nibble.
/// `hi[h]` sets the bits whose row is `h`, and `lo[l]` sets a bit if byte `(h << 4) | l`
/// is in its class, so the AND is exact. A class spanning several rows needs a bit per row
/// (e.g. keyword-initial letters take 2 bits, one for `a`-`o` and one for `p`-`z`).
///
/// Non-ASCII bytes (>= 0x80) get no bits set, because `hi` has no entries for rows 8-15.
/// Additionally, on AVX2 `vpshufb` outputs 0 for an index with its top bit set, so `lo[b]`
/// (`b` without `& 15` mask) evaluates to 0. Neon masks `b & 15` and relies on `hi`.
///
/// `lo_ts` is the TypeScript version of `lo`, which has more keyword-initial letters.
/// The other bits, and `hi`, are the same for both.
pub(super) struct MergedLuts {
    pub lo: VecData128,
    pub hi: VecData128,
    pub lo_ts: VecData128,
}

/// Bits of a [`MERGED_LUTS`] lookup which are set if a byte can start a keyword.
pub(super) const KWINIT_MASK: u8 = 0b_0000_0011;
/// Bits of a [`MERGED_LUTS`] lookup which are set if a byte is an operator char.
///
/// Only used in tests. AVX2 and Neon both compare against [`KWINIT_MASK`] instead.
#[cfg(test)]
pub(super) const OPCH_MASK: u8 = 0b_0011_1100;
/// Bit of a [`MERGED_LUTS`] lookup which is set if a byte is `.`.
pub(super) const DOT_MASK: u8 = 0b_1000_0000;

pub(super) const MERGED_LUTS: MergedLuts = {
    let mut lo = [0; 16];
    let mut hi = [0; 16];
    let mut lo_ts = [0; 16];

    let mut b = 0u8;
    while b < 128 {
        // `bits_ts` is the TS variant - only the keyword-start bits differ
        let mut bits = 0;
        let mut bits_ts = 0;

        let keyword_bit_index = match b {
            b'a'..=b'o' => Some(0), // Row 6
            b'p'..=b'z' => Some(1), // Row 7
            _ => None,
        };
        if let Some(bit_index) = keyword_bit_index {
            if is_kw_init(b) {
                bits |= 1 << bit_index;
            } else if is_kw_init_ts(b) {
                bits_ts |= 1 << bit_index;
            }
        }

        let op_bit_index = match b {
            b'!' | b'%' | b'&' | b'*' | b'+' | b'-' | b'.' => Some(2), // Row 2
            b'<' | b'=' | b'>' | b'?' => Some(3),                      // Row 3
            b'^' => Some(4),                                           // Row 5
            b'|' => Some(5),                                           // Row 7
            _ => None,
        };
        if let Some(bit_index) = op_bit_index {
            bits |= 1 << bit_index;
        }

        if b == b'.' {
            bits |= 1 << 7;
        }

        lo[(b & 15) as usize] |= bits;
        lo_ts[(b & 15) as usize] |= bits | bits_ts;
        hi[(b >> 4) as usize] |= bits | bits_ts;

        b += 1;
    }

    MergedLuts {
        lo: VecData128::from_bytes(lo),
        hi: VecData128::from_bytes(hi),
        lo_ts: VecData128::from_bytes(lo_ts),
    }
};

/// Nibble lookup tables which classify bytes as identifier chars or whitespace.
///
/// Works the same way as [`MergedLuts`].
/// For an ASCII byte `b`, `lo[b & 15] & hi[b >> 4]` has these bits set:
///
/// - Bits 0-4: Identifier chars, including digits (a-z, A-Z, 0-9, _, $).
/// - Bit  1:   Digits (0-9).
/// - Bits 5-7: ASCII whitespace (space, tab, LF, CR, VT, FF).
/// - Bit  7:   Irregular ASCII whitespace (VT, FF).
///
/// Non-ASCII bytes (>= 0x80) get no bits.
/// The SIMD implementations of `classify` count them as identifier chars separately.
pub(super) struct WordLuts {
    pub lo: VecData128,
    pub hi: VecData128,
}

/// Bits of a [`WORD_LUTS`] lookup which are set if a byte is an identifier char.
pub(super) const WORD_MASK: u8 = 0b_0001_1111;
/// Bit of a [`WORD_LUTS`] lookup which is set if a byte is a digit.
pub(super) const DIGIT_MASK: u8 = 0b_0000_0010;
/// Bits of a [`WORD_LUTS`] lookup which are set if a byte is whitespace.
pub(super) const WS_MASK: u8 = 0b_1110_0000;
/// Bit of a [`WORD_LUTS`] lookup which is set if a byte is vertical tab or form feed.
pub(super) const VT_FF_MASK: u8 = 0b_1000_0000;

pub(super) const WORD_LUTS: WordLuts = {
    const VT: u8 = 0xB;
    const FF: u8 = 0xC;

    let mut lo = [0; 16];
    let mut hi = [0; 16];

    let mut b = 0u8;
    while b < 128 {
        let bit_index = match b {
            // Word bits.
            // A-O and a-o share bit 2, which stays exact because their low nibbles are the same.
            b'$' => Some(0),               // Row 2
            b'0'..=b'9' => Some(1),        // Row 3
            b'A'..=b'O' => Some(2),        // Row 4
            b'a'..=b'o' => Some(2),        // Row 6
            b'P'..=b'Z' | b'_' => Some(3), // Row 5
            b'p'..=b'z' => Some(4),        // Row 7

            // Whitespace bits.
            // Space is in a different row from the others, so needs its own bit.
            // VT and FF have their own bit, for `VT_FF_MASK`.
            b' ' => Some(5),                  // Row 2
            b'\t' | b'\n' | b'\r' => Some(6), // Row 0
            VT | FF => Some(7),               // Row 0

            _ => None,
        };

        if let Some(bit_index) = bit_index {
            let bits = 1 << bit_index;
            lo[(b & 15) as usize] |= bits;
            hi[(b >> 4) as usize] |= bits;
        }

        b += 1;
    }

    WordLuts { lo: VecData128::from_bytes(lo), hi: VecData128::from_bytes(hi) }
};

#[cfg(test)]
mod tests {
    use oxc_simd::BASELINE;

    use crate::pipeline::{
        bytes::{is_digit, is_word, is_ws},
        operators::is_op_char,
    };

    use super::*;

    #[test]
    fn test_merged_luts() {
        for c in 0..256usize {
            let cb = c as u8;

            let t = if c < 0x80 { MERGED_LUTS.lo[c & 15] & MERGED_LUTS.hi[c >> 4] } else { 0 };
            let kw = (t & KWINIT_MASK) != 0;
            let opp = (t & OPCH_MASK) != 0;
            let dt = (t & DOT_MASK) != 0;
            assert!(
                kw == is_kw_init(cb) && opp == is_op_char(cb, BASELINE) && dt == (cb == b'.'),
                "MERGED_LUTS.lo / hi wrong at byte {c:#04x}"
            );

            let ts_t =
                if c < 0x80 { MERGED_LUTS.lo_ts[c & 15] & MERGED_LUTS.hi[c >> 4] } else { 0 };
            let ts_kw = (ts_t & KWINIT_MASK) != 0;
            let ts_opp = (ts_t & OPCH_MASK) != 0;
            let ts_dt = (ts_t & DOT_MASK) != 0;
            assert!(
                ts_kw == is_kw_init_ts(cb) && ts_opp == opp && ts_dt == dt,
                "MERGED_LUTS.lo_ts wrong at byte {c:#04x}"
            );
        }
    }

    #[test]
    fn test_word_luts() {
        for c in 0..256usize {
            let cb = c as u8;

            let tb = if c < 0x80 { WORD_LUTS.lo[c & 15] & WORD_LUTS.hi[c >> 4] } else { 0 };
            let wd = (c >= 0x80) || (tb & WORD_MASK) != 0;
            let ws = (tb & WS_MASK) != 0;
            let dg = (tb & DIGIT_MASK) != 0;
            let vt_ff = (tb & VT_FF_MASK) != 0;
            assert!(
                wd == is_word(cb, BASELINE)
                    && ws == is_ws(cb, BASELINE)
                    && dg == is_digit(cb, BASELINE)
                    && vt_ff == matches!(cb, 0x0b | 0x0c),
                " WORD_LUTS.lo / hi wrong at byte {c:#04x}"
            );
        }
    }
}
