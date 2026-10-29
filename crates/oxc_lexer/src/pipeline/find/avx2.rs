use std::arch::x86_64::*;

use crate::pipeline::chunk::{load256, mm, veq};

/// First byte at or after i that is one of needles, or n.
#[inline]
pub unsafe fn find_avx2<const K: usize>(
    src: *const u8,
    n: usize,
    mut i: usize,
    needles: [u8; K],
) -> usize {
    while i + 32 <= n {
        let m = mm(any_of(load256(src, i), needles));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        if needles.contains(&*src.add(i)) {
            return i;
        }
        i += 1;
    }
    n
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`): bit `i` for byte `base + i`.
#[inline]
pub fn bracket_bits_avx2(src: &[u8], base: usize) -> u64 {
    let block = &src[base..base + 64];
    let mut out = 0u64;
    for half in 0..2 {
        // SAFETY: `block` holds 64 bytes, so a 32-byte load at offset 0 or 32 stays inside it;
        // the OR-fold touches no memory and needs only `avx2`, which this module's `#[cfg]`
        // guarantees.
        let v = unsafe { load256(block.as_ptr(), half * 32) };
        let m = unsafe { any_of(v, [b'(', b')', b'[', b']', b'{', b'}']) };
        // SAFETY: This module is only compiled on platforms that support AVX2
        let mask = unsafe { mm(m) };
        out |= u64::from(mask) << (half * 32);
    }
    out
}

// The compares are OR-folded pairwise, level by level, so the tree stays shallow for 7 needles.
#[inline(always)]
unsafe fn any_of<const N: usize>(v: __m256i, needles: [u8; N]) -> __m256i {
    let mut m = needles.map(|c| veq(v, c));
    let mut len = N;
    while len > 1 {
        let half = len / 2;
        for j in 0..half {
            m[j] = _mm256_or_si256(m[2 * j], m[2 * j + 1]);
        }
        if len % 2 == 1 {
            m[half] = m[len - 1];
        }
        len = half + len % 2;
    }
    m[0]
}
