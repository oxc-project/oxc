use std::mem::MaybeUninit;

use oxc_simd::{BASELINE, cpu_version};
use oxc_span::Span;

use crate::token::{is_trivia_byte, matches_tk, tk};

use crate::pipeline::{bitmap::Bitmap, chunk::eqm, kinds::Kinds, lanes::Lanes};

use super::common::{emit_value, invalid_diags};

#[cpu_version(Generic)]
pub(super) fn compress_blocks_generic(
    token_start: Bitmap,
    kinds: Kinds,
    b0: usize,
    b1: usize,
    starts: &mut [u32],
    stage_kinds: &mut [u8],
) -> usize {
    let cpu = BASELINE;

    let mut m = 0usize;
    for b in b0..b1 {
        let mut w = token_start.block(b, cpu);
        let base = b * 64;
        while w != 0 {
            let pos = base + w.trailing_zeros() as usize;
            w &= w - 1;
            starts[m] = pos as u32;
            stage_kinds[m] = kinds.get(pos);
            m += 1;
        }
    }
    m
}

/// Write the kept staged tokens to every slot below the returned count; also the SIMD tails.
#[cpu_version(Generic)]
pub(super) fn build_spans_generic(
    stage_kind: &[u8],
    stage_pos: &[u32],
    spans: &mut [MaybeUninit<Span>],
    sig_kinds: &mut [MaybeUninit<u8>],
) -> usize {
    let cpu = BASELINE;

    let mut w = 0usize;
    for (j, &k) in stage_kind.iter().enumerate() {
        spans[w].write(Span::new(stage_pos[j], stage_pos[j + 1]));
        sig_kinds[w].write(k);
        w += usize::from(!is_trivia_byte(k, cpu) || k == tk!(Hashbang));
    }
    w
}

#[cpu_version(Generic)]
pub(super) fn lanes_post_generic(
    src: &[u8],
    kinds: &[u8],
    spans: &[Span],
    src_len: u32,
    lanes: &mut Lanes,
) {
    let cpu = BASELINE;

    let m = kinds.len();
    let spans = &spans[..m];
    let mut inv = 0u64;
    let mut i = 0usize;
    while i + 8 <= m {
        let x = u64::from_le_bytes(kinds[i..i + 8].try_into().unwrap());
        let mut hits = eqm(x, tk!(Number))
            | eqm(x, tk!(BigInt))
            | eqm(x, tk!(IdentEscaped))
            | eqm(x, tk!(PrivateIdentEscaped));
        inv |= eqm(x, tk!(Invalid));
        while hits != 0 {
            emit_value(src, kinds, spans, i + (hits.trailing_zeros() >> 3) as usize, lanes, cpu);
            hits &= hits - 1;
        }
        i += 8;
    }
    let mut inv_dirty = inv != 0;
    while i < m {
        let k = kinds[i];
        if matches_tk!(k, Number | BigInt | IdentEscaped | PrivateIdentEscaped) {
            emit_value(src, kinds, spans, i, lanes, cpu);
        }
        inv_dirty |= k == tk!(Invalid);
        i += 1;
    }
    if inv_dirty {
        invalid_diags(src, kinds, spans, src_len, lanes);
    }
}
