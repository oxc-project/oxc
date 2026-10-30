use crate::{
    cpu::Avx2,
    simd::avx2::{Vec128, Vec256},
    token::tk,
};

use super::{
    luts::{DIGIT_MASK, KWINIT_MASK, MERGED_LUTS, OPCH_MASK, WORD_LUTS, WORD_MASK, WS_MASK},
    punct1_luts::{PH_A, PH_B, PH_T},
};

#[inline(never)]
#[target_feature(enable = "avx2,bmi2,popcnt")]
pub(super) unsafe fn classify_avx2(
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
    cpu: Avx2,
) {
    // The merged LUT variants differ only in the keyword-initial bits; the
    // selection happens once, outside the loop.
    let mrg_lo = if ts { MERGED_LUTS.lo_ts } else { MERGED_LUTS.lo };
    let mut cw: u64 = 0;
    let mut cs: u64 = 0;
    let mut i = 0usize;
    let mut b = 0usize;
    // Process ceil(n/64) blocks. When n is not a multiple of 64 the final
    // block overreads up to 63 bytes into the caller-guaranteed zeroed PAD
    // and is masked below - this replaces the byte-at-a-time scalar tail,
    // which cost ~18 cyc per tail byte (up to ~1.1k cyc when n mod 64 is
    // near 63) and dominated small-file lexing.
    let nb_ceil = n.div_ceil(64);
    let v_pha = Vec256::splat_v128(Vec128::read_unaligned(&PH_A, cpu));
    let v_phb = Vec256::splat_v128(Vec128::read_unaligned(&PH_B, cpu));
    let (pht, _) = PH_T.as_chunks::<16>();
    let v_pht0 = Vec256::splat_v128(Vec128::read_unaligned(&pht[0], cpu));
    let v_pht1 = Vec256::splat_v128(Vec128::read_unaligned(&pht[1], cpu));
    let v_mlo = Vec256::splat_v128(Vec128::read_unaligned(&mrg_lo, cpu));
    let v_mhi = Vec256::splat_v128(Vec128::read_unaligned(&MERGED_LUTS.hi, cpu));
    let v_wblo = Vec256::splat_v128(Vec128::read_unaligned(&WORD_LUTS.lo, cpu));
    let v_wbhi = Vec256::splat_v128(Vec128::read_unaligned(&WORD_LUTS.hi, cpu));
    while b < nb_ceil {
        let v0 = Vec256::load_unaligned(src.add(i), cpu);
        let v1 = Vec256::load_unaligned(src.add(i + 32), cpu);
        let hn0 = v0.shr_u16::<4>() & 0x0f;
        let hn1 = v1.shr_u16::<4>() & 0x0f;
        let tb0 = v_wblo.shuffle_u8s(v0) & v_wbhi.shuffle_u8s(hn0);
        let tb1 = v_wblo.shuffle_u8s(v1) & v_wbhi.shuffle_u8s(hn1);
        // Non-ASCII bytes count as identifier chars in `word` and are folded
        // into `misc` so `misc_pre` can re-classify Unicode whitespace.
        let na0 = v0.lt_i8(0);
        let na1 = v1.lt_i8(0);
        let w0 = (tb0 & WORD_MASK).gt_i8(0) | na0;
        let w1 = (tb1 & WORD_MASK).gt_i8(0) | na1;
        let nws0 = (tb0 & WS_MASK).eq_u8(0);
        let nws1 = (tb1 & WS_MASK).eq_u8(0);
        let d0 = (tb0 & DIGIT_MASK).gt_i8(0);
        let d1 = (tb1 & DIGIT_MASK).gt_i8(0);
        let wordm = (w0.movemask() as u64) | ((w1.movemask() as u64) << 32);
        let wsm = ((!nws0.movemask()) as u64) | (((!nws1.movemask()) as u64) << 32);
        *word.add(b) = wordm;
        *digit.add(b) = (d0.movemask() as u64) | ((d1.movemask() as u64) << 32);
        let misc0 = (v0.eq_u8(b'#') | v0.eq_u8(b'\\') | na0).movemask() as u64;
        let misc1 = (v1.eq_u8(b'#') | v1.eq_u8(b'\\') | na1).movemask() as u64;
        *misc.add(b) = misc0 | (misc1 << 32);
        let t0 = v_mlo.shuffle_u8s(v0) & v_mhi.shuffle_u8s(hn0);
        let t1 = v_mlo.shuffle_u8s(v1) & v_mhi.shuffle_u8s(hn1);
        *dot.add(b) = (t0.movemask() as u64) | ((t1.movemask() as u64) << 32);
        let kw0 = (t0 & KWINIT_MASK).gt_i8(0).movemask() as u64;
        let kw1 = (t1 & KWINIT_MASK).gt_i8(0).movemask() as u64;
        *kwinit.add(b) = kw0 | (kw1 << 32);
        let op0 = (t0 & OPCH_MASK).gt_i8(0).movemask() as u64;
        let op1 = (t1 & OPCH_MASK).gt_i8(0).movemask() as u64;
        *opch.add(b) = op0 | (op1 << 32);
        let wprev = (wordm << 1) | cw;
        let sprev = (wsm << 1) | cs;
        cw = wordm >> 63;
        cs = wsm >> 63;
        *st.add(b) = (wsm & !sprev) | (wordm & !wprev) | (!wsm & !wordm);
        let k0 = vpunct1(v0, hn0, v_pha, v_phb, v_pht0, v_pht1)
            .blend(tk!(Whitespace), !nws0)
            .blend(tk!(Ident), w0)
            .blend(tk!(Number), d0);
        let k1 = vpunct1(v1, hn1, v_pha, v_phb, v_pht0, v_pht1)
            .blend(tk!(Whitespace), !nws1)
            .blend(tk!(Ident), w1)
            .blend(tk!(Number), d1);
        k0.store_unaligned(kind.add(b * 64));
        k1.store_unaligned(kind.add(b * 64 + 32));
        i += 64;
        b += 1;
    }
    // Mask the overread positions [n, ceil) out of the last block's bitmaps.
    // Zero PAD bytes classify as `st`=1 (non-word, non-ws => token start),
    // which would otherwise mint spurious tokens past `n` in `compress`; the
    // other six bitmaps are already 0 for a zero byte, but masking all seven
    // makes the last word bit-identical to the old scalar tail's output (real
    // bits [0, rem), zeros above) regardless of LUT contents. `kind` past `n`
    // is never read - `compress` only visits masked `st` starts - so it needs
    // no fixup.
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
fn vpunct1(v: Vec256, hn: Vec256, pa: Vec256, pb: Vec256, pt0: Vec256, pt1: Vec256) -> Vec256 {
    let h = pa.shuffle_u8s(v) ^ pb.shuffle_u8s(hn);
    let o0 = pt0.shuffle_u8s(h);
    let o1 = pt1.shuffle_u8s(h);
    let ord = o0.blend(o1, h.shl_u16::<3>());
    let ctl = v.lt_i8(0x20);
    ord.blend(tk!(Invalid), ctl)
}
