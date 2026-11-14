use oxc_simd::{Neon, Vec128, Vec256, VecData128, cpu_version};
use oxc_span::Span;

use crate::token::{TRIVIA_MAX, TRIVIA_MIN, is_trivia_byte, tk};

use crate::pipeline::{
    chunk::{LANE_BITS, mm32, mm64},
    lanes::Lanes,
};

use super::{
    common::{BCOMPACT, emit_value, invalid_diags},
    pair_luts::get_pair_luts,
};

/// For each 4-bit "keep" mask, the shuffle controls which gather the kept spans out of
/// a window of 4 spans: the first 2 kept spans, then the next 2.
///
/// A `Span` is 8 bytes, so span `k` of the window is bytes `8 * k` to `8 * k + 7`.
/// Unused lanes are `0x80`, which zeroes them, and the next group's store lands on top of them.
static SPAN_COMPACT: [[VecData128; 2]; 16] = {
    let mut t = [[VecData128::from_bytes([0x80; 16]); 2]; 16];
    let mut mask = 0usize;
    while mask < 16 {
        let (mut span, mut kept) = (0usize, 0usize);
        while span < 4 {
            if mask & (1 << span) != 0 {
                let mut byte = 0usize;
                while byte < 8 {
                    t[mask][kept / 2].as_bytes_mut()[(kept % 2) * 8 + byte] =
                        (span * 8 + byte) as u8;
                    byte += 1;
                }
                kept += 1;
            }
            span += 1;
        }
        mask += 1;
    }
    t
};

/// Turn the token-start bitmap into a list of token positions and kinds, 16 bits at a time.
///
/// Same as the AVX2 implementation. A `0x80` control byte zeroes its lane on Neon too.
#[cpu_version(Neon)]
#[inline(never)]
pub(super) unsafe fn compress_blocks_neon(
    st: *const u64,
    kind: *const u8,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    kinds: *mut u8,
    cpu: Neon,
) -> usize {
    // SAFETY: `Lexer::new` calls `init_pair_luts`, which initializes `PairLuts` if the CPU supports Neon
    let pair_luts = unsafe { get_pair_luts() };
    let lut0z = &pair_luts.lut0z;
    let lutpad = pair_luts.lutpad.as_ptr().cast::<u8>();

    let mut m = 0usize;
    for b in b0..b1 {
        let mword = *st.add(b);
        if mword == 0 {
            continue;
        }

        // Byte popcounts and running sums in general registers, avoiding vector round trips
        let x = mword - ((mword >> 1) & 0x5555_5555_5555_5555);
        let x = (x & 0x3333_3333_3333_3333) + ((x >> 2) & 0x3333_3333_3333_3333);
        let bc = (x + (x >> 4)) & 0x0f0f_0f0f_0f0f_0f0f;
        let sums = bc.wrapping_mul(0x0101_0101_0101_0101);
        let o0 = 0;
        let o1 = ((sums >> 8) & 0xff) as u32;
        let o2 = ((sums >> 24) & 0xff) as u32;
        let o3 = ((sums >> 40) & 0xff) as u32;
        let o4 = (sums >> 56) as u32;

        let base = b * 64;
        let mut basev = Vec256::splat_u32(base as u32, cpu);

        let ka =
            compress_pair(0, mword, bc, o0, lut0z, lutpad, kind, base, starts, m, &mut basev, cpu);
        let kb =
            compress_pair(1, mword, bc, o1, lut0z, lutpad, kind, base, starts, m, &mut basev, cpu);
        ka.store_unaligned(kinds.add(m + o0 as usize));
        kb.store_unaligned(kinds.add(m + o1 as usize));
        let kc =
            compress_pair(2, mword, bc, o2, lut0z, lutpad, kind, base, starts, m, &mut basev, cpu);
        let kd =
            compress_pair(3, mword, bc, o3, lut0z, lutpad, kind, base, starts, m, &mut basev, cpu);
        kc.store_unaligned(kinds.add(m + o2 as usize));
        kd.store_unaligned(kinds.add(m + o3 as usize));

        m += o4 as usize;
    }
    m
}

