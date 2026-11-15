use oxc_simd::cpu_version;

use crate::pipeline::{chunk::eqm, source::Source};

/// First byte at or after i that is one of needles, or src.len().
#[cpu_version(Generic)]
#[inline(always)]
pub fn find_generic<const K: usize>(src: Source, mut i: usize, needles: [u8; K]) -> usize {
    let n = src.len();
    while i < n {
        let x = u64::from_le_bytes(*src.window(i));
        let mut m = 0u64;
        for c in needles {
            m |= eqm(x, c);
        }
        if m != 0 {
            let hit = i + (m.trailing_zeros() >> 3) as usize;
            // A hit at or past n is in the padding, so the loop ends instead.
            if hit < n {
                return hit;
            }
        }
        i += 8;
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
