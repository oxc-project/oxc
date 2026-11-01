use std::{
    arch::x86_64::*,
    mem::{self, MaybeUninit},
};

use crate::pipeline::{
    chunk::{load256, mm, veq},
    cpu::Avx2,
};

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
        let m = mm(any_of(load256(src, i), needles, cpu));
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
    let mut out = 0u64;
    for half in 0..2 {
        // SAFETY: `block` holds 64 bytes, so a 32-byte load at offset 0 or 32 stays inside it;
        // the OR-fold touches no memory and needs only `avx2`, which this function's
        // `#[target_feature]` enables.
        let v = unsafe { load256(block.as_ptr(), half * 32) };
        let m = unsafe { any_of(v, BRACKET_BYTES, cpu) };
        // SAFETY: This function's `#[target_feature]` enables AVX2
        let mask = unsafe { mm(m) };
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
unsafe fn any_of<const N: usize>(v: __m256i, needles: [u8; N], _cpu: Avx2) -> __m256i {
    let mut m = array_map(needles, |c| veq(v, c));
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
