//! Prefetching memory into the cache.

// These functions must be inlined to get the `target_feature`s of the functions calling them,
// otherwise the intrinsics also can't be inlined into them - 2 function calls for each
#![expect(clippy::inline_always)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};

use crate::{
    cpu::{Avx2, Cpu, DynCpu},
    cpu_multiversion, cpu_version,
};

/// Ask the CPU to fetch the cache line containing `ptr` into all levels of the cache.
///
/// Only a hint. Only AVX2 has an implementation. On other CPUs, it does nothing.
#[cpu_multiversion]
#[inline(always)]
pub fn prefetch(ptr: *const u8, cpu: impl Cpu) {
    if let DynCpu::Avx2(cpu) = DynCpu::from(cpu) {
        prefetch_avx2(ptr, cpu);
    }
}

/// AVX2 version of [`prefetch`].
#[cpu_version(Avx2)]
#[inline(always)]
fn prefetch_avx2(ptr: *const u8, _cpu: Avx2) {
    let _ = ptr; // Avoid unused vars lint warning

    // SAFETY: `Avx2` type guarantees CPU supports AVX2
    #[cfg(target_arch = "x86_64")]
    unsafe {
        _mm_prefetch(ptr.cast::<i8>(), _MM_HINT_T0);
    }
}
