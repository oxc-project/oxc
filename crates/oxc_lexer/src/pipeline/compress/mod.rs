use oxc_span::Span;

use crate::{lanes::Lanes, tables::Tables};

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
mod avx2;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use avx2::{build_spans, compress_blocks, lanes_post};

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
mod generic;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
use generic::{build_spans, compress_blocks, lanes_post};

mod common;
pub use common::write_sentinels;

const STAGE_BLOCKS: usize = 32;
pub const STAGE_CAP: usize = STAGE_BLOCKS * 64 + 128;

pub unsafe fn compress(
    t: &Tables,
    src: &[u8],
    n: usize,
    nb: usize,
    st: *const u64,
    kind: *const u8,
    stage_pos: *mut u32,
    stage_kind: *mut u8,
    out_kinds: *mut u8,
    out_spans: *mut Span,
    lanes: &mut Lanes,
) -> usize {
    let mut c = 0usize;
    let mut w = 0usize;
    let mut b = 0usize;
    while b < nb {
        let b1 = (b + STAGE_BLOCKS).min(nb);
        c += compress_blocks(t, st, kind, b, b1, stage_pos.add(c), stage_kind.add(c));
        b = b1;
        if c > 1 {
            w += build_spans(stage_kind, stage_pos, c - 1, out_spans.add(w), out_kinds.add(w));
            *stage_pos = *stage_pos.add(c - 1);
            *stage_kind = *stage_kind.add(c - 1);
            c = 1;
        }
    }
    if c == 1 {
        *stage_pos.add(1) = n as u32;
        w += build_spans(stage_kind, stage_pos, 1, out_spans.add(w), out_kinds.add(w));
    }
    write_sentinels(n as u32, out_spans.add(w), out_kinds.add(w));
    lanes_post(src, out_kinds, out_spans, w, n as u32, lanes);
    w
}

#[cfg(test)]
mod tests;
