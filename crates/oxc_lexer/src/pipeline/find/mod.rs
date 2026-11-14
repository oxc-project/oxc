// Raw kernels: the unsafe fn boundary is the reviewed surface, not each operation inside.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]

use oxc_simd::{Cpu, DynCpu, cpu_multiversion};

mod avx2;
mod generic;
mod neon;

use avx2::{bracket_bits_avx2, find_avx2};
use generic::{bracket_bits_generic, find_generic};
use neon::{bracket_bits_neon, find_neon};

use crate::pipeline::source::Source;

/// Ordered for speed, pairing `(` with `)`, `[` with `{` and `]` with `}`.
/// See the comment on `any_of` in `avx2.rs`.
const BRACKET_BYTES: [u8; 6] = [b'(', b')', b'[', b'{', b']', b'}'];

/// First byte at or after `i` that is one of `needles`, or `n`.
///
/// Needle order matters for speed. See the comment on `any_of` in `avx2.rs`.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn find<const K: usize>(
    src: Source,
    i: usize,
    needles: [u8; K],
    cpu: impl Cpu,
) -> usize {
    if i > src.len() {
        return src.len();
    }
    // SAFETY: the finders read only src[i..src.len()], which src holds.
    unsafe {
        match DynCpu::from(cpu) {
            DynCpu::Avx2(cpu) => find_avx2(src.as_ptr(), src.len(), i, needles, cpu),
            DynCpu::Neon(cpu) => find_neon(src.as_ptr(), src.len(), i, needles, cpu),
            DynCpu::Generic(_) => find_generic(src.as_ptr(), src.len(), i, needles),
        }
    }
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`). Bit `i` is for byte `base + i`.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn bracket_bits(src: &[u8], base: usize, cpu: impl Cpu) -> u64 {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => bracket_bits_avx2(src, base, cpu),
        DynCpu::Neon(cpu) => bracket_bits_neon(src, base, cpu),
        DynCpu::Generic(_) => bracket_bits_generic(src, base),
    }
}
