use oxc_simd::{Cpu, cpu_multiversion};

use crate::token::{KW_KIND_BASE, tk};

use crate::pipeline::bitmap::Bitmap;

#[cpu_multiversion]
#[inline]
pub unsafe fn misc_post(
    src: *const u8,
    n: usize,
    st: Bitmap,
    word: Bitmap,
    misc: Bitmap,
    kind: *mut u8,
    nesc: usize,
    cpu: impl Cpu,
) {
    if nesc != 0 {
        misc_post_impl(src, n, st, word, misc, kind, cpu)
    }
}

#[cpu_multiversion]
#[inline(never)]
unsafe fn misc_post_impl(
    src: *const u8,
    n: usize,
    st: Bitmap,
    word: Bitmap,
    misc: Bitmap,
    kind: *mut u8,
    cpu: impl Cpu,
) {
    let nw = (n + 63) >> 6;
    for w in 0..nw {
        let mut m = misc.block(w, cpu);
        while m != 0 {
            let bit = m.trailing_zeros() as usize;
            m &= m - 1;
            let p = (w << 6) + bit;
            if *src.add(p) != b'\\' || *src.add(p + 1) != b'u' {
                continue;
            }
            if !st.get(p, cpu) {
                continue;
            }
            if p == 0 || !word.get(p - 1, cpu) {
                continue;
            }
            let tt = st.prev1(p, cpu);
            let k = *kind.add(tt as usize);
            if k == tk!(Ident) || (k >= KW_KIND_BASE && k != tk!(Invalid)) {
                *kind.add(tt as usize) = tk!(IdentEscaped);
                st.clear(p, cpu);
            }
        }
    }
}
