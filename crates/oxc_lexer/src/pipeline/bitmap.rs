#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use core::arch::x86_64::*;

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
#[inline(always)]
pub(super) unsafe fn bm_set1(bm: *mut u64, i: usize) {
    *bm.add(i >> 6) |= 1u64 << (i & 63);
}
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
