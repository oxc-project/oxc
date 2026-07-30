#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use core::arch::x86_64::*;

use oxc_span::Span;

use crate::error::diag_code;
use crate::lanes::Lanes;
use crate::tables::Tables;
use crate::token::{SPAN_SENTINELS, is_trivia_byte};

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
use super::find::{eqm, load8};
use super::{BIGINT, EOF, HASHBANG, IDENT_ESC, NUM, PRIV_IDENT_ESC};

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
const fn qcompact() -> [[u32; 8]; 16] {
    let mut t = [[0u32; 8]; 16];
    let mut mask = 0usize;
    while mask < 16 {
        let (mut q, mut k) = (0usize, 0usize);
        while q < 4 {
            if mask & (1 << q) != 0 {
                t[mask][2 * k] = (2 * q) as u32;
                t[mask][2 * k + 1] = (2 * q + 1) as u32;
                k += 1;
            }
            q += 1;
        }
        mask += 1;
    }
    t
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
const fn bcompact() -> [[u8; 16]; 256] {
    let mut t = [[0x80u8; 16]; 256];
    let mut mask = 0usize;
    while mask < 256 {
        let (mut b, mut k) = (0usize, 0usize);
        while b < 8 {
            if mask & (1 << b) != 0 {
                t[mask][k] = b as u8;
                k += 1;
            }
            b += 1;
        }
        mask += 1;
    }
    t
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
static QCOMPACT: [[u32; 8]; 16] = qcompact();
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
static BCOMPACT: [[u8; 16]; 256] = bcompact();

pub(super) unsafe fn build_spans(
    stage_kind: *const u8,
    stage_pos: *const u32,
    m: usize,
    spans: *mut Span,
    sig_kinds: *mut u8,
) -> usize {
    let sp = spans.cast::<u64>();
    let mut w = 0usize;
    let mut j = 0usize;
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
    {
        let v_min = _mm_set1_epi8(crate::token::TRIVIA_MIN as i8);
        let v_span = _mm_set1_epi8((crate::token::TRIVIA_MAX - crate::token::TRIVIA_MIN) as i8);
        let v_hb = _mm_set1_epi8(HASHBANG as i8);
        let zero = _mm_setzero_si128();
        while j + 8 <= m {
            let k8 = _mm_loadl_epi64(stage_kind.add(j) as *const __m128i);
            let triv = _mm_cmpeq_epi8(_mm_subs_epu8(_mm_sub_epi8(k8, v_min), v_span), zero);
            let drop = _mm_andnot_si128(_mm_cmpeq_epi8(k8, v_hb), triv);
            let sig = (!(_mm_movemask_epi8(drop) as u32)) & 0xff;

            let v0 = _mm256_loadu_si256(stage_pos.add(j) as *const __m256i);
            let v1 = _mm256_loadu_si256(stage_pos.add(j + 1) as *const __m256i);
            let lo = _mm256_unpacklo_epi32(v0, v1);
            let hi = _mm256_unpackhi_epi32(v0, v1);
            let a = _mm256_permute2x128_si256(lo, hi, 0x20);
            let b = _mm256_permute2x128_si256(lo, hi, 0x31);

            let ma = (sig & 0xf) as usize;
            let mb = (sig >> 4) as usize;
            let ca = _mm256_permutevar8x32_epi32(
                a,
                _mm256_loadu_si256(QCOMPACT[ma].as_ptr() as *const __m256i),
            );
            _mm256_storeu_si256(sp.add(w) as *mut __m256i, ca);
            let cb = _mm256_permutevar8x32_epi32(
                b,
                _mm256_loadu_si256(QCOMPACT[mb].as_ptr() as *const __m256i),
            );
            _mm256_storeu_si256(sp.add(w + ma.count_ones() as usize) as *mut __m256i, cb);

            let kc = _mm_shuffle_epi8(
                k8,
                _mm_loadu_si128(BCOMPACT[sig as usize].as_ptr() as *const __m128i),
            );
            _mm_storel_epi64(sig_kinds.add(w) as *mut __m128i, kc);
            w += sig.count_ones() as usize;
            j += 8;
        }
    }
    while j < m {
        let k = *stage_kind.add(j);
        *sp.add(w) = stage_pos.add(j).cast::<u64>().read_unaligned();
        *sig_kinds.add(w) = k;
        w += usize::from(!is_trivia_byte(k) || k == HASHBANG);
        j += 1;
    }
    w
}

pub(super) unsafe fn write_sentinels(n: u32, spans: *mut Span, sig_kinds: *mut u8) {
    let eof = u64::from(n) | (u64::from(n) << 32);
    for s in 0..SPAN_SENTINELS {
        *spans.cast::<u64>().add(s) = eof;
        *sig_kinds.add(s) = EOF;
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline(never)]
pub(super) unsafe fn compress(
    t: &Tables,
    st: *const u64,
    kind: *const u8,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    kinds: *mut u8,
) -> usize {
    let lut0z = t.pair_luts.lut0z.as_ptr().cast::<u8>();
    let lutpad = t.pair_luts.lutpad.as_ptr().cast::<u8>();
    let step16 = _mm256_set1_epi32(16);
    let mut m = 0usize;
    for b in b0..b1 {
        let mword = *st.add(b);
        if mword == 0 {
            continue;
        }
        let base = b * 64;
        let o0: u32 = 0;
        let o1 = o0 + (mword & 0xffff).count_ones();
        let o2 = o1 + ((mword >> 16) & 0xffff).count_ones();
        let o3 = o2 + ((mword >> 32) & 0xffff).count_ones();
        let o4 = o3 + (mword >> 48).count_ones();
        let mut basev = _mm256_set1_epi32(base as i32);
        macro_rules! pair {
            ($p:expr, $off:expr) => {{
                let sub0 = ((mword >> (16 * $p)) & 0xff) as usize;
                let sub1 = ((mword >> (16 * $p + 8)) & 0xff) as usize;
                let pc0 = sub0.count_ones() as usize;
                let row0 = _mm_loadl_epi64(lut0z.add(sub0 * 8) as *const __m128i);
                let row1 = _mm_loadu_si128(lutpad.add(sub1 * 32 + 8 - pc0) as *const __m128i);
                let ctrl = _mm_or_si128(row0, row1);
                let kw = _mm_loadu_si128(kind.add(base + 16 * $p) as *const __m128i);
                let lo = _mm256_add_epi32(basev, _mm256_cvtepu8_epi32(ctrl));
                _mm256_storeu_si256(starts.add(m + $off as usize) as *mut __m256i, lo);
                let hi = _mm256_add_epi32(basev, _mm256_cvtepu8_epi32(_mm_srli_si128(ctrl, 8)));
                _mm256_storeu_si256(starts.add(m + $off as usize + 8) as *mut __m256i, hi);
                basev = _mm256_add_epi32(basev, step16);
                _mm_shuffle_epi8(kw, ctrl)
            }};
        }
        let ka = pair!(0, o0);
        let kb = pair!(1, o1);
        _mm_storeu_si128(kinds.add(m + o0 as usize) as *mut __m128i, ka);
        _mm_storeu_si128(kinds.add(m + o1 as usize) as *mut __m128i, kb);
        let kc = pair!(2, o2);
        let kd = pair!(3, o3);
        _mm_storeu_si128(kinds.add(m + o2 as usize) as *mut __m128i, kc);
        _mm_storeu_si128(kinds.add(m + o3 as usize) as *mut __m128i, kd);
        m += o4 as usize;
    }
    m
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub(super) unsafe fn compress(
    _t: &Tables,
    st: *const u64,
    kind: *const u8,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    kinds: *mut u8,
) -> usize {
    let mut m = 0usize;
    for b in b0..b1 {
        let mut w = *st.add(b);
        if w == 0 {
            continue;
        }
        let base = (b * 64) as u32;
        while w != 0 {
            let bit = w.trailing_zeros();
            w &= w - 1;
            *starts.add(m) = base + bit;
            *kinds.add(m) = *kind.add((base + bit) as usize);
            m += 1;
        }
    }
    m
}
#[inline(always)]
unsafe fn emit_value(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    j: usize,
    lanes: &mut Lanes,
) {
    let sp = *out_spans.add(j);
    let s = sp.start as usize;
    let e = sp.end as usize;
    let k = *out_kinds.add(j);
    if k < IDENT_ESC {
        lanes.push_number_swar(src, s, e);
    } else if k == IDENT_ESC {
        lanes.push_atom(src, s, e);
    } else {
        lanes.push_atom(src, s + 1, e);
    }
}
#[cold]
unsafe fn invalid_diags(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
) {
    let char_after = |cs: u32| -> u32 {
        if cs >= nn {
            return 0;
        }
        let b = src[cs as usize];
        let l: u32 = if b < 0x80 {
            1
        } else if b >= 0xF0 {
            4
        } else if b >= 0xE0 {
            3
        } else if b >= 0xC0 {
            2
        } else {
            1
        };
        l.min(nn - cs)
    };
    for j in 0..m {
        if *out_kinds.add(j) == 255 {
            let sp = *out_spans.add(j);
            let (s, e) = (sp.start, sp.end);
            let b0 = src[s as usize];
            if b0 == b'\\' {
                lanes.push_diag(s + 1, char_after(s + 1), diag_code::INVALID_IDENTIFIER_ESCAPE);
            } else if b0 == b'#' {
                lanes.push_diag(s + 1, char_after(s + 1), diag_code::UNEXPECTED_CHARACTER);
            } else {
                lanes.push_diag(s, e - s, diag_code::UNEXPECTED_CHARACTER);
            }
        }
    }
}
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
pub(super) unsafe fn lanes_post(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
) {
    let v_num = _mm256_set1_epi8(NUM as i8);
    let v_big = _mm256_set1_epi8(BIGINT as i8);
    let v_esc = _mm256_set1_epi8(IDENT_ESC as i8);
    let v_pesc = _mm256_set1_epi8(PRIV_IDENT_ESC as i8);
    macro_rules! hits {
        ($v:expr) => {{
            let v = $v;
            _mm256_or_si256(
                _mm256_or_si256(_mm256_cmpeq_epi8(v, v_num), _mm256_cmpeq_epi8(v, v_big)),
                _mm256_or_si256(_mm256_cmpeq_epi8(v, v_esc), _mm256_cmpeq_epi8(v, v_pesc)),
            )
        }};
    }
    // 255 (INVALID) is the byte-class default for stray/control bytes and
    // reaches the output as a 1-byte token. Track "any seen" alongside the
    // value sweep; localize cold.
    let v_inv = _mm256_set1_epi8(-1i8); // 0xFF == token_kind::INVALID
    let mut inv = _mm256_setzero_si256();
    let mut i = 0usize;
    while i + 64 <= m {
        let v0 = _mm256_loadu_si256(out_kinds.add(i) as *const __m256i);
        let v1 = _mm256_loadu_si256(out_kinds.add(i + 32) as *const __m256i);
        let h0 = hits!(v0);
        let h1 = hits!(v1);
        inv = _mm256_or_si256(
            inv,
            _mm256_or_si256(_mm256_cmpeq_epi8(v0, v_inv), _mm256_cmpeq_epi8(v1, v_inv)),
        );
        let mut mask = (_mm256_movemask_epi8(h0) as u32 as u64)
            | ((_mm256_movemask_epi8(h1) as u32 as u64) << 32);
        while mask != 0 {
            emit_value(src, out_kinds, out_spans, i + mask.trailing_zeros() as usize, lanes);
            mask &= mask - 1;
        }
        i += 64;
    }
    let mut inv_dirty = _mm256_movemask_epi8(inv) != 0;
    while i < m {
        let v = _mm256_loadu_si256(out_kinds.add(i) as *const __m256i);
        let hit = hits!(v);
        let mut mask = _mm256_movemask_epi8(hit) as u32;
        let mut invm = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v, v_inv)) as u32;
        let rem = m - i;
        if rem < 32 {
            mask &= (1u32 << rem) - 1;
            invm &= (1u32 << rem) - 1;
        }
        inv_dirty |= invm != 0;
        while mask != 0 {
            emit_value(src, out_kinds, out_spans, i + mask.trailing_zeros() as usize, lanes);
            mask &= mask - 1;
        }
        i += 32;
    }
    if inv_dirty {
        invalid_diags(src, out_kinds, out_spans, m, nn, lanes);
    }
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub(super) unsafe fn lanes_post(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
) {
    let mut inv = 0u64;
    let mut i = 0usize;
    while i + 8 <= m {
        let x = load8(out_kinds, i);
        let mut hits = eqm(x, NUM) | eqm(x, BIGINT) | eqm(x, IDENT_ESC) | eqm(x, PRIV_IDENT_ESC);
        inv |= eqm(x, 255);
        while hits != 0 {
            emit_value(src, out_kinds, out_spans, i + (hits.trailing_zeros() >> 3) as usize, lanes);
            hits &= hits - 1;
        }
        i += 8;
    }
    let mut inv_dirty = inv != 0;
    while i < m {
        let k = *out_kinds.add(i);
        if k == NUM || k == BIGINT || k == IDENT_ESC || k == PRIV_IDENT_ESC {
            emit_value(src, out_kinds, out_spans, i, lanes);
        }
        inv_dirty |= k == 255;
        i += 1;
    }
    if inv_dirty {
        invalid_diags(src, out_kinds, out_spans, m, nn, lanes);
    }
}

#[cfg(test)]
mod tests {
    use super::compress;
    use crate::tables::Tables;

    fn xorshift(s: &mut u64) -> u64 {
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        *s
    }

    #[test]
    fn compress_matches_scalar_reference() {
        let t = Tables::new();
        let mut cases: Vec<Vec<u64>> = Vec::new();
        let all_pairs: Vec<u64> = (0..65536u64)
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|c| c.iter().enumerate().fold(0u64, |w, (i, v)| w | (v << (16 * i))))
            .collect();
        cases.push(all_pairs);
        let mut s = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..3 {
            cases.push(core::iter::repeat_with(|| xorshift(&mut s)).take(512).collect());
        }
        cases.push(
            core::iter::repeat_with(|| xorshift(&mut s) & xorshift(&mut s) & xorshift(&mut s))
                .take(512)
                .collect(),
        );
        cases.push(
            core::iter::repeat_with(|| xorshift(&mut s) | xorshift(&mut s) | xorshift(&mut s))
                .take(512)
                .collect(),
        );
        cases.push(vec![!0u64; 64]);
        cases.push({
            let mut v = vec![0u64; 64];
            v[63] = 1u64 << 63;
            v
        });
        for st in &cases {
            let nb = st.len();
            let n = nb * 64;
            let mut kind = vec![0u8; n];
            let mut ks = 0xdead_beef_cafe_f00du64;
            for b in kind.iter_mut() {
                *b = (xorshift(&mut ks) & 0xff) as u8;
            }
            let mut starts = vec![0u32; n + 64];
            let mut kinds = vec![0u8; n + 64];
            let m = unsafe {
                compress(
                    &t,
                    st.as_ptr(),
                    kind.as_ptr(),
                    0,
                    nb,
                    starts.as_mut_ptr(),
                    kinds.as_mut_ptr(),
                )
            };
            let mut rs: Vec<u32> = Vec::new();
            let mut rk: Vec<u8> = Vec::new();
            for (b, &w0) in st.iter().enumerate() {
                let mut w = w0;
                while w != 0 {
                    let bit = w.trailing_zeros() as usize;
                    w &= w - 1;
                    rs.push((b * 64 + bit) as u32);
                    rk.push(kind[b * 64 + bit]);
                }
            }
            assert_eq!(m, rs.len(), "token count mismatch (nb={nb})");
            assert_eq!(&starts[..m], &rs[..], "starts mismatch");
            assert_eq!(&kinds[..m], &rk[..], "kinds mismatch");
        }
    }
}
