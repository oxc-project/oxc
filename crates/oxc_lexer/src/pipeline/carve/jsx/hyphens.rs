use super::super::super::bitmap::{bm_clear, bm_get};

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use super::super::super::chunk::{load256, mm, veq};

/// JSXIdentifier admits `-`, so `data-x` and `aria-label` are one name token
/// where JS would read three. Fuse every hyphen in `[a, b)` into the run
/// before it: drop its token start and its `opch` bit (so `coalesce` cannot
/// read it as an operator), plus the token start of the run that follows.
///
/// The caller only ever passes name/attribute regions: strings and `{}`
/// containers are consumed whole before the next region begins, so a hyphen
/// reached here is never a minus.
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline(always)]
pub(super) unsafe fn jsx_glue_hyphens(
    src: *const u8,
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    a: usize,
    b: usize,
) {
    let mut i = a;
    let mut last = usize::MAX;
    while i < b {
        let mut m = mm(veq(load256(src, i), b'-'));
        let rem = b - i;
        if rem < 32 {
            m &= (1u32 << rem) - 1;
        }
        while m != 0 {
            let h = i + m.trailing_zeros() as usize;
            m &= m - 1;
            glue_hyphen_at(n, st, opch, word, h, &mut last);
        }
        i += 32;
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline(always)]
pub(super) unsafe fn jsx_glue_hyphens(
    src: *const u8,
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    a: usize,
    b: usize,
) {
    let mut last = usize::MAX;
    let mut h = a;
    while h < b {
        if *src.add(h) == b'-' {
            glue_hyphen_at(n, st, opch, word, h, &mut last);
        }
        h += 1;
    }
}

#[inline(always)]
unsafe fn glue_hyphen_at(
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    h: usize,
    last: &mut usize,
) {
    if h == 0 || !(bm_get(word, h - 1) || *last == h - 1) {
        return;
    }
    bm_clear(st, h);
    bm_clear(opch, h);
    if h + 1 < n && bm_get(word, h + 1) {
        bm_clear(st, h + 1);
    }
    *last = h;
}
