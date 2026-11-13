use oxc_simd::{Cpu, DynCpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::token::{SPAN_SENTINELS, tk};

use crate::pipeline::{bitmap::Bitmap, lanes::Lanes, source::Source};

mod avx2;
mod common;
mod generic;
mod neon;
mod pair_luts;

#[cfg(test)]
mod tests;

use avx2::{build_spans_avx2, compress_blocks_avx2, lanes_post_avx2};
use generic::{build_spans_generic, compress_blocks_generic, lanes_post_generic};
use neon::{build_spans_neon, compress_blocks_neon, lanes_post_neon};

const STAGE_BLOCKS: usize = 32;
pub const STAGE_CAP: usize = STAGE_BLOCKS * 64 + 128;

#[cpu_multiversion]
#[inline(never)]
pub unsafe fn compress(
    src: Source,
    nb: usize,
    st: Bitmap,
    kind: *const u8,
    stage_pos: *mut u32,
    stage_kind: *mut u8,
    out_kinds: *mut u8,
    out_spans: *mut Span,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let mut c = 0usize;
    let mut w = 0usize;
    let mut b = 0usize;
    while b < nb {
        let b1 = (b + STAGE_BLOCKS).min(nb);
        c += compress_blocks(st, kind, b, b1, stage_pos.add(c), stage_kind.add(c), cpu);
        b = b1;
        if c > 1 {
            w += build_spans(stage_kind, stage_pos, c - 1, out_spans.add(w), out_kinds.add(w), cpu);
            *stage_pos = *stage_pos.add(c - 1);
            *stage_kind = *stage_kind.add(c - 1);
            c = 1;
        }
    }
    if c == 1 {
        *stage_pos.add(1) = src.len() as u32;
        w += build_spans(stage_kind, stage_pos, 1, out_spans.add(w), out_kinds.add(w), cpu);
    }
    write_sentinels(src.len() as u32, out_spans.add(w), out_kinds.add(w), cpu);
    lanes_post(src.padded(), out_kinds, out_spans, w, src.len() as u32, lanes, cpu);
    w
}

#[cpu_multiversion]
pub unsafe fn write_sentinels(n: u32, spans: *mut Span, sig_kinds: *mut u8, _cpu: impl Cpu) {
    let eof = u64::from(n) | (u64::from(n) << 32);
    for s in 0..SPAN_SENTINELS {
        *spans.cast::<u64>().add(s) = eof;
        *sig_kinds.add(s) = tk!(Eof);
    }
}

/// Initialize the global `PairLuts` instance.
///
/// This function must be called before calling [`compress`].
///
/// `PairLuts` is only used by the SIMD implementations, so this function is a no-op on other CPUs.
#[cpu_version(Generic)]
pub(super) fn init_pair_luts(cpu: DynCpu) {
    if matches!(cpu, DynCpu::Avx2(_) | DynCpu::Neon(_)) {
        pair_luts::init_pair_luts();
    }
}

#[cpu_multiversion]
#[inline(always)]
unsafe fn compress_blocks(
    st: Bitmap,
    kind: *const u8,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    kinds: *mut u8,
    cpu: impl Cpu,
) -> usize {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => compress_blocks_avx2(st.as_ptr(), kind, b0, b1, starts, kinds, cpu),
        DynCpu::Neon(cpu) => compress_blocks_neon(st.as_ptr(), kind, b0, b1, starts, kinds, cpu),
        DynCpu::Generic(_) => compress_blocks_generic(st.as_ptr(), kind, b0, b1, starts, kinds),
    }
}

#[cpu_multiversion]
#[inline(always)]
unsafe fn build_spans(
    stage_kind: *const u8,
    stage_pos: *const u32,
    m: usize,
    spans: *mut Span,
    sig_kinds: *mut u8,
    cpu: impl Cpu,
) -> usize {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => build_spans_avx2(stage_kind, stage_pos, m, spans, sig_kinds, cpu),
        DynCpu::Neon(cpu) => build_spans_neon(stage_kind, stage_pos, m, spans, sig_kinds, cpu),
        DynCpu::Generic(_) => build_spans_generic(stage_kind, stage_pos, m, spans, sig_kinds),
    }
}

#[cpu_multiversion]
#[inline(always)]
unsafe fn lanes_post(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => lanes_post_avx2(src, out_kinds, out_spans, m, nn, lanes, cpu),
        DynCpu::Neon(cpu) => lanes_post_neon(src, out_kinds, out_spans, m, nn, lanes, cpu),
        DynCpu::Generic(_) => lanes_post_generic(src, out_kinds, out_spans, m, nn, lanes),
    };
}
