use std::mem::MaybeUninit;

use oxc_simd::{Avx2, Vec128, Vec256, VecData256, cpu_version};
use oxc_span::Span;

use crate::token::{TRIVIA_MAX, TRIVIA_MIN, tk};

use crate::pipeline::{bitmap::Bitmap, kinds::Kinds, lanes::Lanes};

use super::{
    common::{BCOMPACT, emit_value, invalid_diags},
    generic::build_spans_generic,
    pair_luts::{PairLuts, get_pair_luts},
};

static QCOMPACT: [VecData256; 16] = {
    let mut t = [VecData256::from_u32s([0; 8]); 16];
    let mut mask = 0usize;
    while mask < 16 {
        let (mut q, mut k) = (0usize, 0usize);
        while q < 4 {
            if mask & (1 << q) != 0 {
                t[mask].as_u32s_mut()[2 * k] = (2 * q) as u32;
                t[mask].as_u32s_mut()[2 * k + 1] = (2 * q + 1) as u32;
                k += 1;
            }
            q += 1;
        }
        mask += 1;
    }
    t
};

#[cpu_version(Avx2)]
#[inline(never)]
pub(super) fn compress_blocks_avx2(
    token_start: Bitmap,
    kinds: Kinds,
    b0: usize,
    b1: usize,
    starts: &mut [u32],
    stage_kinds: &mut [u8],
    cpu: Avx2,
) -> usize {
    // Blocks b0..b1 must be in both handles, with 64 stage slots for each.
    assert!(b1 <= token_start.len().min(kinds.len() / 64));
    assert!(b0 <= b1 && b1 - b0 <= starts.len().min(stage_kinds.len()) / 64);
    let (st, kind) = (token_start.as_ptr(), kinds.as_ptr());
    let (starts, stage_kinds) = (starts.as_mut_ptr(), stage_kinds.as_mut_ptr());
    let luts = get_pair_luts();

    let mut m = 0usize;
    for b in b0..b1 {
        // SAFETY: block b is in both handles, and m <= 64 * (b - b0) leaves it 64 stage slots.
        unsafe {
            let mword = *st.add(b);
            if mword == 0 {
                continue;
            }

            let o0 = 0;
            let o1 = o0 + (mword & 0xffff).count_ones();
            let o2 = o1 + ((mword >> 16) & 0xffff).count_ones();
            let o3 = o2 + ((mword >> 32) & 0xffff).count_ones();
            let o4 = o3 + (mword >> 48).count_ones();

            let base = b * 64;
            let mut basev = Vec256::splat_u32(base as u32, cpu);

            let ka = compress_pair(0, mword, o0, luts, kind, base, starts, m, &mut basev, cpu);
            let kb = compress_pair(1, mword, o1, luts, kind, base, starts, m, &mut basev, cpu);
            ka.store_unaligned(stage_kinds.add(m + o0 as usize));
            kb.store_unaligned(stage_kinds.add(m + o1 as usize));
            let kc = compress_pair(2, mword, o2, luts, kind, base, starts, m, &mut basev, cpu);
            let kd = compress_pair(3, mword, o3, luts, kind, base, starts, m, &mut basev, cpu);
            kc.store_unaligned(stage_kinds.add(m + o2 as usize));
            kd.store_unaligned(stage_kinds.add(m + o3 as usize));

            m += o4 as usize;
        }
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
#[cpu_version(Avx2)]
#[inline(always)]
unsafe fn compress_pair(
    p: usize,
    mword: u64,
    off: u32,
    luts: &PairLuts,
    kind: *const u8,
    base: usize,
    starts: *mut u32,
    m: usize,
    basev: &mut Vec256<Avx2>,
    cpu: Avx2,
) -> Vec128<Avx2> {
    let sub0 = ((mword >> (16 * p)) & 0xff) as usize;
    let sub1 = ((mword >> (16 * p + 8)) & 0xff) as usize;
    let pc0 = sub0.count_ones() as usize;
    let row0 = Vec128::read_low_unaligned(&luts.lut0z[sub0], cpu);
    // SAFETY: pc0 counts the bits of a byte, so the 16 bytes from 8 - pc0 lie in the 32-byte row.
    let row1 = unsafe { Vec128::load_unaligned(luts.lutpad[sub1].as_ptr().add(8 - pc0), cpu) };
    let ctrl = row0 | row1;
    // SAFETY: the caller keeps the 16 kinds at base + 16 * p readable.
    let kw = unsafe { Vec128::load_unaligned(kind.add(base + 16 * p), cpu) };
    let lo = basev.add_u32(Vec256::widen_u8_to_u32(ctrl));
    let hi = basev.add_u32(Vec256::widen_u8_to_u32(ctrl.shr_bytes::<8>()));
    // SAFETY: the caller keeps the 16 slots at m + off writable.
    unsafe {
        lo.store_unaligned(starts.add(m + off as usize).cast::<u8>());
        hi.store_unaligned(starts.add(m + off as usize + 8).cast::<u8>());
    }
    *basev = basev.add_u32(16);
    kw.shuffle_u8s(ctrl)
}

/// Write the kept staged tokens to every slot below the returned count, 8 at a time.
#[cpu_version(Avx2)]
pub(super) fn build_spans_avx2(
    stage_kind: &[u8],
    stage_pos: &[u32],
    spans: &mut [MaybeUninit<Span>],
    sig_kinds: &mut [MaybeUninit<u8>],
    cpu: Avx2,
) -> usize {
    let m = stage_kind.len();
    // Each staged token needs the next one's start, and a slot each for its span and kind.
    assert!(stage_pos.len() > m && spans.len().min(sig_kinds.len()) >= m);
    let (sp, out) = (spans.as_mut_ptr().cast::<u64>(), sig_kinds.as_mut_ptr().cast::<u8>());
    let mut w = 0usize;
    let mut j = 0usize;

    while j + 8 <= m {
        // SAFETY: j + 8 <= m keeps the reads below m + 1, and the writes from w <= j below m.
        unsafe {
            let k8 = Vec128::load_low_unaligned(stage_kind.as_ptr().add(j), cpu);
            let triv = k8.sub_u8(TRIVIA_MIN).saturating_sub_u8(TRIVIA_MAX - TRIVIA_MIN).eq_u8(0);
            let drop = triv & !k8.eq_u8(tk!(Hashbang));
            let sig = !drop.movemask() & 0xff;

            let v0 = Vec256::load_unaligned(stage_pos.as_ptr().add(j).cast::<u8>(), cpu);
            let v1 = Vec256::load_unaligned(stage_pos.as_ptr().add(j + 1).cast::<u8>(), cpu);
            let lo = v0.unpack_lo_u32(v1);
            let hi = v0.unpack_hi_u32(v1);
            let a = lo.join_lo_halves(hi);
            let b = lo.join_hi_halves(hi);

            let ma = (sig & 0xf) as usize;
            let mb = (sig >> 4) as usize;
            let ca = a.shuffle_u32s(QCOMPACT[ma].read(cpu));
            ca.store_unaligned(sp.add(w).cast::<u8>());
            let cb = b.shuffle_u32s(QCOMPACT[mb].read(cpu));
            cb.store_unaligned(sp.add(w + ma.count_ones() as usize).cast::<u8>());

            let kc = k8.shuffle_u8s(BCOMPACT[sig as usize].read(cpu));
            kc.store_low_unaligned(out.add(w));
            w += sig.count_ones() as usize;
        }
        j += 8;
    }

    let (kinds, pos) = (&stage_kind[j..], &stage_pos[j..]);
    w + build_spans_generic(kinds, pos, &mut spans[w..], &mut sig_kinds[w..])
}

#[cpu_version(Avx2)]
pub(super) fn lanes_post_avx2(
    src: &[u8],
    kinds: &[u8],
    spans: &[Span],
    src_len: u32,
    lanes: &mut Lanes,
    cpu: Avx2,
) {
    // `tk!(Invalid)` is the byte-class default for stray/control bytes and
    // reaches the output as a 1-byte token. Track "any seen" alongside the
    // value sweep; localize cold.
    let m = kinds.len();
    let spans = &spans[..m];
    let mut inv = Vec256::zero(cpu);
    let mut i = 0usize;
    while i + 64 <= m {
        let v0 = Vec256::read_unaligned(kinds[i..].first_chunk::<32>().unwrap(), cpu);
        let v1 = Vec256::read_unaligned(kinds[i + 32..].first_chunk::<32>().unwrap(), cpu);
        let h0 = hits(v0);
        let h1 = hits(v1);
        inv |= v0.eq_u8(tk!(Invalid)) | v1.eq_u8(tk!(Invalid));
        let mut mask = (h0.movemask() as u64) | ((h1.movemask() as u64) << 32);
        while mask != 0 {
            emit_value(src, kinds, spans, i + mask.trailing_zeros() as usize, lanes, cpu);
            mask &= mask - 1;
        }
        i += 64;
    }
    let mut inv_dirty = inv.movemask() != 0;
    while i < m {
        // The last kinds, padded with Eof, which is neither a value nor invalid.
        let mut tail = [tk!(Eof); 32];
        let rem = (m - i).min(32);
        tail[..rem].copy_from_slice(&kinds[i..i + rem]);
        let v = Vec256::read_unaligned(&tail, cpu);
        let mut mask = hits(v).movemask();
        inv_dirty |= v.eq_u8(tk!(Invalid)).movemask() != 0;
        while mask != 0 {
            emit_value(src, kinds, spans, i + mask.trailing_zeros() as usize, lanes, cpu);
            mask &= mask - 1;
        }
        i += 32;
    }
    if inv_dirty {
        invalid_diags(src, kinds, spans, src_len, lanes);
    }
}

/// Get a mask of the bytes of `v` which are token kinds that [`emit_value`] handles.
#[cpu_version(Avx2)]
#[inline(always)]
fn hits(v: Vec256<Avx2>) -> Vec256<Avx2> {
    (v.eq_u8(tk!(Number)) | v.eq_u8(tk!(BigInt)))
        | (v.eq_u8(tk!(IdentEscaped)) | v.eq_u8(tk!(PrivateIdentEscaped)))
}
