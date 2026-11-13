use oxc_simd::{Avx2, Cpu, DynCpu, Vec256, cpu_multiversion, cpu_version};

/// One bit per source byte, a u64 per 64-byte block; no method checks bounds.
#[derive(Clone, Copy)]
pub(super) struct Bitmap(*mut u64);

impl Bitmap {
    pub(super) fn new(ptr: *mut u64) -> Self {
        Self(ptr)
    }

    pub(super) fn as_ptr(self) -> *mut u64 {
        self.0
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn block(self, block_idx: usize, _cpu: impl Cpu) -> u64 {
        *self.0.add(block_idx)
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn get(self, i: usize, _cpu: impl Cpu) -> bool {
        (*self.0.add(i >> 6) >> (i & 63)) & 1 != 0
    }

    /// First clear bit at or after i, or n if bits i..n are all set.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn next0(self, i: usize, n: usize, _cpu: impl Cpu) -> usize {
        let mut w = i >> 6;
        // Reads one block past the source when i == n and n is a multiple of 64.
        let mut inv = !*self.0.add(w) & !((1u64 << (i & 63)).wrapping_sub(1));
        while inv == 0 {
            w += 1;
            if (w << 6) >= n {
                return n;
            }
            inv = !*self.0.add(w);
        }
        let r = (w << 6) + inv.trailing_zeros() as usize;
        if r < n { r } else { n }
    }

    /// Last set bit before p, or -1 if there is none.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn prev1(self, p: usize, _cpu: impl Cpu) -> i64 {
        if p == 0 {
            return -1;
        }
        let i = p - 1;
        let mut w = (i >> 6) as i64;
        let lim = i & 63;
        let mask = if lim == 63 { !0u64 } else { (1u64 << (lim + 1)) - 1 };
        let x = *self.0.add(w as usize) & mask;
        if x != 0 {
            return (w << 6) + (63 - x.leading_zeros() as i64);
        }
        w -= 1;
        while w >= 0 {
            let x = *self.0.add(w as usize);
            if x != 0 {
                return (w << 6) + (63 - x.leading_zeros() as i64);
            }
            w -= 1;
        }
        -1
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn set(self, i: usize, _cpu: impl Cpu) {
        *self.0.add(i >> 6) |= 1u64 << (i & 63);
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn clear(self, i: usize, _cpu: impl Cpu) {
        *self.0.add(i >> 6) &= !(1u64 << (i & 63));
    }

    /// Clears bits a..=b, or nothing if a > b.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn clear_range(self, a: usize, b: usize, _cpu: impl Cpu) {
        if a > b {
            return;
        }
        let wa = a >> 6;
        let wb = b >> 6;
        let lo = (!0u64) << (a & 63);
        let hi = if (b & 63) == 63 { !0u64 } else { (1u64 << ((b & 63) + 1)) - 1 };
        if wa == wb {
            *self.0.add(wa) &= !(lo & hi);
            return;
        }
        *self.0.add(wa) &= !lo;
        let mut w = wa + 1;
        while w < wb {
            *self.0.add(w) = 0;
            w += 1;
        }
        *self.0.add(wb) &= !hi;
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn any(self, block_count: usize, cpu: impl Cpu) -> bool {
        match DynCpu::from(cpu) {
            DynCpu::Avx2(cpu) => any_avx2(self.0, block_count, cpu),
            _ => any_generic(self.0, block_count),
        }
    }
}

#[cpu_version(Avx2)]
#[inline]
unsafe fn any_avx2(bm: *const u64, nw: usize, cpu: Avx2) -> bool {
    let mut w = 0usize;
    while w + 4 <= nw {
        let v = Vec256::load_unaligned(bm.add(w).cast::<u8>(), cpu);
        if !v.is_zero() {
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

#[cpu_version(Generic)]
#[inline]
unsafe fn any_generic(bm: *const u64, nw: usize) -> bool {
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
