use std::arch::aarch64::*;

use crate::token::tk;

use crate::pipeline::cpu::Neon;

use super::{
    luts::{MERGED_LUTS, WORD_LUTS},
    punct1_luts::{PH_A, PH_B, PH_T0, PH_T1},
};

struct Luts {
    wb_lo: uint8x16_t,
    wb_hi: uint8x16_t,
    mrg_lo: uint8x16_t,
    mrg_hi: uint8x16_t,
    ph_a: uint8x16_t,
    ph_b: uint8x16_t,
    ph_t: uint8x16x2_t,
}

/// One chunk's lanes, each 0xFF or 0x00 as mm64 requires, plus its kind bytes.
struct Classified {
    word: uint8x16_t,
    ws: uint8x16_t,
    digit: uint8x16_t,
    dot: uint8x16_t,
    kwinit: uint8x16_t,
    opch: uint8x16_t,
    misc: uint8x16_t,
    kind: uint8x16_t,
}

#[inline(never)]
#[target_feature(enable = "neon")]
pub(super) unsafe fn classify_neon(
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
    cpu: Neon,
) {
    let mrg_lo = if ts { MERGED_LUTS.lo_ts } else { MERGED_LUTS.lo };
    let l = Luts {
        wb_lo: lut(&WORD_LUTS.lo, cpu),
        wb_hi: lut(&WORD_LUTS.hi, cpu),
        mrg_lo: lut(&mrg_lo, cpu),
        mrg_hi: lut(&MERGED_LUTS.hi, cpu),
        ph_a: lut(&PH_A, cpu),
        ph_b: lut(&PH_B, cpu),
        ph_t: uint8x16x2_t(lut(&PH_T0, cpu), lut(&PH_T1, cpu)),
    };

    let mut cw: u64 = 0;
    let mut cs: u64 = 0;
    let mut i = 0usize;
    let mut b = 0usize;
    let nb_ceil = n.div_ceil(64);
    while b < nb_ceil {
        // The last block overreads up to 63 bytes into the caller-guaranteed zeroed PAD.
        let c0 = chunk(vld1q_u8(src.add(i)), &l, cpu);
        let c1 = chunk(vld1q_u8(src.add(i + 16)), &l, cpu);
        let c2 = chunk(vld1q_u8(src.add(i + 32)), &l, cpu);
        let c3 = chunk(vld1q_u8(src.add(i + 48)), &l, cpu);

        vst1q_u8(kind.add(i), c0.kind);
        vst1q_u8(kind.add(i + 16), c1.kind);
        vst1q_u8(kind.add(i + 32), c2.kind);
        vst1q_u8(kind.add(i + 48), c3.kind);

        let wordm = mm64(c0.word, c1.word, c2.word, c3.word, cpu);
        let wsm = mm64(c0.ws, c1.ws, c2.ws, c3.ws, cpu);
        *word.add(b) = wordm;
        *digit.add(b) = mm64(c0.digit, c1.digit, c2.digit, c3.digit, cpu);
        *dot.add(b) = mm64(c0.dot, c1.dot, c2.dot, c3.dot, cpu);
        *kwinit.add(b) = mm64(c0.kwinit, c1.kwinit, c2.kwinit, c3.kwinit, cpu);
        *opch.add(b) = mm64(c0.opch, c1.opch, c2.opch, c3.opch, cpu);
        *misc.add(b) = mm64(c0.misc, c1.misc, c2.misc, c3.misc, cpu);

        let wprev = (wordm << 1) | cw;
        let sprev = (wsm << 1) | cs;
        cw = wordm >> 63;
        cs = wsm >> 63;
        *st.add(b) = (wsm & !sprev) | (wordm & !wprev) | (!wsm & !wordm);

        i += 64;
        b += 1;
    }
    // Zero PAD bytes classify as a token start, which would mint spurious tokens past n.
    let rem = n & 63;
    if rem != 0 {
        let last = nb_ceil - 1;
        let m = (1u64 << rem) - 1;
        *word.add(last) &= m;
        *st.add(last) &= m;
        *kwinit.add(last) &= m;
        *opch.add(last) &= m;
        *digit.add(last) &= m;
        *dot.add(last) &= m;
        *misc.add(last) &= m;
    }
}

