use oxc_simd::{Avx2, BASELINE, Cpu, DynCpu, Vec256, cpu_multiversion, cpu_version};

use crate::pipeline::bitmap::Bitmap;

/// JSXIdentifier admits `-`, so `data-x` and `aria-label` are one name token
/// where JS would read three. Fuse every hyphen in `[a, b)` into the run
/// before it: drop its token start and its `opch` bit (so `coalesce` cannot
/// read it as an operator), plus the token start of the run that follows.
///
/// The caller only ever passes name/attribute regions: strings and `{}`
/// containers are consumed whole before the next region begins, so a hyphen
/// reached here is never a minus.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn jsx_glue_hyphens(
    src: *const u8,
    n: usize,
    st: Bitmap,
    opch: Bitmap,
    word: Bitmap,
    a: usize,
    b: usize,
    cpu: impl Cpu,
) {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => jsx_glue_hyphens_avx2(src, n, st, opch, word, a, b, cpu),
        _ => jsx_glue_hyphens_generic(src, n, st, opch, word, a, b),
    }
}

/// [`jsx_glue_hyphens`] for AVX2.
#[cpu_version(Avx2)]
#[inline(always)]
unsafe fn jsx_glue_hyphens_avx2(
    src: *const u8,
    n: usize,
    st: Bitmap,
    opch: Bitmap,
    word: Bitmap,
    a: usize,
    b: usize,
    cpu: Avx2,
) {
    let mut i = a;
    let mut last = usize::MAX;
    while i < b {
        let mut m = Vec256::load_unaligned(src.add(i), cpu).eq_u8(b'-').movemask();
        let rem = b - i;
        if rem < 32 {
            m &= (1u32 << rem) - 1;
        }
        while m != 0 {
            let h = i + m.trailing_zeros() as usize;
            m &= m - 1;
            glue_hyphen_at(n, st, opch, word, h, &mut last, cpu);
        }
        i += 32;
    }
}

/// [`jsx_glue_hyphens`] for all other CPUs.
#[cpu_version(Generic)]
#[inline(always)]
unsafe fn jsx_glue_hyphens_generic(
    src: *const u8,
    n: usize,
    st: Bitmap,
    opch: Bitmap,
    word: Bitmap,
    a: usize,
    b: usize,
) {
    let cpu = BASELINE;

    let mut last = usize::MAX;
    let mut h = a;
    while h < b {
        if *src.add(h) == b'-' {
            glue_hyphen_at(n, st, opch, word, h, &mut last, cpu);
        }
        h += 1;
    }
}

#[cpu_multiversion]
#[inline(always)]
unsafe fn glue_hyphen_at(
    n: usize,
    st: Bitmap,
    opch: Bitmap,
    word: Bitmap,
    h: usize,
    last: &mut usize,
    cpu: impl Cpu,
) {
    if h == 0 || !(word.get(h - 1, cpu) || *last == h - 1) {
        return;
    }
    st.clear(h, cpu);
    opch.clear(h, cpu);
    if h + 1 < n && word.get(h + 1, cpu) {
        st.clear(h + 1, cpu);
    }
    *last = h;
}
