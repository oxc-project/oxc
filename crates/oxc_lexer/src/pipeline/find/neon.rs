use oxc_simd::{Neon, Vec128, cpu_version};

use crate::pipeline::chunk::mm64;

use super::BRACKET_BYTES;

/// First byte at or after i that is one of needles, or n.
///
/// 16 bytes at a time, as a nibble mask is the cheapest way to find the first hit on Neon.
#[cpu_version(Neon)]
#[inline]
pub unsafe fn find_neon<const K: usize>(
    src: *const u8,
    n: usize,
    mut i: usize,
    needles: [u8; K],
    cpu: Neon,
) -> usize {
    while i + 16 <= n {
        let m = any_of(Vec128::load_unaligned(src.add(i), cpu), needles).nibble_mask();
        if m != 0 {
            return i + (m.trailing_zeros() / 4) as usize;
        }
        i += 16;
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
#[cpu_version(Neon)]
#[inline]
pub fn bracket_bits_neon(src: &[u8], base: usize, cpu: Neon) -> u64 {
    let (chunks, _) = src[base..base + 64].as_chunks::<16>();
    mm64(
        any_of(Vec128::read_unaligned(&chunks[0], cpu), BRACKET_BYTES),
        any_of(Vec128::read_unaligned(&chunks[1], cpu), BRACKET_BYTES),
        any_of(Vec128::read_unaligned(&chunks[2], cpu), BRACKET_BYTES),
        any_of(Vec128::read_unaligned(&chunks[3], cpu), BRACKET_BYTES),
    )
}

/// Lanes of `v` which equal any of `needles`.
///
/// The compares are OR-folded pairwise, level by level, so the tree stays shallow.
/// Needle order matters for speed, as it does for `any_of` in `avx2.rs`.
#[cpu_version(Neon)]
#[inline(always)]
fn any_of<const N: usize>(v: Vec128<Neon>, needles: [u8; N]) -> Vec128<Neon> {
    let mut m = [Vec128::zero(v.cpu()); N];
    for (slot, c) in m.iter_mut().zip(needles) {
        *slot = v.eq_u8(c);
    }
    let mut len = N;
    while len > 1 {
        let half = len / 2;
        for j in 0..half {
            m[j] = m[2 * j] | m[2 * j + 1];
        }
        if len % 2 == 1 {
            m[half] = m[len - 1];
        }
        len = half + len % 2;
    }
    m[0]
}
