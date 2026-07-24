// Kernel lint policy — see the note in `pipeline/mod.rs`.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::collapsible_if,
    clippy::collapsible_match
)]

use crate::opmap::{
    KEYWORDS, KEYWORDS_TS, KW_HASH_HINT_JS, KW_HASH_HINT_TS, KW_KIND_BASE, KwSet, OPMAP_NOPS,
    OPMAP_OPS, OpMap, PUNCT1_KIND_UNKNOWN, PUNCT1_LIST, PUNCT1_NKNOWN, PUNCT1_TOK, op_key,
};

#[inline(always)]
pub const fn is_ws(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0c || c == 0x0b
}
#[inline(always)]
pub const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}
#[inline(always)]
pub const fn is_id_start(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' || c == b'$' || c >= 0x80
}
#[inline(always)]
pub const fn is_word(c: u8) -> bool {
    is_id_start(c) || is_digit(c)
}
#[inline(always)]
pub fn hex_val(c: u8) -> u32 {
    if c >= b'0' && c <= b'9' {
        return (c - b'0') as u32;
    }
    let l = c | 0x20;
    if l >= b'a' && l <= b'f' {
        return (l - b'a' + 10) as u32;
    }
    255
}
#[inline(always)]
pub fn is_glue_join(c: u8) -> bool {
    is_word(c) || c == b'.' || c == b'+' || c == b'-' || c == b'?'
}

