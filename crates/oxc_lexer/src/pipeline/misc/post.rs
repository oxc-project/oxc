use crate::token::{KW_KIND_BASE, tk};

use crate::pipeline::bitmap::{bm_clear, bm_get, bm_prev1};

#[inline]
pub unsafe fn misc_post(
    src: *const u8,
    n: usize,
    st: *mut u64,
    word: *const u64,
    misc: *const u64,
    kind: *mut u8,
    nesc: usize,
) {
    if nesc != 0 {
        misc_post_impl(src, n, st, word, misc, kind)
    }
}

unsafe fn misc_post_impl(
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
            if !bm_get(st, p) {
                continue;
            }
            if p == 0 || !bm_get(word, p - 1) {
                continue;
            }
            let tt = bm_prev1(st, p);
            let k = *kind.add(tt as usize);
            if k == tk!(Ident) || (k >= KW_KIND_BASE && k != tk!(Invalid)) {
                *kind.add(tt as usize) = tk!(IdentEscaped);
                bm_clear(st, p);
            }
        }
    }
}
