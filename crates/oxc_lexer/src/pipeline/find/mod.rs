use crate::pipeline::cpu::{Cpu, cpu_select};

#[cfg(target_arch = "x86_64")]
mod avx2;
#[cfg(target_arch = "x86_64")]
use avx2::{bracket_bits_avx2, find_avx2};

mod generic;
use generic::{bracket_bits_generic, find_generic};

/// First byte at or after `i` that is one of `needles`, or `n`.
///
/// Needle order matters for speed. See the comment on `any_of` in `avx2.rs`.
#[inline(always)]
pub(super) unsafe fn find<const K: usize>(
    src: *const u8,
    n: usize,
    i: usize,
    needles: [u8; K],
    cpu: impl Cpu,
) -> usize {
    cpu_select!(cpu, {
        Avx2(cpu) => find_avx2(src, n, i, needles, cpu),
        Neon(_) => find_generic(src, n, i, needles),
        Generic(_) => find_generic(src, n, i, needles),
    })
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`). Bit `i` is for byte `base + i`.
#[inline(always)]
pub(super) fn bracket_bits(src: &[u8], base: usize, cpu: impl Cpu) -> u64 {
    cpu_select!(cpu, {
        Avx2(cpu) => {
            // SAFETY: `Avx2` type guarantees required target features are present
            unsafe { bracket_bits_avx2(src, base, cpu) }
        }
        Neon(_) => bracket_bits_generic(src, base),
        Generic(_) => bracket_bits_generic(src, base),
    })
}
