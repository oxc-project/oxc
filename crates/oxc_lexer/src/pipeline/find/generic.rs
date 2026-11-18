use oxc_simd::cpu_version;

use crate::pipeline::{chunk::eqm, source::Source};

/// First byte at or after i that is one of needles, or src.len().
#[cpu_version(Generic)]
#[inline(always)]
pub fn find_generic<const K: usize>(src: Source, mut i: usize, needles: [u8; K]) -> usize {
    let n = src.len();
    if let Some(last) = n.checked_sub(8) {
        while i <= last {
            let m = any_of8(u64::from_le_bytes(*src.window(i)), needles);
            if m != 0 {
                return i + (m.trailing_zeros() >> 3) as usize;
            }
            i += 8;
        }
    }
    if i < n {
        // The last window reaches into the padding, so its bytes past n are dropped.
        let m = any_of8(u64::from_le_bytes(*src.window(i)), needles);
        let m = m & (u64::MAX >> (8 * (8 - (n - i))));
        if m != 0 {
            return i + (m.trailing_zeros() >> 3) as usize;
        }
    }
    n
}

/// The high bit of each byte of x that equals one of needles.
#[cpu_version(Generic)]
#[inline(always)]
fn any_of8<const K: usize>(x: u64, needles: [u8; K]) -> u64 {
    let mut m = 0u64;
    for c in needles {
        m |= eqm(x, c);
    }
    m
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
