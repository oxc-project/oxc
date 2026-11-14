use oxc_simd::cpu_version;

use crate::pipeline::chunk::eqm;

/// First byte at or after i that is one of needles, or src.len().
#[cpu_version(Generic)]
#[inline(always)]
pub fn find_generic<const K: usize>(src: &[u8], mut i: usize, needles: [u8; K]) -> usize {
    let n = src.len();
    if i > n {
        return n;
    }
    while i + 8 <= n {
        // SAFETY: the guard keeps i <= n, so i + 8 cannot wrap, and the 8 bytes at i are in src.
        let x = u64::from_le_bytes(unsafe { src.as_ptr().add(i).cast::<[u8; 8]>().read() });
        let mut m = 0u64;
        for c in needles {
            m |= eqm(x, c);
        }
        if m != 0 {
            return i + (m.trailing_zeros() >> 3) as usize;
        }
        i += 8;
    }
    while i < n {
        if needles.contains(&src[i]) {
            return i;
        }
        i += 1;
    }
    n
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`): bit `i` for byte `base + i`.
#[cpu_version(Generic)]
#[inline]
pub fn bracket_bits_generic(src: &[u8], base: usize) -> u64 {
    let mut out = 0u64;
    for (i, &c) in src[base..base + 64].iter().enumerate() {
        if matches!(c, b'(' | b')' | b'[' | b']' | b'{' | b'}') {
            out |= 1u64 << i;
        }
    }
    out
}