/// Compress 16 bits of a block's token-start mask `mword`: bits `16 * p` to `16 * p + 15`.
///
/// Writes the positions of the tokens which start at those bits to `starts`,
/// from index `m + off`, and returns their kinds, gathered into the low bytes of a [`Vec128`].
///
/// Each lane of `basev` holds the position of bit `16 * p`.
/// Adds 16 to it, ready for the next 16 bits.
///
/// # SAFETY
///
/// - `kind` must be valid for reading 16 bytes at `base + 16 * p`.
/// - `starts` must be valid for writing 16 `u32`s at `m + off`.
/// - `lutpad` must point to the `lutpad` table of `PairLuts`.
#[cpu_version(Neon)]
#[inline(always)]
unsafe fn compress_pair(
    p: usize,
    mword: u64,
    bc: u64,
    off: u32,
    lut0z: &[[u8; 8]; 256],
    lutpad: *const u8,
    kind: *const u8,
    base: usize,
    starts: *mut u32,
    m: usize,
    basev: &mut Vec256<Neon>,
    cpu: Neon,
) -> Vec128<Neon> {
    let sub0 = ((mword >> (16 * p)) & 0xff) as usize;
    let sub1 = ((mword >> (16 * p + 8)) & 0xff) as usize;
    let pc0 = ((bc >> (16 * p)) & 0xff) as usize;
    let row0 = Vec128::read_low_unaligned(&lut0z[sub0], cpu);
    let row1 = Vec128::load_unaligned(lutpad.add(sub1 * 32 + 8 - pc0), cpu);
    let ctrl = row0 | row1;
    let kw = Vec128::load_unaligned(kind.add(base + 16 * p), cpu);
    // Lanes past this group's token count are junk positions, which the next group's store covers.
    let lo = basev.add_u32(Vec256::widen_u8_to_u32(ctrl));
    lo.store_unaligned(starts.add(m + off as usize).cast::<u8>());
    let hi = basev.add_u32(Vec256::widen_u8_to_u32(ctrl.shr_bytes::<8>()));
    hi.store_unaligned(starts.add(m + off as usize + 8).cast::<u8>());
    *basev = basev.add_u32(16);
    kw.shuffle_u8s(ctrl)
}

/// Write the staged `(pos, kind)` pairs out as spans, dropping trivia, 8 at a time.
///
/// A token's span runs to the next token's start, so the spans are the overlapping `u32` pairs
/// of `stage_pos`, which interleaving it with itself shifted by 1 builds.
#[cpu_version(Neon)]
pub(super) unsafe fn build_spans_neon(
    stage_kind: *const u8,
    stage_pos: *const u32,
    m: usize,
    spans: *mut Span,
    sig_kinds: *mut u8,
    cpu: Neon,
) -> usize {
    let sp = spans.cast::<u64>();
    let mut w = 0usize;
    let mut j = 0usize;

    while j + 8 <= m {
        let k8 = Vec128::load_low_unaligned(stage_kind.add(j), cpu);
        let triv = k8.sub_u8(TRIVIA_MIN).saturating_sub_u8(TRIVIA_MAX - TRIVIA_MIN).eq_u8(0);
        // The 8 lanes past the kinds are zero, which is never dropped
        let drop = triv & !k8.eq_u8(tk!(Hashbang));
        let sig = usize::from(!(drop & LANE_BITS.read(cpu)).sum_u8());

        // Minified input stages almost no trivia, so this is the common case
        if sig == 0xff {
            for g in [0, 4] {
                let (v0, v1) = span_window(stage_pos.add(j + g), cpu);
                v0.unpack_lo_u32(v1).store_unaligned(sp.add(w + g).cast::<u8>());
                v0.unpack_hi_u32(v1).store_unaligned(sp.add(w + g + 2).cast::<u8>());
            }
            k8.store_low_unaligned(sig_kinds.add(w));
            w += 8;
        } else {
            // Popcounts of each 4 bits from a constant, avoiding a vector round trip
            let (lo4, hi4) = (sig & 0xf, sig >> 4);
            let lo_count = ((0x4332_3221_3221_2110u64 >> (lo4 * 4)) & 0xf) as usize;
            let hi_count = ((0x4332_3221_3221_2110u64 >> (hi4 * 4)) & 0xf) as usize;
            compact_spans(sp.add(w), stage_pos.add(j), lo4, cpu);
            compact_spans(sp.add(w + lo_count), stage_pos.add(j + 4), hi4, cpu);
            k8.shuffle_u8s(BCOMPACT[sig].read(cpu)).store_low_unaligned(sig_kinds.add(w));
            w += lo_count + hi_count;
        }
        j += 8;
    }

    // Same code as generic implementation
    while j < m {
        let k = *stage_kind.add(j);
        *sp.add(w) = stage_pos.add(j).cast::<u64>().read_unaligned();
        *sig_kinds.add(w) = k;
        w += usize::from(!is_trivia_byte(k, cpu) || k == tk!(Hashbang));
        j += 1;
    }

    w
}

