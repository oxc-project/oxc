use std::slice;

use oxc_simd::{Cpu, DynCpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{
    PAD,
    token::{SPAN_SENTINELS, TokenKind, debug_assert_kind_bytes, tk},
};

use crate::pipeline::{bitmap::Bitmap, kinds::Kinds, lanes::Lanes, lex::Lex, source::Source};

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

/// Fill kinds and spans with the significant tokens, then the sentinels; returns the token count.
#[cpu_multiversion]
#[inline(never)]
pub fn compress(
    lex: Lex,
    stage_pos: &mut [u32; STAGE_CAP],
    stage_kind: &mut [u8; STAGE_CAP],
    out_kinds: &mut Vec<TokenKind>,
    out_spans: &mut Vec<Span>,
    cpu: impl Cpu,
) -> usize {
    let Lex { src, bitmaps, kinds, .. } = lex;
    // Can't wrap: n + PAD fits in the source slice, and SPAN_SENTINELS is small.
    let capacity = src.len() + PAD + SPAN_SENTINELS;
    out_kinds.clear();
    out_kinds.reserve(capacity);
    out_spans.clear();
    out_spans.reserve(capacity);
    let blocks = src.block_count();
    let (token_start, kinds) = (bitmaps.token_start.first(blocks), kinds.first(blocks * 64));
    let (stage_pos, stage_kind) = (stage_pos.as_mut_ptr(), stage_kind.as_mut_ptr());
    let (out, spans) = (out_kinds.as_mut_ptr().cast::<u8>(), out_spans.as_mut_ptr());
    // SAFETY: rounds fit STAGE_CAP, outputs fit the capacity, and every kind byte is a TokenKind.
    unsafe {
        let w = compress_raw(src, token_start, kinds, stage_pos, stage_kind, out, spans, cpu);
        let len = w + SPAN_SENTINELS;
        debug_assert_kind_bytes(slice::from_raw_parts(out, len));
        out_kinds.set_len(len);
        out_spans.set_len(len);
        w
    }
}

/// The loop of compress: handles over src's blocks, a STAGE_CAP stage, n + PAD + 8 outputs.
#[cpu_multiversion]
#[inline(always)]
unsafe fn compress_raw(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    stage_pos: *mut u32,
    stage_kind: *mut u8,
    out_kinds: *mut u8,
    out_spans: *mut Span,
    cpu: impl Cpu,
) -> usize {
    let mut c = 0usize;
    let mut w = 0usize;
    let mut b = 0usize;
    let block_count = src.block_count();
    while b < block_count {
        let b1 = (b + STAGE_BLOCKS).min(block_count);
        c += compress_blocks(token_start, kinds, b, b1, stage_pos.add(c), stage_kind.add(c), cpu);
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
    w
}

#[cpu_multiversion]
unsafe fn write_sentinels(n: u32, spans: *mut Span, sig_kinds: *mut u8, _cpu: impl Cpu) {
    let eof = u64::from(n) | (u64::from(n) << 32);
    for s in 0..SPAN_SENTINELS {
        *spans.cast::<u64>().add(s) = eof;
        *sig_kinds.add(s) = tk!(Eof);
    }
}

/// Initialize the global `PairLuts` instance.
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
    token_start: Bitmap,
    kinds: Kinds,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    stage_kinds: *mut u8,
    cpu: impl Cpu,
) -> usize {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => compress_blocks_avx2(
            token_start.as_ptr(),
            kinds.as_ptr(),
            b0,
            b1,
            starts,
            stage_kinds,
            cpu,
        ),
        DynCpu::Neon(cpu) => compress_blocks_neon(
            token_start.as_ptr(),
            kinds.as_ptr(),
            b0,
            b1,
            starts,
            stage_kinds,
            cpu,
        ),
        DynCpu::Generic(_) => compress_blocks_generic(
            token_start.as_ptr(),
            kinds.as_ptr(),
            b0,
            b1,
            starts,
            stage_kinds,
        ),
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

/// Push the values of the number and escaped identifier tokens, and flag the invalid ones.
#[cpu_multiversion]
#[inline(never)]
pub fn lanes_post(
    src: &[u8],
    kinds: &[u8],
    spans: &[Span],
    src_len: u32,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => lanes_post_avx2(src, kinds, spans, src_len, lanes, cpu),
        DynCpu::Neon(cpu) => lanes_post_neon(src, kinds, spans, src_len, lanes, cpu),
        DynCpu::Generic(_) => lanes_post_generic(src, kinds, spans, src_len, lanes),
    };
}
