use oxc_simd::{Cpu, cpu_multiversion};

use crate::token::{KW_KIND_BASE, tk};

use crate::pipeline::{bitmap::Bitmaps, kinds::Kinds, source::Source};

#[cpu_multiversion]
#[inline]
pub unsafe fn misc_post(src: Source, bitmaps: &Bitmaps, kinds: Kinds, nesc: usize, cpu: impl Cpu) {
    if nesc != 0 {
        misc_post_impl(src, bitmaps, kinds, cpu)
    }
}

#[cpu_multiversion]
#[inline(never)]
unsafe fn misc_post_impl(src: Source, bitmaps: &Bitmaps, kinds: Kinds, cpu: impl Cpu) {
    let Bitmaps { token_start, word, misc, .. } = *bitmaps;
    let nw = src.block_count();
    for w in 0..nw {
        let mut m = misc.block(w, cpu);
        while m != 0 {
            let bit = m.trailing_zeros() as usize;
            m &= m - 1;
            let p = (w << 6) + bit;
            if src[p] != b'\\' || src[p + 1] != b'u' {
                continue;
            }
            if !token_start.get(p, cpu) {
                continue;
            }
            if p == 0 || !word.get(p - 1, cpu) {
                continue;
            }
            let tt = token_start.prev1(p, cpu);
            let k = kinds.get(tt as usize);
            if k == tk!(Ident) || (k >= KW_KIND_BASE && k != tk!(Invalid)) {
                kinds.set(tt as usize, tk!(IdentEscaped));
                token_start.clear(p, cpu);
            }
        }
    }
}
