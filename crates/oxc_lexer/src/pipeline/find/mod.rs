use crate::cpu::{Cpu, DynCpu, cpu_multiversion, to_dyn};

mod avx2;
use avx2::{bracket_bits_avx2, find_avx2};

mod generic;
use generic::{bracket_bits_generic, find_generic};

/// First byte at or after `i` that is one of `needles`, or `n`.
///
/// Needle order matters for speed. See the comment on `any_of` in `avx2.rs`.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn find<const K: usize>(
    src: *const u8,
    n: usize,
    i: usize,
    needles: [u8; K],
    cpu: impl Cpu,
) -> usize {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => find_avx2(src, n, i, needles, cpu),
        _ => find_generic(src, n, i, needles),
    }
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`). Bit `i` is for byte `base + i`.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn bracket_bits(src: &[u8], base: usize, cpu: impl Cpu) -> u64 {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => bracket_bits_avx2(src, base, cpu),
        _ => bracket_bits_generic(src, base),
    }
}
