use core::arch::x86_64::*;

use crate::{
    opmap::PUNCT1_KIND_UNKNOWN,
    tables::{PH_A, PH_B, PH_T0, PH_T1, Tables},
};

use super::super::{
    IDENT, NUM, WS,
    find::{load256, mm, veq},
};

pub unsafe fn classify(
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
    // Process ceil(n/64) blocks. When n is not a multiple of 64 the final
    // block overreads up to 63 bytes into the caller-guaranteed zeroed PAD
    // and is masked below — this replaces the byte-at-a-time scalar tail,
    // which cost ~18 cyc per tail byte (up to ~1.1k cyc when n mod 64 is
    // near 63) and dominated small-file lexing.
    let nb_ceil = n.div_ceil(64);
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
    while b < nb_ceil {
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
    // Mask the overread positions [n, ceil) out of the last block's bitmaps.
    // Zero PAD bytes classify as `st`=1 (non-word, non-ws => token start),
    // which would otherwise mint spurious tokens past `n` in `compress`; the
    // other six bitmaps are already 0 for a zero byte, but masking all seven
    // makes the last word bit-identical to the old scalar tail's output (real
    // bits [0, rem), zeros above) regardless of LUT contents. `kind` past `n`
    // is never read — `compress` only visits masked `st` starts — so it needs
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