#[inline(always)]
fn chunk(v: uint8x16_t, l: &Luts, cpu: Neon) -> Classified {
    // SAFETY: `Neon` type guarantees required target features are present
    unsafe {
        // Mask keeps the index in TBL range; hi has no 8-15 entry, so bytes >= 0x80 give zero.
        let lo = vandq_u8(v, vdupq_n_u8(0x0f));
        let hn = vshrq_n_u8::<4>(v);
        let tb = vandq_u8(vqtbl1q_u8(l.wb_lo, lo), vqtbl1q_u8(l.wb_hi, hn));
        let t = vandq_u8(vqtbl1q_u8(l.mrg_lo, lo), vqtbl1q_u8(l.mrg_hi, hn));

        let na = vtstq_u8(v, vdupq_n_u8(0x80));
        let word = vorrq_u8(vtstq_u8(tb, vdupq_n_u8(0x3f)), na);
        let ws = vtstq_u8(tb, vdupq_n_u8(0xc0));
        let digit = vtstq_u8(tb, vdupq_n_u8(0x02));
        let misc = vorrq_u8(vorrq_u8(veq(v, b'#', cpu), veq(v, b'\\', cpu)), na);

        let mut kind = punct1(v, lo, hn, l, cpu);
        kind = vbslq_u8(ws, vdupq_n_u8(tk!(Whitespace)), kind);
        kind = vbslq_u8(word, vdupq_n_u8(tk!(Ident)), kind);
        kind = vbslq_u8(digit, vdupq_n_u8(tk!(Number)), kind);

        Classified {
            word,
            ws,
            digit,
            dot: vtstq_u8(t, vdupq_n_u8(0x80)),
            kwinit: vtstq_u8(t, vdupq_n_u8(0x03)),
            opch: vtstq_u8(t, vdupq_n_u8(0x3c)),
            misc,
            kind,
        }
    }
}

#[inline(always)]
fn punct1(v: uint8x16_t, lo: uint8x16_t, hn: uint8x16_t, l: &Luts, _cpu: Neon) -> uint8x16_t {
    // SAFETY: `Neon` type guarantees required target features are present
    unsafe {
        // Every PH_A and PH_B entry is under 32, so h is in range for the 32-entry ph_t.
        let h = veorq_u8(vqtbl1q_u8(l.ph_a, lo), vqtbl1q_u8(l.ph_b, hn));
        let ord = vqtbl2q_u8(l.ph_t, h);
        let ctl = vcltq_s8(vreinterpretq_s8_u8(v), vdupq_n_s8(0x20));
        vbslq_u8(ctl, vdupq_n_u8(tk!(Invalid)), ord)
    }
}

#[inline(always)]
fn veq(v: uint8x16_t, c: u8, _cpu: Neon) -> uint8x16_t {
    // SAFETY: `Neon` type guarantees required target features are present
    unsafe { vceqq_u8(v, vdupq_n_u8(c)) }
}

#[inline(always)]
fn lut(t: &[u8; 16], _cpu: Neon) -> uint8x16_t {
    // SAFETY: `Neon` type guarantees required target features are present.
    // `t` is a `&[u8; 16]`, so is valid for reading 16 bytes.
    unsafe { vld1q_u8(t.as_ptr()) }
}

/// Bit i is the top bit of byte i of a-d, gathered by summing the lane bit values.
#[inline(always)]
fn mm64(a: uint8x16_t, b: uint8x16_t, c: uint8x16_t, d: uint8x16_t, cpu: Neon) -> u64 {
    const BITS: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];
    let m = lut(&BITS, cpu);
    // SAFETY: `Neon` type guarantees required target features are present
    unsafe {
        let ab = vpaddq_u8(vandq_u8(a, m), vandq_u8(b, m));
        let cd = vpaddq_u8(vandq_u8(c, m), vandq_u8(d, m));
        let s = vpaddq_u8(ab, cd);
        vgetq_lane_u64(vreinterpretq_u64_u8(vpaddq_u8(s, s)), 0)
    }
}
