#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use core::arch::x86_64::*;

/// Get bit `i`.
///
/// Returns `true` if the bit is set, `false` if it is clear.
///
/// # SAFETY
///
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads of `(i / 64) + 1` words.
#[inline(always)]
pub(super) unsafe fn bm_get(bm: *const u64, i: usize) -> bool {
    (*bm.add(i >> 6) >> (i & 63)) & 1 != 0
}

/// Get index of the first clear bit at or after `i`.
///
/// If every bit in `i..n` is set, returns `n`.
///
/// # SAFETY
///
/// - `i` must be `<= n`.
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads of `n.div_ceil(64) + 1` words.
///   The last word is read only when `i` is `n` and `n` is a multiple of 64.
#[inline(always)]
pub(super) unsafe fn bm_next0(bm: *const u64, i: usize, n: usize) -> usize {
    let mut w = i >> 6;
    let mut inv = !*bm.add(w) & !((1u64 << (i & 63)).wrapping_sub(1));
    while inv == 0 {
        w += 1;
        if (w << 6) >= n {
            return n;
        }
        inv = !*bm.add(w);
    }
    let r = (w << 6) + inv.trailing_zeros() as usize;
    if r < n { r } else { n }
}

/// Get index of the first set bit at or after `i`.
///
/// If every bit in `i..n` is clear, returns `n`.
///
/// # SAFETY
///
/// - `i` must be `<= n`.
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads of `n.div_ceil(64) + 1` words.
///   The last word is read only when `i` is `n` and `n` is a multiple of 64.
#[inline(always)]
pub(super) unsafe fn bm_next1(bm: *const u64, i: usize, n: usize) -> usize {
    let mut w = i >> 6;
    let x = *bm.add(w) & !((1u64 << (i & 63)).wrapping_sub(1));
    if x != 0 {
        let r = (w << 6) + x.trailing_zeros() as usize;
        return if r < n { r } else { n };
    }
    w += 1;
    while (w << 6) < n {
        let x = *bm.add(w);
        if x != 0 {
            let r = (w << 6) + x.trailing_zeros() as usize;
            return if r < n { r } else { n };
        }
        w += 1;
    }
    n
}

/// Get index of the last set bit before `p`.
///
/// If bits `0..p` are all clear, returns `-1`.
///
/// The scan always runs down to bit 0, so there is no lower bound parameter.
/// Reporting absence as `-1` is why the return type is `i64`, not `usize`.
///
/// # SAFETY
///
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads of `p.div_ceil(64)` words.
///   Nothing is read when `p` is 0.
#[inline(always)]
pub(super) unsafe fn bm_prev1(bm: *const u64, p: usize) -> i64 {
    if p == 0 {
        return -1;
    }
    let i = p - 1;
    let mut w = (i >> 6) as i64;
    let lim = i & 63;
    let mask = if lim == 63 { !0u64 } else { (1u64 << (lim + 1)) - 1 };
    let x = *bm.add(w as usize) & mask;
    if x != 0 {
        return (w << 6) + (63 - x.leading_zeros() as i64);
    }
    w -= 1;
    while w >= 0 {
        let x = *bm.add(w as usize);
        if x != 0 {
            return (w << 6) + (63 - x.leading_zeros() as i64);
        }
        w -= 1;
    }
    -1
}

/// Set bit `i`.
///
/// # SAFETY
///
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads and writes of `(i / 64) + 1` words.
#[inline(always)]
pub(super) unsafe fn bm_set1(bm: *mut u64, i: usize) {
    *bm.add(i >> 6) |= 1u64 << (i & 63);
}

/// Clear bits `a` to `b` inclusive.
///
/// If `a > b`, this is a no-op - no bits are cleared.
///
/// # SAFETY
///
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads and writes of `(b / 64) + 1` words.
///   Nothing is accessed when `a > b`.
#[inline(always)]
pub(super) unsafe fn bm_clear_range(bm: *mut u64, a: usize, b: usize) {
    if a > b {
        return;
    }
    let wa = a >> 6;
    let wb = b >> 6;
    let lo = (!0u64) << (a & 63);
    let hi = if (b & 63) == 63 { !0u64 } else { (1u64 << ((b & 63) + 1)) - 1 };
    if wa == wb {
        *bm.add(wa) &= !(lo & hi);
        return;
    }
    *bm.add(wa) &= !lo;
    let mut w = wa + 1;
    while w < wb {
        *bm.add(w) = 0;
        w += 1;
    }
    *bm.add(wb) &= !hi;
}

/// Check if any bit is set in the first `nw` words.
///
/// Returns `true` if any bit is set, `false` if no bits are set.
///
/// The unit is words, not the bit indices all other functions in this file take.
///
/// # SAFETY
///
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads of `nw` words.
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline]
pub(super) unsafe fn bm_any(bm: *const u64, nw: usize) -> bool {
    let mut w = 0usize;
    while w + 4 <= nw {
        let v = _mm256_loadu_si256(bm.add(w) as *const __m256i);
        if _mm256_testz_si256(v, v) == 0 {
            return true;
        }
        w += 4;
    }
    while w < nw {
        if *bm.add(w) != 0 {
            return true;
        }
        w += 1;
    }
    false
}

/// Check if any bit is set in the first `nw` words.
///
/// Returns `true` if any bit is set, `false` if no bits are set.
///
/// The unit is words, not the bit indices all other functions in this file take.
///
/// # SAFETY
///
/// - `bm` must be aligned for `u64`.
/// - `bm` must be valid for reads of `nw` words.
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline]
pub(super) unsafe fn bm_any(bm: *const u64, nw: usize) -> bool {
    let mut w = 0usize;
    while w + 4 <= nw {
        if (*bm.add(w) | *bm.add(w + 1) | *bm.add(w + 2) | *bm.add(w + 3)) != 0 {
            return true;
        }
        w += 4;
    }
    while w < nw {
        if *bm.add(w) != 0 {
            return true;
        }
        w += 1;
    }
    false
}
