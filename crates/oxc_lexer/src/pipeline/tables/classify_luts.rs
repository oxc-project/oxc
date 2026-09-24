use crate::pipeline::{
    bytes::{is_digit, is_ws},
    operators::is_op_char,
};

use super::keywords::{is_kw_init, is_kw_init_ts};

#[cfg_attr(
    all(
        not(test),
        not(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "popcnt"
        ))
    ),
    expect(dead_code, reason = "only used in SIMD implementation and tests")
)]
pub struct MergedLuts {
    pub lo: [u8; 16],
    pub hi: [u8; 16],
    pub lo_ts: [u8; 16],
}

impl MergedLuts {
    pub fn new() -> Self {
        let mut mrg_lo = [0; 16];
        let mut mrg_hi = [0; 16];
        let mut mrg_lo_ts = [0; 16];

        const ROWS: [(u8, u8, u8); 7] =
            [(6, 0, 0), (7, 1, 0), (2, 2, 1), (3, 3, 1), (5, 4, 1), (7, 5, 1), (2, 7, 2)];

        for &(hi, bit, set) in ROWS.iter() {
            for lo in 0..16u8 {
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
            }
            mrg_hi[hi as usize] |= 1u8 << bit;
        }

        Self { lo: mrg_lo, hi: mrg_hi, lo_ts: mrg_lo_ts }
    }
}

#[cfg_attr(
    all(
        not(test),
        not(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "popcnt"
        ))
    ),
    expect(dead_code, reason = "only used in SIMD implementation and tests")
)]
pub struct WordLuts {
    pub lo: [u8; 16],
    pub hi: [u8; 16],
}

impl WordLuts {
    pub fn new() -> Self {
        let mut wb_lo = [0; 16];
        let mut wb_hi = [0; 16];

        const BROWS: [(u8, u8); 8] =
            [(2, 0), (3, 1), (4, 2), (5, 3), (6, 4), (7, 5), (0, 6), (2, 7)];

        for &(hi, bit) in BROWS.iter() {
            for lo in 0..16u8 {
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
            }
            wb_hi[hi as usize] |= 1u8 << bit;
        }

        Self { lo: wb_lo, hi: wb_hi }
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline::bytes::is_word;

    use super::*;

    #[test]
    fn test_merged_luts() {
        let merged_luts = MergedLuts::new();

        for c in 0..256usize {
            let cb = c as u8;

            let t = if c < 0x80 { merged_luts.lo[c & 15] & merged_luts.hi[c >> 4] } else { 0 };
            let kw = (t & 0x03) != 0;
            let opp = (t & 0x3C) != 0;
            let dt = (t & 0x80) != 0;
            assert!(
                kw == is_kw_init(cb) && opp == is_op_char(cb) && dt == (cb == b'.'),
                "MRG_LO/HI wrong at byte {c:#04x}"
            );

            let ts_t =
                if c < 0x80 { merged_luts.lo_ts[c & 15] & merged_luts.hi[c >> 4] } else { 0 };
            let ts_kw = (ts_t & 0x03) != 0;
            let ts_opp = (ts_t & 0x3C) != 0;
            let ts_dt = (ts_t & 0x80) != 0;
            assert!(
                ts_kw == is_kw_init_ts(cb) && ts_opp == opp && ts_dt == dt,
                "MRG_LO_TS wrong at byte {c:#04x}"
            );
        }
    }

    #[test]
    fn test_word_luts() {
        let word_luts = WordLuts::new();

        for c in 0..256usize {
            let cb = c as u8;

            let tb = if c < 0x80 { word_luts.lo[c & 15] & word_luts.hi[c >> 4] } else { 0 };
            let wd = (c >= 0x80) || (tb & 0x3F) != 0;
            let ws = (tb & 0xC0) != 0;
            let dg = (tb & 0x02) != 0;
            assert!(
                wd == is_word(cb) && ws == is_ws(cb) && dg == is_digit(cb),
                "WB_LO/HI wrong at byte {c:#04x}"
            );
        }
    }
}