/// Load the 4 `u32`s at `pos`, and the 4 at `pos + 1`.
///
/// Interleaving them gives the 4 spans which start at `pos`.
#[cpu_version(Neon)]
#[inline(always)]
unsafe fn span_window(pos: *const u32, cpu: Neon) -> (Vec128<Neon>, Vec128<Neon>) {
    let v0 = Vec128::load_unaligned(pos.cast::<u8>(), cpu);
    let v1 = Vec128::load_unaligned(pos.add(1).cast::<u8>(), cpu);
    (v0, v1)
}

/// Write the spans of the 4 starting at `pos` which `mask` keeps, to `dst`.
///
/// Always writes 4 spans. Those past `mask`'s population count are zero, and the next write
/// lands on top of them, which the `PAD` on the caller's token buffers covers.
#[cpu_version(Neon)]
#[inline(always)]
unsafe fn compact_spans(dst: *mut u64, pos: *const u32, mask: usize, cpu: Neon) {
    let (v0, v1) = span_window(pos, cpu);
    let window = Vec256::from_halves(v0.unpack_lo_u32(v1), v0.unpack_hi_u32(v1));
    let [lo, hi] = &SPAN_COMPACT[mask];
    window.shuffle_u8s(lo.read(cpu)).store_unaligned(dst.cast::<u8>());
    window.shuffle_u8s(hi.read(cpu)).store_unaligned(dst.add(2).cast::<u8>());
}

#[cpu_version(Neon)]
pub(super) unsafe fn lanes_post_neon(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    src_len: u32,
    lanes: &mut Lanes,
    cpu: Neon,
) {
    // `tk!(Invalid)` is the byte-class default for stray/control bytes and
    // reaches the output as a 1-byte token. Track "any seen" alongside the
    // value sweep; localize cold.
    let mut inv = Vec128::zero(cpu);
    let mut i = 0usize;
    while i + 64 <= m {
        let v0 = Vec128::load_unaligned(out_kinds.add(i), cpu);
        let v1 = Vec128::load_unaligned(out_kinds.add(i + 16), cpu);
        let v2 = Vec128::load_unaligned(out_kinds.add(i + 32), cpu);
        let v3 = Vec128::load_unaligned(out_kinds.add(i + 48), cpu);
        inv |= (v0.eq_u8(tk!(Invalid)) | v1.eq_u8(tk!(Invalid)))
            | (v2.eq_u8(tk!(Invalid)) | v3.eq_u8(tk!(Invalid)));
        let mut mask = mm64(hits(v0), hits(v1), hits(v2), hits(v3));
        while mask != 0 {
            emit_value(src, out_kinds, out_spans, i + mask.trailing_zeros() as usize, lanes, cpu);
            mask &= mask - 1;
        }
        i += 64;
    }
    let mut inv_dirty = !inv.is_zero();
    while i < m {
        let v0 = Vec128::load_unaligned(out_kinds.add(i), cpu);
        let v1 = Vec128::load_unaligned(out_kinds.add(i + 16), cpu);
        let mut mask = mm32(hits(v0), hits(v1));
        let mut invm = mm32(v0.eq_u8(tk!(Invalid)), v1.eq_u8(tk!(Invalid)));
        let rem = m - i;
        if rem < 32 {
            mask &= (1u32 << rem) - 1;
            invm &= (1u32 << rem) - 1;
        }
        inv_dirty |= invm != 0;
        while mask != 0 {
            emit_value(src, out_kinds, out_spans, i + mask.trailing_zeros() as usize, lanes, cpu);
            mask &= mask - 1;
        }
        i += 32;
    }
    if inv_dirty {
        invalid_diags(src, out_kinds, out_spans, m, src_len, lanes);
    }
}

/// Get a mask of the bytes of `v` which are token kinds that [`emit_value`] handles.
#[cpu_version(Neon)]
#[inline(always)]
fn hits(v: Vec128<Neon>) -> Vec128<Neon> {
    (v.eq_u8(tk!(Number)) | v.eq_u8(tk!(BigInt)))
        | (v.eq_u8(tk!(IdentEscaped)) | v.eq_u8(tk!(PrivateIdentEscaped)))
}
