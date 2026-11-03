//! Prefetching memory into the cache.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};

use crate::cpu::{Cpu, cpu_multiversion, cpu_select};

/// Ask the CPU to fetch the cache line containing `p` into all levels of the cache.
///
/// Only a hint. Only AVX2 has an implementation. On other CPUs, it does nothing.
#[cpu_multiversion]
#[expect(clippy::inline_always)]
#[inline(always)]
pub fn prefetch(p: *const u8, cpu: impl Cpu) {
    let _ = p; // Avoid clippy unused vars warning

    cpu_select!(cpu, {
        Avx2(_) => {
            cfg_select! {
                target_arch = "x86_64" => {
                    // SAFETY: `Avx2` type guarantees CPU supports AVX2
                    unsafe { _mm_prefetch(p.cast::<i8>(), _MM_HINT_T0) }
                }
                _ => {}
            }
        }
        Neon(_) => { /* no-op */ }
        Generic(_) => { /* no-op */ }
    });
}
