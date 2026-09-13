#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
mod primitives {
    use core::arch::x86_64::*;

    /// Load 256 bits starting at byte `i` as an `__m256i`.
    ///
    /// The load is unaligned, so neither `src` nor `i` has any alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `src.add(i)` must be valid for reads of 32 bytes.
    #[inline(always)]
    pub unsafe fn load256(src: *const u8, i: usize) -> __m256i {
        _mm256_loadu_si256(src.add(i) as *const __m256i)
    }

    /// Compare every byte of chunk `v` against `c`.
    ///
    /// Returns a `__m256i` where each of the 32 lanes is `0xFF`
    /// where the lane in `v` equals `c`, and `0x00` where it does not.
    #[inline(always)]
    pub fn veq(v: __m256i, c: u8) -> __m256i {
        // SAFETY: These intrinsics touch no memory and require only the `avx2` target feature,
        // which this module's `#[cfg]` guarantees
        unsafe { _mm256_cmpeq_epi8(v, _mm256_set1_epi8(c as i8)) }
    }

    /// Gather the top bit of each byte of chunk `v` into a 32-bit mask.
    ///
    /// Bit N of the result is the top bit of byte N in `v`.
    #[inline(always)]
    pub fn mm(v: __m256i) -> u32 {
        // SAFETY: This intrinsic touches no memory and requires only the `avx2` target feature,
        // which this module's `#[cfg]` guarantees
        unsafe { _mm256_movemask_epi8(v) as u32 }
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
mod primitives {
    use core::ptr;

    /// Load 64 bits starting at byte `i` as a `u64`.
    ///
    /// The load is unaligned, so neither `src` nor `i` has any alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `src.add(i)` must be valid for reads of 8 bytes.
    #[inline(always)]
    pub unsafe fn load64(src: *const u8, i: usize) -> u64 {
        ptr::read_unaligned(src.add(i) as *const u64)
    }

    /// Compare every byte of chunk `x` against `b`.
    ///
    /// Returns a `u64` where each of the 8 bytes is `0x80`
    /// where the byte in `x` equals `b`, and `0x00` where it does not.
    ///
    /// On a little-endian target, `trailing_zeros() >> 3` gives the offset of the first match.
    /// This is the SWAR counterpart of `veq` followed by `mm`.
    #[inline(always)]
    pub fn eqm(x: u64, b: u8) -> u64 {
        let lo = 0x0101_0101_0101_0101u64;
        let low7 = 0x7F7F_7F7F_7F7F_7F7Fu64;
        let y = x ^ lo.wrapping_mul(b as u64);
        !((y & low7).wrapping_add(low7) | y | low7)
    }
}

pub(super) use primitives::*;
