use crate::pipeline::{
    bytes::{is_digit, is_ws},
    operators::is_op_char,
    tables::{is_kw_init, is_kw_init_ts},
};

/// Nibble lookup tables which classify bytes as keyword-initial letters, operator chars, or `.`.
///
/// "Merged" because one lookup answers all three questions.
/// `classify_impl` looks up each byte's low nibble in `lo` and its high nibble in `hi` with `vpshufb`,
/// and ANDs the results. For an ASCII byte `c`, `lo[c & 15] & hi[c >> 4]` has these bits set:
///
/// * Bits 0-1: `c` can start a keyword.
/// * Bits 2-5: `c` is an operator char.
/// * Bit 7: `c` is `.`. It's the top bit, so a movemask extracts it without a compare.
///
/// Each bit covers only one row of 16 bytes which share a high nibble.
/// `hi[h]` sets the bits whose row is `h`, and `lo[l]` sets a bit if byte `(h << 4) | l` is in its class,
/// so the AND is exact. A class spanning several rows needs a bit per row
/// (e.g. keyword-initial letters take 2 bits, one for `a`-`o` and one for `p`-`z`).
///
/// Bytes >= 0x80 get no bits, because `vpshufb` outputs 0 for an index with its top bit set.
///
/// `lo_ts` is the TypeScript version of `lo`, which has more keyword-initial letters.
/// The other bits, and `hi`, are the same for both.
pub(super) struct MergedLuts {
    pub lo: [u8; 16],
    pub hi: [u8; 16],
    pub lo_ts: [u8; 16],
}

pub(super) const MERGED_LUTS: MergedLuts = {
    let mut mrg_lo = [0; 16];
    let mut mrg_hi = [0; 16];
    let mut mrg_lo_ts = [0; 16];

    const ROWS: [(u8, u8, u8); 7] =
        [(6, 0, 0), (7, 1, 0), (2, 2, 1), (3, 3, 1), (5, 4, 1), (7, 5, 1), (2, 7, 2)];

    let mut i = 0;
    while i < ROWS.len() {
        let (hi, bit, set) = ROWS[i];

        let mut lo = 0u8;
        while lo < 16 {
            let c = (hi << 4) | lo;
            let inset = match set {
                0 => is_kw_init(c),
                1 => is_op_char(c),
                _ => c == b'.',
            };
            if inset {
                mrg_lo[lo as usize] |= 1u8 << bit;
            }
            // TS variant: only the keyword-initial rows differ.
            let inset_ts = if set == 0 { is_kw_init_ts(c) } else { inset };
            if inset_ts {
                mrg_lo_ts[lo as usize] |= 1u8 << bit;
            }

            lo += 1;
        }
        mrg_hi[hi as usize] |= 1u8 << bit;

        i += 1;
    }

    MergedLuts { lo: mrg_lo, hi: mrg_hi, lo_ts: mrg_lo_ts }
};

/// Nibble lookup tables which classify bytes as identifier chars or whitespace.
///
/// Works the same way as [`MergedLuts`].
/// For an ASCII byte `c`, `lo[c & 15] & hi[c >> 4]` has these bits set:
///
/// * Bits 0-5: `c` is an identifier char. One bit per row: `$`, digits, `A`-`O`, `P`-`Z` and `_`,
///   `a`-`o`, `p`-`z`. Bit 1 (digits) also serves as the digit class.
/// * Bit 6: `c` is whitespace other than space (tab, line feed etc).
/// * Bit 7: `c` is space.
///
/// Bytes >= 0x80 get no bits. `classify_impl` counts them as identifier chars separately.
pub(super) struct WordLuts {
    pub lo: [u8; 16],
    pub hi: [u8; 16],
}

pub(super) const WORD_LUTS: WordLuts = {
    let mut wb_lo = [0; 16];
    let mut wb_hi = [0; 16];

    const BROWS: [(u8, u8); 8] = [(2, 0), (3, 1), (4, 2), (5, 3), (6, 4), (7, 5), (0, 6), (2, 7)];

    let mut i = 0;
    while i < BROWS.len() {
        let (hi, bit) = BROWS[i];

        let mut lo = 0u8;
        while lo < 16 {
            let c = (hi << 4) | lo;
            let inset = match bit {
                0 => c == b'$',
                1 => is_digit(c),
                2 => c >= b'A' && c <= b'O',
                3 => (c >= b'P' && c <= b'Z') || c == b'_',
                4 => c >= b'a' && c <= b'o',
                5 => c >= b'p' && c <= b'z',
                6 => is_ws(c) && c != b' ',
                _ => c == b' ',
            };
            if inset {
                wb_lo[lo as usize] |= 1u8 << bit;
            }

            lo += 1;
        }
        wb_hi[hi as usize] |= 1u8 << bit;

        i += 1;
    }

    WordLuts { lo: wb_lo, hi: wb_hi }
};

#[cfg(test)]
mod tests {
    use crate::pipeline::bytes::is_word;

    use super::*;

    #[test]
    fn test_merged_luts() {
        for c in 0..256usize {
            let cb = c as u8;

            let t = if c < 0x80 { MERGED_LUTS.lo[c & 15] & MERGED_LUTS.hi[c >> 4] } else { 0 };
            let kw = (t & 0x03) != 0;
            let opp = (t & 0x3C) != 0;
            let dt = (t & 0x80) != 0;
            assert!(
                kw == is_kw_init(cb) && opp == is_op_char(cb) && dt == (cb == b'.'),
                "MERGED_LUTS.lo / hi wrong at byte {c:#04x}"
            );

            let ts_t =
                if c < 0x80 { MERGED_LUTS.lo_ts[c & 15] & MERGED_LUTS.hi[c >> 4] } else { 0 };
            let ts_kw = (ts_t & 0x03) != 0;
            let ts_opp = (ts_t & 0x3C) != 0;
            let ts_dt = (ts_t & 0x80) != 0;
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
            let wd = (c >= 0x80) || (tb & 0x3F) != 0;
            let ws = (tb & 0xC0) != 0;
            let dg = (tb & 0x02) != 0;
            assert!(
                wd == is_word(cb) && ws == is_ws(cb) && dg == is_digit(cb),
                " WORD_LUTS.lo / hi wrong at byte {c:#04x}"
            );
        }
    }
}
