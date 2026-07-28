#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use core::arch::x86_64::*;

use crate::error::diag_code;
use crate::lanes::Lanes;
use crate::opmap::{KW_KIND_BASE, PUNCT1_KIND_UNKNOWN};
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
use crate::opmap::{PUNCT1, PUNCT1_NKNOWN};
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use crate::tables::{PH_A, PH_B, PH_T0, PH_T1};
use crate::tables::{Tables, is_digit, is_id_start, is_op_char, is_word};
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
use crate::tables::{is_kw_init, is_kw_init_ts, is_ws};

use super::bitmap::{bm_clear_range, bm_next0, bm_prev1, bm_set1};
use super::find::scan_ident_esc;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use super::find::{load256, mm, veq};
use super::{IDENT, IDENT_ESC, NUM, PRIV_IDENT, PRIV_IDENT_ESC, WS};

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline(always)]
unsafe fn vpunct1(
    v: __m256i,
    hn: __m256i,
    pa: __m256i,
    pb: __m256i,
    pt0: __m256i,
    pt1: __m256i,
    v96: __m256i,
) -> __m256i {
    let h = _mm256_xor_si256(_mm256_shuffle_epi8(pa, v), _mm256_shuffle_epi8(pb, hn));
    let o0 = _mm256_shuffle_epi8(pt0, h);
    let o1 = _mm256_shuffle_epi8(pt1, h);
    let ord = _mm256_blendv_epi8(o0, o1, _mm256_slli_epi16::<3>(h));
    let ctl = _mm256_cmpgt_epi8(_mm256_set1_epi8(0x20), v);
    _mm256_blendv_epi8(ord, v96, ctl)
}
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
pub(super) unsafe fn classify(
    t: &Tables,
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
    // The merged LUT variants differ only in the keyword-initial bits; the
    // selection happens once, outside the loop.
    let mrg_lo = if ts { &t.mrg_lo_ts } else { &t.mrg_lo };
    let mut cw: u64 = 0;
    let mut cs: u64 = 0;
    let mut i = 0usize;
    let mut b = 0usize;
    let v_pha = _mm256_broadcastsi128_si256(_mm_loadu_si128(PH_A.as_ptr() as *const __m128i));
    let v_phb = _mm256_broadcastsi128_si256(_mm_loadu_si128(PH_B.as_ptr() as *const __m128i));
    let v_pht0 = _mm256_broadcastsi128_si256(_mm_loadu_si128(PH_T0.as_ptr() as *const __m128i));
    let v_pht1 = _mm256_broadcastsi128_si256(_mm_loadu_si128(PH_T1.as_ptr() as *const __m128i));
    let v_96 = _mm256_set1_epi8(PUNCT1_KIND_UNKNOWN as i8);
    let v_ws = _mm256_set1_epi8(WS as i8);
    let v_ident = _mm256_set1_epi8(IDENT as i8);
    let v_num = _mm256_set1_epi8(NUM as i8);
    let v_mlo = _mm256_broadcastsi128_si256(_mm_loadu_si128(mrg_lo.as_ptr() as *const __m128i));
    let v_mhi = _mm256_broadcastsi128_si256(_mm_loadu_si128(t.mrg_hi.as_ptr() as *const __m128i));
    let v_wblo = _mm256_broadcastsi128_si256(_mm_loadu_si128(t.wb_lo.as_ptr() as *const __m128i));
    let v_wbhi = _mm256_broadcastsi128_si256(_mm_loadu_si128(t.wb_hi.as_ptr() as *const __m128i));
    let v_kwpl = _mm256_set1_epi8(0x03);
    let v_oppl = _mm256_set1_epi8(0x3c);
    let v_wdpl = _mm256_set1_epi8(0x3f);
    let v_wspl = _mm256_set1_epi8(0xc0u8 as i8);
    let v_dgpl = _mm256_set1_epi8(0x02);
    let v_ones = _mm256_set1_epi8(0xffu8 as i8);
    let v_zero = _mm256_setzero_si256();
    let v_x0f = _mm256_set1_epi8(0x0f);
    while i + 64 <= n {
        let v0 = load256(src, i);
        let v1 = load256(src, i + 32);
        let hn0 = _mm256_and_si256(_mm256_srli_epi16::<4>(v0), v_x0f);
        let hn1 = _mm256_and_si256(_mm256_srli_epi16::<4>(v1), v_x0f);
        let tb0 =
            _mm256_and_si256(_mm256_shuffle_epi8(v_wblo, v0), _mm256_shuffle_epi8(v_wbhi, hn0));
        let tb1 =
            _mm256_and_si256(_mm256_shuffle_epi8(v_wblo, v1), _mm256_shuffle_epi8(v_wbhi, hn1));
        let nw0 = _mm256_cmpeq_epi8(_mm256_and_si256(tb0, v_wdpl), v_zero);
        let nw1 = _mm256_cmpeq_epi8(_mm256_and_si256(tb1, v_wdpl), v_zero);
        // Non-ASCII bytes count as identifier chars in `word` and are folded
        // into `misc` so `misc_pre` can re-classify Unicode whitespace.
        let na0 = _mm256_cmpgt_epi8(v_zero, v0);
        let na1 = _mm256_cmpgt_epi8(v_zero, v1);
        let w0 = _mm256_or_si256(_mm256_xor_si256(nw0, v_ones), na0);
        let w1 = _mm256_or_si256(_mm256_xor_si256(nw1, v_ones), na1);
        let nws0 = _mm256_cmpeq_epi8(_mm256_and_si256(tb0, v_wspl), v_zero);
        let nws1 = _mm256_cmpeq_epi8(_mm256_and_si256(tb1, v_wspl), v_zero);
        let d0 = _mm256_cmpgt_epi8(_mm256_and_si256(tb0, v_dgpl), v_zero);
        let d1 = _mm256_cmpgt_epi8(_mm256_and_si256(tb1, v_dgpl), v_zero);
        let wordm = (mm(w0) as u64) | ((mm(w1) as u64) << 32);
        let wsm = ((!mm(nws0)) as u64) | (((!mm(nws1)) as u64) << 32);
        *word.add(b) = wordm;
        *digit.add(b) = (mm(d0) as u64) | ((mm(d1) as u64) << 32);
        *misc.add(b) = (mm(_mm256_or_si256(_mm256_or_si256(veq(v0, b'#'), veq(v0, b'\\')), na0))
            as u64)
            | ((mm(_mm256_or_si256(_mm256_or_si256(veq(v1, b'#'), veq(v1, b'\\')), na1)) as u64)
                << 32);
        let t0 = _mm256_and_si256(_mm256_shuffle_epi8(v_mlo, v0), _mm256_shuffle_epi8(v_mhi, hn0));
        let t1 = _mm256_and_si256(_mm256_shuffle_epi8(v_mlo, v1), _mm256_shuffle_epi8(v_mhi, hn1));
        *dot.add(b) = (mm(t0) as u64) | ((mm(t1) as u64) << 32);
        *kwinit.add(b) = (mm(_mm256_cmpgt_epi8(_mm256_and_si256(t0, v_kwpl), v_zero)) as u64)
            | ((mm(_mm256_cmpgt_epi8(_mm256_and_si256(t1, v_kwpl), v_zero)) as u64) << 32);
        *opch.add(b) = (mm(_mm256_cmpgt_epi8(_mm256_and_si256(t0, v_oppl), v_zero)) as u64)
            | ((mm(_mm256_cmpgt_epi8(_mm256_and_si256(t1, v_oppl), v_zero)) as u64) << 32);
        let wprev = (wordm << 1) | cw;
        let sprev = (wsm << 1) | cs;
        cw = wordm >> 63;
        cs = wsm >> 63;
        *st.add(b) = (wsm & !sprev) | (wordm & !wprev) | (!wsm & !wordm);
        let mut k0 = vpunct1(v0, hn0, v_pha, v_phb, v_pht0, v_pht1, v_96);
        k0 = _mm256_blendv_epi8(v_ws, k0, nws0);
        k0 = _mm256_blendv_epi8(k0, v_ident, w0);
        k0 = _mm256_blendv_epi8(k0, v_num, d0);
        let mut k1 = vpunct1(v1, hn1, v_pha, v_phb, v_pht0, v_pht1, v_96);
        k1 = _mm256_blendv_epi8(v_ws, k1, nws1);
        k1 = _mm256_blendv_epi8(k1, v_ident, w1);
        k1 = _mm256_blendv_epi8(k1, v_num, d1);
        _mm256_storeu_si256(kind.add(b * 64) as *mut __m256i, k0);
        _mm256_storeu_si256(kind.add(b * 64 + 32) as *mut __m256i, k1);
        i += 64;
        b += 1;
    }
    if i < n {
        let mut w: u64 = 0;
        let mut stw: u64 = 0;
        let mut kwi: u64 = 0;
        let mut opw: u64 = 0;
        let mut dgw: u64 = 0;
        let mut dtw: u64 = 0;
        let mut msw: u64 = 0;
        let mut prev_word = cw & 1;
        let mut prev_ws = cs & 1;
        let tail = n - i;
        for kk in 0..tail {
            let c = *src.add(i + kk);
            let isw = is_word(c);
            let isws = crate::tables::is_ws(c);
            if isw {
                w |= 1u64 << kk;
            }
            if if ts { crate::tables::is_kw_init_ts(c) } else { crate::tables::is_kw_init(c) } {
                kwi |= 1u64 << kk;
            }
            if is_op_char(c) {
                opw |= 1u64 << kk;
            }
            if is_digit(c) {
                dgw |= 1u64 << kk;
            }
            if c == b'.' {
                dtw |= 1u64 << kk;
            }
            if c == b'#' || c == b'\\' || c >= 0x80 {
                msw |= 1u64 << kk;
            }
            let start = (isws && prev_ws == 0) || (isw && prev_word == 0) || (!isws && !isw);
            if start {
                stw |= 1u64 << kk;
            }
            let kd = if isws {
                WS
            } else if isw {
                if is_digit(c) { NUM } else { IDENT }
            } else {
                t.op.punct1_ord[c as usize]
            };
            *kind.add(i + kk) = kd;
            prev_word = isw as u64;
            prev_ws = isws as u64;
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

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const FL_WORD: u32 = 0;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const FL_WS: u32 = 1;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const FL_DIGIT: u32 = 2;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const FL_DOT: u32 = 3;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const FL_OPCH: u32 = 4;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const FL_KWINIT: u32 = 5;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const FL_MISC: u32 = 6;

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
const fn cls_table(ts: bool) -> [u16; 256] {
    let mut punct = [PUNCT1_KIND_UNKNOWN; 256];
    let mut i = 0;
    while i < PUNCT1_NKNOWN {
        punct[PUNCT1[i].0 as usize] = PUNCT1[i].1;
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
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
static CLS_JS: [u16; 256] = cls_table(false);
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
static CLS_TS: [u16; 256] = cls_table(true);

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline(always)]
fn pk(fw: u64, bit: u32) -> u64 {
    (((fw >> bit) & 0x0101_0101_0101_0101).wrapping_mul(0x0102_0408_1020_4080)) >> 56
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub(super) unsafe fn classify(
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
/// Byte length (2 or 3) of the multi-byte ECMAScript WhiteSpace /
/// LineTerminator at `p`, or 0. The non-ASCII set: U+00A0, U+1680,
/// U+2000..=U+200A, U+2028, U+2029, U+202F, U+205F, U+3000, U+FEFF.
#[inline]
unsafe fn unicode_ws_len(src: *const u8, p: usize) -> usize {
    let c1 = *src.add(p + 1);
    match *src.add(p) {
        0xC2 => usize::from(c1 == 0xA0) * 2,
        0xE1 => usize::from(c1 == 0x9A && *src.add(p + 2) == 0x80) * 3,
        0xE2 => {
            let c2 = *src.add(p + 2);
            let is_ws = (c1 == 0x80
                && ((0x80..=0x8A).contains(&c2) || c2 == 0xA8 || c2 == 0xA9 || c2 == 0xAF))
                || (c1 == 0x81 && c2 == 0x9F);
            usize::from(is_ws) * 3
        }
        0xE3 => usize::from(c1 == 0x80 && *src.add(p + 2) == 0x80) * 3,
        0xEF => usize::from(c1 == 0xBB && *src.add(p + 2) == 0xBF) * 3,
        _ => 0,
    }
}
/// Validate the UTF-8 sequence led by the byte at `p` (>= 0x80). Returns
/// `(valid, cont)` where `cont` is the count of range-valid continuation
/// bytes after the lead. Second-byte ranges are per-lead, so overlongs,
/// surrogates, and out-of-range sequences report the same maximal subpart
/// as `std::str::from_utf8`'s `error_len`.
#[inline]
unsafe fn utf8_seq_check(src: *const u8, p: usize, n: usize) -> (bool, usize) {
    let (need, lo, hi): (usize, u8, u8) = match *src.add(p) {
        0xC2..=0xDF => (1, 0x80, 0xBF),
        0xE0 => (2, 0xA0, 0xBF),
        0xE1..=0xEC | 0xEE..=0xEF => (2, 0x80, 0xBF),
        0xED => (2, 0x80, 0x9F),
        0xF0 => (3, 0x90, 0xBF),
        0xF1..=0xF3 => (3, 0x80, 0xBF),
        0xF4 => (3, 0x80, 0x8F),
        // 0x80..=0xBF stray continuation, 0xC0/0xC1 overlong leads, 0xF5..=0xFF
        _ => return (false, 0),
    };
    let mut cont = 0usize;
    if p + 1 < n && *src.add(p + 1) >= lo && *src.add(p + 1) <= hi {
        cont = 1;
        while cont < need && p + 1 + cont < n {
            let b = *src.add(p + 1 + cont);
            if !(0x80..=0xBF).contains(&b) {
                break;
            }
            cont += 1;
        }
    }
    (cont == need, cont)
}
/// `VUTF8` (`LexOptions::validate_utf8`) adds UTF-8 well-formedness
/// validation to the non-ASCII walk. Monomorphized so the default `false`
/// copy pays nothing for it.
pub(super) unsafe fn misc_pre<const VUTF8: bool>(
    src: *const u8,
    n: usize,
    st: *mut u64,
    word: *mut u64,
    misc: *const u64,
    kind: *mut u8,
    lanes: &mut Lanes,
) -> usize {
    let mut nesc = 0usize;
    let nw = (n + 63) >> 6;
    // VUTF8 only: continuation bits spilling into the next word, and the
    // once-per-file latch for the UTF-8 diagnostic (parser parity; also
    // bounds the diag Vec on binary input).
    let mut skip_lo: u64 = 0;
    let mut utf8_bad = false;
    for w in 0..nw {
        let mut m = if VUTF8 { *misc.add(w) & !skip_lo } else { *misc.add(w) };
        if VUTF8 {
            skip_lo = 0;
        }
        while m != 0 {
            let bit = m.trailing_zeros() as usize;
            m &= m - 1;
            let p = (w << 6) + bit;
            let c = *src.add(p);
            // Multi-byte Unicode whitespace was bulk-classified as an
            // identifier char; re-mark it as a whitespace token boundary.
            // Continuation bytes and non-ws leads fall through cheaply.
            if c >= 0x80 {
                if VUTF8 {
                    let (ok, cont) = utf8_seq_check(src, p, n);
                    if ok {
                        // Consume the verified continuation bits so they are
                        // not re-visited; a continuation still visible to the
                        // walk had no valid lead — that is the stray-
                        // continuation check.
                        let cm: u128 = (((1u128 << cont) - 1) << 1) << (p & 63);
                        m &= !(cm as u64);
                        skip_lo |= (cm >> 64) as u64;
                    } else if !utf8_bad {
                        utf8_bad = true;
                        // Span = the maximal invalid subpart; one diag per
                        // file, context-free (fires inside strings too).
                        lanes.push_diag(p as u32, (1 + cont) as u32, diag_code::INVALID_UTF8);
                        continue;
                    } else {
                        continue;
                    }
                }
                let len = unicode_ws_len(src, p);
                if len != 0 {
                    *kind.add(p) = WS;
                    bm_clear_range(word, p, p + len - 1);
                    bm_set1(st, p);
                    bm_clear_range(st, p + 1, p + len - 1);
                    if p + len < n {
                        bm_set1(st, p + len);
                    }
                } else if (0xC2..=0xF4).contains(&c) {
                    // Non-whitespace lead: record the position only. The
                    // identifier-char check is deferred to drain, where leads
                    // inside literal tokens drop before ever paying for it.
                    lanes.unicode_leads.push(p as u32);
                }
                continue;
            }
            if (*st.add(p >> 6) >> (p & 63)) & 1 == 0 {
                continue;
            }
            if c == b'#' {
                if p == 0 && n > 1 && *src.add(1) == b'!' {
                    continue;
                }
                // Accept an id-start or a leading unicode escape, so a
                // private name whose first char is itself an escape forms
                // one token instead of `#` + escape.
                let c1 = *src.add(p + 1);
                if !(is_id_start(c1) || (c1 == b'\\' && *src.add(p + 2) == b'u')) {
                    continue;
                }
                let e0 = bm_next0(word, p + 1, n);
                let e = if *src.add(e0) == b'\\' && *src.add(e0 + 1) == b'u' {
                    *kind.add(p) = PRIV_IDENT_ESC;
                    scan_ident_esc(src, n, e0)
                } else {
                    *kind.add(p) = PRIV_IDENT;
                    e0
                };
                bm_clear_range(st, p + 1, e - 1);
            } else {
                if *src.add(p + 1) != b'u' {
                    continue;
                }
                *kind.add(p) = IDENT_ESC;
                nesc += 1;
                let e = scan_ident_esc(src, n, p);
                bm_clear_range(st, p + 1, e - 1);
            }
        }
    }
    nesc
}
pub(super) unsafe fn misc_post(
    src: *const u8,
    n: usize,
    st: *mut u64,
    word: *const u64,
    misc: *const u64,
    kind: *mut u8,
) {
    let nw = (n + 63) >> 6;
    for w in 0..nw {
        let mut m = *misc.add(w);
        while m != 0 {
            let bit = m.trailing_zeros() as usize;
            m &= m - 1;
            let p = (w << 6) + bit;
            if *src.add(p) != b'\\' || *src.add(p + 1) != b'u' {
                continue;
            }
            if (*st.add(p >> 6) >> (p & 63)) & 1 == 0 {
                continue;
            }
            if p == 0 || (*word.add((p - 1) >> 6) >> ((p - 1) & 63)) & 1 == 0 {
                continue;
            }
            let tt = bm_prev1(st, p);
            let k = *kind.add(tt as usize);
            if k == IDENT || (k >= KW_KIND_BASE && k != PUNCT1_KIND_UNKNOWN) {
                *kind.add(tt as usize) = IDENT_ESC;
                *st.add(p >> 6) &= !(1u64 << (p & 63));
            }
        }
    }
}
