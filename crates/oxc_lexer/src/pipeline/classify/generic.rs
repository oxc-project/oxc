use crate::{
    opmap::{PUNCT1, PUNCT1_KIND_UNKNOWN, PUNCT1_NKNOWN},
    tables::{Tables, is_digit, is_kw_init, is_kw_init_ts, is_op_char, is_word, is_ws},
};

use super::super::{IDENT, NUM, WS};

const FL_WORD: u32 = 0;
const FL_WS: u32 = 1;
const FL_DIGIT: u32 = 2;
const FL_DOT: u32 = 3;
const FL_OPCH: u32 = 4;
const FL_KWINIT: u32 = 5;
const FL_MISC: u32 = 6;

const fn cls_table(ts: bool) -> [u16; 256] {
    let mut punct = [PUNCT1_KIND_UNKNOWN; 256];
    let mut i = 0;
    while i < PUNCT1_NKNOWN {
        punct[PUNCT1[i].0 as usize] = PUNCT1[i].1 as u8;
        i += 1;
    }
    let mut t = [0u16; 256];
    let mut c = 0usize;
    while c < 256 {
        let b = c as u8;
        let mut f = 0u16;
        if is_word(b) {
            f |= 1 << FL_WORD;
        }
        if is_ws(b) {
            f |= 1 << FL_WS;
        }
        if is_digit(b) {
            f |= 1 << FL_DIGIT;
        }
        if b == b'.' {
            f |= 1 << FL_DOT;
        }
        if is_op_char(b) {
            f |= 1 << FL_OPCH;
        }
        if (ts && is_kw_init_ts(b)) || (!ts && is_kw_init(b)) {
            f |= 1 << FL_KWINIT;
        }
        if b == b'#' || b == b'\\' || b >= 0x80 {
            f |= 1 << FL_MISC;
        }
        let kd = if is_ws(b) {
            WS
        } else if is_word(b) {
            if is_digit(b) { NUM } else { IDENT }
        } else {
            punct[c]
        };
        t[c] = (f << 8) | kd as u16;
        c += 1;
    }
    t
}

static CLS_JS: [u16; 256] = cls_table(false);
static CLS_TS: [u16; 256] = cls_table(true);

pub unsafe fn classify(
    _t: &Tables,
    ts: bool,
    src: *const u8,
    n: usize,
    word: *mut u64,
    st: *mut u64,
    kwinit: *mut u64,
    opch: *mut u64,
    digit: *mut u64,
    dot: *mut u64,
    misc: *mut u64,
    kind: *mut u8,
) {
    let t16: &[u16; 256] = if ts { &CLS_TS } else { &CLS_JS };
    let mut cw: u64 = 0;
    let mut cs: u64 = 0;
    let nbf = n / 64;
    for b in 0..nbf {
        let base = b * 64;
        let (mut mw, mut ms, mut mdg, mut mdt, mut mop, mut mkw, mut mmi) =
            (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
        let mut g = 0usize;
        while g < 8 {
            let p = base + g * 8;
            let mut fw = 0u64;
            let mut kw = 0u64;
            let mut j = 0usize;
            while j < 8 {
                let v = t16[*src.add(p + j) as usize] as u64;
                fw |= (v >> 8) << (j * 8);
                kw |= (v & 0xff) << (j * 8);
                j += 1;
            }
            core::ptr::write_unaligned(kind.add(p) as *mut u64, kw);
            let sh = (g * 8) as u32;
            mw |= pk(fw, FL_WORD) << sh;
            ms |= pk(fw, FL_WS) << sh;
            mdg |= pk(fw, FL_DIGIT) << sh;
            mdt |= pk(fw, FL_DOT) << sh;
            mop |= pk(fw, FL_OPCH) << sh;
            mkw |= pk(fw, FL_KWINIT) << sh;
            mmi |= pk(fw, FL_MISC) << sh;
            g += 1;
        }
        let wprev = (mw << 1) | cw;
        let sprev = (ms << 1) | cs;
        *st.add(b) = (ms & !sprev) | (mw & !wprev) | (!ms & !mw);
        cw = mw >> 63;
        cs = ms >> 63;
        *word.add(b) = mw;
        *kwinit.add(b) = mkw;
        *opch.add(b) = mop;
        *digit.add(b) = mdg;
        *dot.add(b) = mdt;
        *misc.add(b) = mmi;
    }
    let i = nbf * 64;
    if i < n {
        let b = nbf;
        let tail = n - i;
        let (mut w, mut stw, mut kwi, mut opw, mut dgw, mut dtw, mut msw) =
            (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
        let mut prev_word = cw & 1;
        let mut prev_ws = cs & 1;
        for kk in 0..tail {
            let v = t16[*src.add(i + kk) as usize];
            let f = (v >> 8) as u64;
            w |= ((f >> FL_WORD) & 1) << kk;
            kwi |= ((f >> FL_KWINIT) & 1) << kk;
            opw |= ((f >> FL_OPCH) & 1) << kk;
            dgw |= ((f >> FL_DIGIT) & 1) << kk;
            dtw |= ((f >> FL_DOT) & 1) << kk;
            msw |= ((f >> FL_MISC) & 1) << kk;
            let isw = (f >> FL_WORD) & 1;
            let isws = (f >> FL_WS) & 1;
            let start = (isws != 0 && prev_ws == 0)
                || (isw != 0 && prev_word == 0)
                || (isws == 0 && isw == 0);
            if start {
                stw |= 1u64 << kk;
            }
            *kind.add(i + kk) = (v & 0xff) as u8;
            prev_word = isw;
            prev_ws = isws;
        }
        *word.add(b) = w;
        *st.add(b) = stw;
        *kwinit.add(b) = kwi;
        *opch.add(b) = opw;
        *digit.add(b) = dgw;
        *dot.add(b) = dtw;
        *misc.add(b) = msw;
    }
}

#[inline(always)]
fn pk(fw: u64, bit: u32) -> u64 {
    (((fw >> bit) & 0x0101_0101_0101_0101).wrapping_mul(0x0102_0408_1020_4080)) >> 56
}
