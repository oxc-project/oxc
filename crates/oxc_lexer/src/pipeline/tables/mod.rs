use crate::pipeline::bytes::{is_digit, is_word, is_ws};

mod keywords;
mod operators;
mod pair_luts;
mod punct1;

pub(super) use keywords::{KwSet, is_kw_init, is_kw_init_ts};
pub(super) use operators::is_op_char;

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub(super) use punct1::PUNCT1;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
pub(super) use punct1::{PH_A, PH_B, PH_T0, PH_T1};

use keywords::Keywords;
use operators::{OpMap, opch_selfcheck};
use pair_luts::PairLuts;
use punct1::punct1_hash_selfcheck;

pub(super) struct Tables {
    pub op: OpMap,
    pub keywords: Keywords,
    pub mrg_lo: [u8; 16],
    pub mrg_hi: [u8; 16],
    pub mrg_lo_ts: [u8; 16],
    pub wb_lo: [u8; 16],
    pub wb_hi: [u8; 16],
    pub pair_luts: PairLuts,
}

impl Tables {
    pub fn new() -> Tables {
        let op = OpMap::new();
        let mut t = Tables {
            op,
            keywords: Keywords::new(),
            mrg_lo: [0; 16],
            mrg_hi: [0; 16],
            mrg_lo_ts: [0; 16],
            wb_lo: [0; 16],
            wb_hi: [0; 16],
            pair_luts: PairLuts::new(),
        };
        t.build_merged_luts();
        t.pair_luts.build();
        opch_selfcheck();
        t.merged_selfcheck();
        punct1_hash_selfcheck();
        t.keywords.self_check();
        t
    }

    fn build_merged_luts(&mut self) {
        self.mrg_lo = [0; 16];
        self.mrg_hi = [0; 16];
        self.mrg_lo_ts = [0; 16];
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
                    self.mrg_lo[lo as usize] |= 1u8 << bit;
                }
                // TS variant: only the keyword-initial rows differ.
                let inset_ts = if set == 0 { is_kw_init_ts(c) } else { inset };
                if inset_ts {
                    self.mrg_lo_ts[lo as usize] |= 1u8 << bit;
                }
            }
            self.mrg_hi[hi as usize] |= 1u8 << bit;
        }
        self.wb_lo = [0; 16];
        self.wb_hi = [0; 16];
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
                    self.wb_lo[lo as usize] |= 1u8 << bit;
                }
            }
            self.wb_hi[hi as usize] |= 1u8 << bit;
        }
    }

    fn merged_selfcheck(&self) {
        for c in 0..256usize {
            let cb = c as u8;
            let t = if c < 0x80 { self.mrg_lo[c & 15] & self.mrg_hi[c >> 4] } else { 0 };
            let kw = (t & 0x03) != 0;
            let opp = (t & 0x3C) != 0;
            let dt = (t & 0x80) != 0;
            assert!(
                kw == is_kw_init(cb) && opp == is_op_char(cb) && dt == (cb == b'.'),
                "tables.rs: MRG_LO/HI wrong at byte {c:#04x}"
            );
            let tt = if c < 0x80 { self.mrg_lo_ts[c & 15] & self.mrg_hi[c >> 4] } else { 0 };
            assert!(
                ((tt & 0x03) != 0) == is_kw_init_ts(cb)
                    && ((tt & 0x3C) != 0) == opp
                    && ((tt & 0x80) != 0) == dt,
                "tables.rs: MRG_LO_TS wrong at byte {c:#04x}"
            );
            let tb = if c < 0x80 { self.wb_lo[c & 15] & self.wb_hi[c >> 4] } else { 0 };
            let wd = (c >= 0x80) || (tb & 0x3F) != 0;
            let ws = (tb & 0xC0) != 0;
            let dg = (tb & 0x02) != 0;
            assert!(
                wd == is_word(cb) && ws == is_ws(cb) && dg == is_digit(cb),
                "tables.rs: WB_LO/HI wrong at byte {c:#04x}"
            );
        }
    }
}
