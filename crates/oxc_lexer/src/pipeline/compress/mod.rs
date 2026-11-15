use std::{mem::MaybeUninit, slice};

use oxc_simd::{Cpu, DynCpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{
    PAD,
    token::{SPAN_SENTINELS, TokenKind, debug_assert_kind_bytes, tk},
};

use crate::pipeline::{bitmap::Bitmap, kinds::Kinds, lanes::Lanes, lex::Lex};

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
    let spans = out_spans.spare_capacity_mut();
    let sig_kinds = kind_slots(out_kinds.spare_capacity_mut());
    let mut c = 0usize;
    let mut w = 0usize;
    let mut b = 0usize;
    let block_count = src.block_count();
    while b < block_count {
        let b1 = (b + STAGE_BLOCKS).min(block_count);
        let (pos, kind) = (&mut stage_pos[c..], &mut stage_kind[c..]);
        c += compress_blocks(bitmaps.token_start, kinds, b, b1, pos, kind, cpu);
        b = b1;
        if c > 1 {
            let (kind, pos) = (&stage_kind[..c - 1], &stage_pos[..c]);
            w += build_spans(kind, pos, &mut spans[w..], &mut sig_kinds[w..], cpu);
            stage_pos[0] = stage_pos[c - 1];
            stage_kind[0] = stage_kind[c - 1];
            c = 1;
        }
    }
    if c == 1 {
        stage_pos[1] = src.len() as u32;
        let (kind, pos) = (&stage_kind[..1], &stage_pos[..2]);
        w += build_spans(kind, pos, &mut spans[w..], &mut sig_kinds[w..], cpu);
    }
    let len = w + SPAN_SENTINELS;
    let eof = Span::new(src.len() as u32, src.len() as u32);
    for (span, kind) in spans[w..len].iter_mut().zip(&mut sig_kinds[w..len]) {
        span.write(eof);
        kind.write(tk!(Eof));
    }
    // SAFETY: build_spans filled each slot below w with a Kinds byte, and the loop the sentinels.
    unsafe {
        debug_assert_kind_bytes(slice::from_raw_parts(out_kinds.as_ptr().cast::<u8>(), len));
        out_kinds.set_len(len);
        out_spans.set_len(len);
    }
    w
}

/// The kind slots as bytes, for the kernels to fill from the staged kinds.
fn kind_slots(slots: &mut [MaybeUninit<TokenKind>]) -> &mut [MaybeUninit<u8>] {
    // SAFETY: TokenKind is repr(u8), and a MaybeUninit byte may hold any value.
    unsafe { slice::from_raw_parts_mut(slots.as_mut_ptr().cast(), slots.len()) }
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

/// Stage the token starts and kinds of blocks b0..b1, at most 64 a block; returns the count.
#[cpu_multiversion]
#[inline(always)]
fn compress_blocks(
    token_start: Bitmap,
    kinds: Kinds,
    b0: usize,
    b1: usize,
    starts: &mut [u32],
    stage_kinds: &mut [u8],
    cpu: impl Cpu,
) -> usize {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => {
            compress_blocks_avx2(token_start, kinds, b0, b1, starts, stage_kinds, cpu)
        }
        DynCpu::Neon(cpu) => {
            compress_blocks_neon(token_start, kinds, b0, b1, starts, stage_kinds, cpu)
        }
        DynCpu::Generic(_) => {
            compress_blocks_generic(token_start, kinds, b0, b1, starts, stage_kinds)
        }
    }
}

/// Write the kept staged tokens to every slot below the returned count, which compress relies on.
#[cpu_multiversion]
#[inline(always)]
fn build_spans(
    stage_kind: &[u8],
    stage_pos: &[u32],
    spans: &mut [MaybeUninit<Span>],
    sig_kinds: &mut [MaybeUninit<u8>],
    cpu: impl Cpu,
) -> usize {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => build_spans_avx2(stage_kind, stage_pos, spans, sig_kinds, cpu),
        DynCpu::Neon(cpu) => build_spans_neon(stage_kind, stage_pos, spans, sig_kinds, cpu),
        DynCpu::Generic(_) => build_spans_generic(stage_kind, stage_pos, spans, sig_kinds),
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
