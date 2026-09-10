#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
mod primitives {
    use core::arch::x86_64::*;

    #[inline(always)]
    pub unsafe fn load256(src: *const u8, i: usize) -> __m256i {
        _mm256_loadu_si256(src.add(i) as *const __m256i)
    }

    #[inline(always)]
    pub unsafe fn veq(v: __m256i, c: u8) -> __m256i {
        _mm256_cmpeq_epi8(v, _mm256_set1_epi8(c as i8))
    }

    #[inline(always)]
    pub unsafe fn mm(v: __m256i) -> u32 {
        _mm256_movemask_epi8(v) as u32
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
mod primitives {
    use core::ptr;

    #[inline(always)]
    pub unsafe fn load64(src: *const u8, i: usize) -> u64 {
        ptr::read_unaligned(src.add(i) as *const u64)
    }

    #[inline(always)]
    pub fn eqm(x: u64, b: u8) -> u64 {
        let lo = 0x0101_0101_0101_0101u64;
        let low7 = 0x7F7F_7F7F_7F7F_7F7Fu64;
        let y = x ^ lo.wrapping_mul(b as u64);
        !((y & low7).wrapping_add(low7) | y | low7)
    }
}

pub(super) use primitives::*;
