use oxc_span::Span;

use crate::{error::diag_code, lanes::Lanes, token::SPAN_SENTINELS};

use super::super::{EOF, IDENT_ESC};

pub unsafe fn write_sentinels(n: u32, spans: *mut Span, sig_kinds: *mut u8) {
    let eof = u64::from(n) | (u64::from(n) << 32);
    for s in 0..SPAN_SENTINELS {
        *spans.cast::<u64>().add(s) = eof;
        *sig_kinds.add(s) = EOF;
    }
}

#[inline(always)]
pub(super) unsafe fn emit_value(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    j: usize,
    lanes: &mut Lanes,
) {
    let sp = *out_spans.add(j);
    let s = sp.start as usize;
    let e = sp.end as usize;
    let k = *out_kinds.add(j);
    if k < IDENT_ESC {
        lanes.push_number_swar(src, s, e);
    } else if k == IDENT_ESC {
        lanes.push_atom(src, s, e);
    } else {
        lanes.push_atom(src, s + 1, e);
    }
}

#[cold]
pub(super) unsafe fn invalid_diags(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
) {
    let char_after = |cs: u32| -> u32 {
        if cs >= nn {
            return 0;
        }
        let b = src[cs as usize];
        let l: u32 = if b < 0x80 {
            1
        } else if b >= 0xF0 {
            4
        } else if b >= 0xE0 {
            3
        } else if b >= 0xC0 {
            2
        } else {
            1
        };
        l.min(nn - cs)
    };
    for j in 0..m {
        if *out_kinds.add(j) == 255 {
            let sp = *out_spans.add(j);
            let (s, e) = (sp.start, sp.end);
            let b0 = src[s as usize];
            if b0 == b'\\' {
                lanes.push_diag(s + 1, char_after(s + 1), diag_code::INVALID_IDENTIFIER_ESCAPE);
            } else if b0 == b'#' {
                lanes.push_diag(s + 1, char_after(s + 1), diag_code::UNEXPECTED_CHARACTER);
            } else {
                lanes.push_diag(s, e - s, diag_code::UNEXPECTED_CHARACTER);
            }
        }
    }
}
