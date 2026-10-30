use std::mem::{self, MaybeUninit};

use crate::{cpu::Avx2, simd::avx2::Vec256};

/// Ordered for speed, pairing `(` with `)`, `[` with `{` and `]` with `}` (see [`any_of`]).
const BRACKET_BYTES: [u8; 6] = [b'(', b')', b'[', b'{', b']', b'}'];

/// First byte at or after i that is one of needles, or n.
///
/// Needle order matters for speed. See comment on [`any_of`] below.
#[inline]
#[target_feature(enable = "avx2,bmi2,popcnt")]
pub unsafe fn find_avx2<const K: usize>(
    src: *const u8,
    n: usize,
    mut i: usize,
    needles: [u8; K],
    cpu: Avx2,
) -> usize {
    while i + 32 <= n {
        let m = any_of(Vec256::load_unaligned(src.add(i), cpu), needles).movemask();
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
#[target_feature(enable = "avx2,bmi2,popcnt")]
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
#[inline(always)]
fn any_of<const N: usize>(v: Vec256, needles: [u8; N]) -> Vec256 {
    let mut m = array_map(needles, |c| v.eq_u8(c));
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

/// Equivalent of `<[T; N]>::map`, which is guaranteed to be inlined.
///
/// `map` is `std` code, which is compiled without AVX2.
/// If LLVM doesn't inline it into the caller, it calls the intrinsics out of line.
#[inline(always)]
fn array_map<T, U, const N: usize>(array: [T; N], f: impl Fn(T) -> U) -> [U; N] {
    // If `f` panics, the values already in `out` aren't dropped. Ensure that can't leak memory.
    const { assert!(!mem::needs_drop::<U>()) };

    let mut out = [const { MaybeUninit::<U>::uninit() }; N];

    for (slot, value) in out.iter_mut().zip(array) {
        slot.write(f(value));
    }

    // SAFETY: `out` and `array` are both `N` long, so the loop initialized every element of `out`.
    // `MaybeUninit<U>` has the same layout as `U`.
    unsafe { mem::transmute_copy::<[MaybeUninit<U>; N], [U; N]>(&out) }
}
