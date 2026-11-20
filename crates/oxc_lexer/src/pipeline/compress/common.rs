use oxc_simd::{Cpu, VecData128, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{error::DiagCode, token::tk};

use crate::pipeline::lanes::Lanes;

/// For each 8-bit "keep" mask, the indexes of its set bits in ascending order, padded with `0x80`.
///
/// Used as a byte shuffle control, to squeeze 8 staged kinds down to the ones which survive.
/// `0x80` zeroes its lane on every CPU, and the next group's store lands on top of those lanes.
pub(super) static BCOMPACT: [VecData128; 256] = {
    let mut t = [VecData128::from_bytes([0x80; 16]); 256];
    let mut mask = 0usize;
    while mask < 256 {
        let (mut b, mut k) = (0usize, 0usize);
        while b < 8 {
            if mask & (1 << b) != 0 {
                t[mask].as_bytes_mut()[k] = b as u8;
                k += 1;
            }
            b += 1;
        }
        mask += 1;
    }
    t
};

#[cpu_multiversion]
#[inline(always)]
pub(super) fn emit_value(
    src: &[u8],
    kinds: &[u8],
    spans: &[Span],
    j: usize,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    let sp = spans[j];
    let s = sp.start as usize;
    let e = sp.end as usize;
    let k = kinds[j];
    if k < tk!(IdentEscaped) {
        lanes.push_number(src, s, e, cpu);
    } else if k == tk!(IdentEscaped) {
        lanes.push_atom(src, s, e, cpu);
    } else {
        lanes.push_atom(src, s + 1, e, cpu);
    }
}

/// Push a diagnostic for each `Invalid` token.
///
/// As it's cold code, it's compiled only once, not separately for each CPU feature set,
/// which keeps the binary smaller. `#[inline(never)]` stops it being inlined into
/// the separate copies of its callers.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
pub(super) fn invalid_diags(
    src: &[u8],
    kinds: &[u8],
    spans: &[Span],
    src_len: u32,
    lanes: &mut Lanes,
) {
    let char_after = |cs: u32| -> u32 {
        if cs >= src_len {
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
        l.min(src_len - cs)
    };
    for (&k, sp) in kinds.iter().zip(spans) {
        if k == tk!(Invalid) {
            let (s, e) = (sp.start, sp.end);
            let b0 = src[s as usize];
            if b0 == b'\\' {
                lanes.push_diag(s + 1, char_after(s + 1), DiagCode::InvalidIdentifierEscape);
            } else if b0 == b'#' {
                lanes.push_diag(s + 1, char_after(s + 1), DiagCode::UnexpectedCharacter);
            } else {
                lanes.push_diag(s, e - s, DiagCode::UnexpectedCharacter);
            }
        }
    }
}