pub const KWINIT_LO: [u8; 16] = [0, 1, 3, 3, 3, 1, 3, 3, 0, 3, 0, 0, 1, 0, 1, 1];
pub const KWINIT_HI: [u8; 16] = [0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0];
#[inline(always)]
pub const fn is_kw_init(c: u8) -> bool {
    (KWINIT_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}
/// TS-mode variant: the JS set plus `k`/`m`/`p`/`u` (keyof, module, the
/// p-words, unique/unknown/undefined/using). Same hi-nibble rows.
pub const KWINIT_TS_LO: [u8; 16] = [2, 1, 3, 3, 3, 3, 3, 3, 0, 3, 0, 1, 1, 1, 1, 1];
#[inline(always)]
pub const fn is_kw_init_ts(c: u8) -> bool {
    (KWINIT_TS_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}
pub const OPCH_LO: [u8; 16] = [0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 10, 3, 7, 2];
pub const OPCH_HI: [u8; 16] = [0, 0, 1, 2, 0, 4, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];
#[inline(always)]
pub const fn is_op_char(c: u8) -> bool {
    (OPCH_LO[(c & 15) as usize] & OPCH_HI[(c >> 4) as usize]) != 0
}
pub const PH_A: [u8; 16] = [4, 13, 19, 20, 0, 14, 7, 8, 10, 26, 22, 0, 29, 23, 3, 2];
pub const PH_B: [u8; 16] = [24, 26, 2, 16, 31, 25, 19, 30, 0, 0, 0, 0, 0, 0, 0, 0];
pub const PH_T0: [u8; 16] = [68, 38, 58, 76, 255, 72, 42, 52, 34, 33, 255, 255, 70, 48, 37, 55];
pub const PH_T1: [u8; 16] = [40, 255, 43, 50, 64, 61, 255, 255, 35, 36, 80, 89, 255, 82, 32, 41];
#[inline(always)]
pub fn punct1_hash(c: u8) -> u8 {
    if c < 0x20 {
        return PUNCT1_KIND_UNKNOWN;
    }
    let h = (PH_A[(c & 15) as usize] ^ PH_B[((c >> 4) & 15) as usize]) & 31;
    if h < 16 { PH_T0[h as usize] } else { PH_T1[(h & 15) as usize] }
}

#[repr(C, align(64))]
pub struct PairLuts {
    pub lut0z: [[u8; 8]; 256],
    pub lutpad: [[u8; 32]; 256],
}

const _: () = assert!(core::mem::size_of::<[[u8; 8]; 256]>() % 64 == 0);

pub struct Tables {
    pub op: OpMap,
    pub kwjs: KwSet,
    pub kwts: KwSet,
    pub regex_kw_mask: u64,
    pub mrg_lo: [u8; 16],
    pub mrg_hi: [u8; 16],
    pub mrg_lo_ts: [u8; 16],
    pub wb_lo: [u8; 16],
    pub wb_hi: [u8; 16],
    pub op2_pack: [u32; 256],
    pub op3_pack: [u64; 256],
    pub pair_luts: PairLuts,
}

impl Tables {
    pub fn new() -> Tables {
        let op = OpMap::new();
        let kwjs = KwSet::build(&KEYWORDS, false, &[25, 24], KW_HASH_HINT_JS);
        let kwts = KwSet::build(&KEYWORDS_TS, true, &[23], KW_HASH_HINT_TS);
        kwjs.self_check(&KEYWORDS);
        kwts.self_check(&KEYWORDS_TS);
        let mut t = Tables {
            op,
            kwjs,
            kwts,
            regex_kw_mask: 0,
            mrg_lo: [0; 16],
            mrg_hi: [0; 16],
            mrg_lo_ts: [0; 16],
            wb_lo: [0; 16],
            wb_hi: [0; 16],
            op2_pack: [0; 256],
            op3_pack: [0; 256],
            pair_luts: PairLuts { lut0z: [[0; 8]; 256], lutpad: [[0; 32]; 256] },
        };
        t.build_regex_kw_mask();
        t.build_op_pack();
        t.build_merged_luts();
        t.build_pair_luts();
        kwinit_selfcheck();
        opch_selfcheck();
        t.merged_selfcheck();
        punct1_hash_selfcheck();
        t.kwset_selfcheck();
        t
    }

    fn build_regex_kw_mask(&mut self) {
        // `of` is deliberately absent: it precedes a regex only in a for-of
        // head (never written — a RegExp isn't iterable), while `instance/of/g`
        // style division is real code. Matches es-module-lexer/SWC/RESS.
        const RX: [&str; 16] = [
            "in",
            "do",
            "new",
            "case",
            "void",
            "else",
            "yield",
            "await",
            "throw",
            "return",
            "typeof",
            "delete",
            "default",
            "extends",
            "debugger",
            "instanceof",
        ];
        // block, offsets < 64), so the mask is set-independent.
        let mut mask = 0u64;
        for r in RX.iter() {
            let mut found: i32 = -1;
            for kw in KEYWORDS.iter() {
                if kw.0 == *r {
                    found = (kw.1 - KW_KIND_BASE) as i32;
                    break;
                }
            }
            assert!(found >= 0 && found < 64, "tables.rs: regex-kw {r} missing from KEYWORDS");
            mask |= 1u64 << found;
        }
        self.regex_kw_mask = mask;
    }

    /// Is the word at `p` one of the keywords a regex may directly follow?
    /// Text-based and called only before any keyword-kind rewrite, so the JS
    /// set answers for both modes (every RX word is in both sets, and no
    /// other spelling has its mask bit).
    #[inline(always)]
    pub unsafe fn is_regex_keyword(&self, p: *const u8, len: usize) -> bool {
        let k = self.kwjs.lookup(p, len);
        k >= KW_KIND_BASE as u32 && ((self.regex_kw_mask >> (k - KW_KIND_BASE as u32)) & 1) != 0
    }

    fn build_op_pack(&mut self) {
        self.op2_pack = [0; 256];
        self.op3_pack = [0; 256];
        for i in 0..OPMAP_NOPS {
            let o = &OPMAP_OPS[i];
            let c2 = if o.len >= 3 { o.txt[2] } else { 0 };
            let key = op_key(o.txt[0], o.txt[1], c2, o.len as u32);
            let h = (key.wrapping_mul(self.op.opmap_mul) >> 24) as usize;
            if o.len == 2 {
                self.op2_pack[h] = 2u32
                    | ((o.txt[0] as u32) << 8)
                    | ((o.txt[1] as u32) << 16)
                    | ((o.kind as u32) << 24);
            } else if o.len == 3 {
                self.op3_pack[h] = 3u64
                    | ((o.txt[0] as u64) << 8)
                    | ((o.txt[1] as u64) << 16)
                    | ((o.txt[2] as u64) << 24)
                    | ((o.kind as u64) << 32);
            }
        }
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

    fn build_pair_luts(&mut self) {
        for m in 0..256usize {
            let mut k = 0usize;
            for bit in 0..8usize {
                if (m >> bit) & 1 != 0 {
                    self.pair_luts.lut0z[m][k] = bit as u8;
                    self.pair_luts.lutpad[m][8 + k] = (bit + 8) as u8;
                    k += 1;
                }
            }
            for j in k..8 {
                self.pair_luts.lutpad[m][8 + j] = 0x80;
            }
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

    /// Cross-set behavior the unit tests rely on: TS spellings resolve only
    /// through the TS set, and JS words agree byte-for-byte across sets.
    fn kwset_selfcheck(&self) {
        let mut buf = [0u8; 16];
        for (w, tok) in KEYWORDS_TS.iter() {
            let bytes = w.as_bytes();
            buf.fill(0);
            buf[..bytes.len()].copy_from_slice(bytes);
            let js = unsafe { self.kwjs.lookup(buf.as_ptr(), bytes.len()) };
            let ts = unsafe { self.kwts.lookup(buf.as_ptr(), bytes.len()) };
            assert!(ts == *tok as u32, "tables.rs: kwts lookup({w}) wrong");
            let in_js = KEYWORDS.iter().any(|k| k.0 == *w);
            assert!(js == if in_js { *tok as u32 } else { 0 }, "tables.rs: kwjs lookup({w}) wrong");
        }
    }
}

fn kwinit_selfcheck() {
    let mut in_set = [false; 256];
    for kw in KEYWORDS.iter() {
        in_set[kw.0.as_bytes()[0] as usize] = true;
    }
    for c in 0..256usize {
        assert!(is_kw_init(c as u8) == in_set[c], "tables.rs: KWINIT_LO/HI wrong at byte {c:#04x}");
    }
    let mut in_set_ts = [false; 256];
    for kw in KEYWORDS_TS.iter() {
        in_set_ts[kw.0.as_bytes()[0] as usize] = true;
    }
    for c in 0..256usize {
        assert!(
            is_kw_init_ts(c as u8) == in_set_ts[c],
            "tables.rs: KWINIT_TS_LO wrong at byte {c:#04x}"
        );
    }
}

fn opch_selfcheck() {
    const OPCHARS: &[u8] = b"=!<>+-*&|^%?.";
    let mut in_set = [false; 256];
    for &q in OPCHARS {
        in_set[q as usize] = true;
    }
    for c in 0..256usize {
        assert!(is_op_char(c as u8) == in_set[c], "tables.rs: OPCH_LO/HI wrong at byte {c:#04x}");
    }
}

fn punct1_hash_selfcheck() {
    let mut punct1_ord = [PUNCT1_KIND_UNKNOWN; 256];
    for i in 0..PUNCT1_NKNOWN {
        punct1_ord[PUNCT1_LIST[i] as usize] = PUNCT1_TOK[i];
    }
    for c in 0..256usize {
        let cb = c as u8;
        if is_word(cb) || is_ws(cb) {
            continue;
        }
        assert!(punct1_hash(cb) == punct1_ord[c], "tables.rs: PH_A/B/T wrong at byte {c:#04x}");
    }
}
