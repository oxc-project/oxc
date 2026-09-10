use oxc_span::Span;

use crate::{lanes::Lanes, tables::Tables, token::is_trivia_byte};

use super::super::{
    BIGINT, HASHBANG, IDENT_ESC, NUM, PRIV_IDENT_ESC,
    chunk::{eqm, load64},
};

use super::common::{emit_value, invalid_diags};

pub(super) unsafe fn compress_blocks(
    _t: &Tables,
    st: *const u64,
    kind: *const u8,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    kinds: *mut u8,
) -> usize {
    let mut m = 0usize;
    for b in b0..b1 {
        let mut w = *st.add(b);
        if w == 0 {
            continue;
        }
        let base = (b * 64) as u32;
        while w != 0 {
            let bit = w.trailing_zeros();
            w &= w - 1;
            *starts.add(m) = base + bit;
            *kinds.add(m) = *kind.add((base + bit) as usize);
            m += 1;
        }
    }
    m
}

/// Contents of this function is in common with AVX2 implementation.
pub(super) unsafe fn build_spans(
    stage_kind: *const u8,
    stage_pos: *const u32,
    m: usize,
    spans: *mut Span,
    sig_kinds: *mut u8,
) -> usize {
    let sp = spans.cast::<u64>();
    let mut w = 0usize;
    let mut j = 0usize;

    while j < m {
        let k = *stage_kind.add(j);
        *sp.add(w) = stage_pos.add(j).cast::<u64>().read_unaligned();
        *sig_kinds.add(w) = k;
        w += usize::from(!is_trivia_byte(k) || k == HASHBANG);
        j += 1;
    }

    w
}

pub(super) unsafe fn lanes_post(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
) {
    let mut inv = 0u64;
    let mut i = 0usize;
    while i + 8 <= m {
        let x = load64(out_kinds, i);
        let mut hits = eqm(x, NUM) | eqm(x, BIGINT) | eqm(x, IDENT_ESC) | eqm(x, PRIV_IDENT_ESC);
        inv |= eqm(x, 255);
        while hits != 0 {
            emit_value(src, out_kinds, out_spans, i + (hits.trailing_zeros() >> 3) as usize, lanes);
            hits &= hits - 1;
        }
        i += 8;
    }
    let mut inv_dirty = inv != 0;
    while i < m {
        let k = *out_kinds.add(i);
        if k == NUM || k == BIGINT || k == IDENT_ESC || k == PRIV_IDENT_ESC {
            emit_value(src, out_kinds, out_spans, i, lanes);
        }
        inv_dirty |= k == 255;
        i += 1;
    }
    if inv_dirty {
        invalid_diags(src, out_kinds, out_spans, m, nn, lanes);
    }
}
