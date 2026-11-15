use oxc_simd::{Avx2, Vec256, cpu_version};

use super::BRACKET_BYTES;

/// First byte at or after i that is one of needles, or src.len().
///
/// Needle order matters for speed. See comment on [`any_of`] below.
#[cpu_version(Avx2)]
#[inline]
pub fn find_avx2<const K: usize>(src: &[u8], mut i: usize, needles: [u8; K], cpu: Avx2) -> usize {
    let n = src.len();
    if i > n {
        return n;
    }
    while i + 32 <= n {
        // SAFETY: the guard keeps i <= n, so i + 32 cannot wrap, and the 32 bytes at i are in src.
        let v = unsafe { Vec256::load_unaligned(src.as_ptr().add(i), cpu) };
        let m = any_of(v, needles).movemask();
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
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
#[cpu_version(Avx2)]
#[inline]
pub fn bracket_bits_avx2(src: &[u8], base: usize, cpu: Avx2) -> u64 {
    let block = &src[base..base + 64];
    let (chunks, _) = block.as_chunks::<32>();
    let mut out = 0u64;
    for (half, chunk) in chunks.iter().enumerate() {
        let v = Vec256::read_unaligned(chunk, cpu);
        let mask = any_of(v, BRACKET_BYTES).movemask();
        out |= u64::from(mask) << (half * 32);
    }
    out
}

// The compares are OR-folded pairwise, level by level, so the tree stays shallow for 7 needles.
//
// Needle order matters for speed. When 2 bytes differ in only 1 bit, LLVM folds their 2 compares
// and the OR into 1 AND and 1 compare, saving an op.
//
// For example, `<` is 0x3C (0b0011_1100) and `>` is 0x3E (0b0011_1110). They differ only in bit 1,
// so clearing that bit with `& 0xFD` turns both of them into 0x3C:
//
//     (v == 0x3C) | (v == 0x3E)    2 compares + 1 OR
//     (v & 0xFD) == 0x3C           1 AND + 1 compare
//
// LLVM only does this when the 2 compares are OR-ed together directly,
// i.e. when the bytes are at indexes `2 * j` and `2 * j + 1` in `needles`.
// The byte sets passed to `find` are ordered to make such pairs.
#[cpu_version(Avx2)]
#[inline(always)]
fn any_of<const N: usize>(v: Vec256<Avx2>, needles: [u8; N]) -> Vec256<Avx2> {
    // Not needles.map, which is std code built without AVX2 and may call the compares out of line.
    let mut m = [Vec256::zero(v.cpu()); N];
    for (slot, &c) in m.iter_mut().zip(&needles) {
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
