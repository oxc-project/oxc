use oxc_span::Span;

use crate::{
    cpu::Avx2,
    simd::avx2::{Vec128, Vec256},
    token::{TRIVIA_MAX, TRIVIA_MIN, is_trivia_byte, tk},
};

use crate::pipeline::lanes::Lanes;

use super::common::{emit_value, invalid_diags};

mod pair_luts;
use pair_luts::get_pair_luts;
pub(super) use pair_luts::init_pair_luts_avx2;

#[cfg(test)]
mod tests;

static QCOMPACT: [[u32; 8]; 16] = qcompact();
static BCOMPACT: [[u8; 16]; 256] = bcompact();

const fn qcompact() -> [[u32; 8]; 16] {
    let mut t = [[0u32; 8]; 16];
    let mut mask = 0usize;
    while mask < 16 {
        let (mut q, mut k) = (0usize, 0usize);
        while q < 4 {
            if mask & (1 << q) != 0 {
                t[mask][2 * k] = (2 * q) as u32;
                t[mask][2 * k + 1] = (2 * q + 1) as u32;
                k += 1;
            }
            q += 1;
        }
        mask += 1;
    }
    t
}

const fn bcompact() -> [[u8; 16]; 256] {
    let mut t = [[0x80u8; 16]; 256];
    let mut mask = 0usize;
    while mask < 256 {
        let (mut b, mut k) = (0usize, 0usize);
        while b < 8 {
            if mask & (1 << b) != 0 {
                t[mask][k] = b as u8;
                k += 1;
            }
            b += 1;
        }
        mask += 1;
    }
    t
}

#[inline(never)]
#[target_feature(enable = "avx2,bmi2,popcnt")]
pub(super) unsafe fn compress_blocks_avx2(
    st: *const u64,
    kind: *const u8,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    kinds: *mut u8,
    cpu: Avx2,
) -> usize {
    // SAFETY: `Lexer::new` calls `init_pair_luts`, which initializes `PairLuts` if the CPU supports AVX2
    let pair_luts = unsafe { get_pair_luts(cpu) };
    let lut0z = &pair_luts.lut0z;
    let lutpad = pair_luts.lutpad.as_ptr().cast::<u8>();

    let mut m = 0usize;
    for b in b0..b1 {
        let mword = *st.add(b);
        if mword == 0 {
            continue;
        }
        let base = b * 64;
        let o0: u32 = 0;
        let o1 = o0 + (mword & 0xffff).count_ones();
        let o2 = o1 + ((mword >> 16) & 0xffff).count_ones();
        let o3 = o2 + ((mword >> 32) & 0xffff).count_ones();
        let o4 = o3 + (mword >> 48).count_ones();
        let mut basev = Vec256::splat_u32(base as u32, cpu);
        macro_rules! pair {
            ($p:expr, $off:expr) => {{
                let sub0 = ((mword >> (16 * $p)) & 0xff) as usize;
                let sub1 = ((mword >> (16 * $p + 8)) & 0xff) as usize;
                let pc0 = sub0.count_ones() as usize;
                let row0 = Vec128::read_low_unaligned(&lut0z[sub0], cpu);
                let row1 = Vec128::load_unaligned(lutpad.add(sub1 * 32 + 8 - pc0), cpu);
                let ctrl = row0 | row1;
                let kw = Vec128::load_unaligned(kind.add(base + 16 * $p), cpu);
                let lo = basev.add_u32(Vec256::widen_u8_to_u32(ctrl));
                lo.store_unaligned(starts.add(m + $off as usize).cast::<u8>());
                let hi = basev.add_u32(Vec256::widen_u8_to_u32(ctrl.shr_bytes::<8>()));
                hi.store_unaligned(starts.add(m + $off as usize + 8).cast::<u8>());
                basev = basev.add_u32(16);
                kw.shuffle_u8s(ctrl)
            }};
        }
        let ka = pair!(0, o0);
        let kb = pair!(1, o1);
        ka.store_unaligned(kinds.add(m + o0 as usize));
        kb.store_unaligned(kinds.add(m + o1 as usize));
        let kc = pair!(2, o2);
        let kd = pair!(3, o3);
        kc.store_unaligned(kinds.add(m + o2 as usize));
        kd.store_unaligned(kinds.add(m + o3 as usize));
        m += o4 as usize;
    }
    m
}

#[target_feature(enable = "avx2,bmi2,popcnt")]
pub(super) unsafe fn build_spans_avx2(
    stage_kind: *const u8,
    stage_pos: *const u32,
    m: usize,
    spans: *mut Span,
    sig_kinds: *mut u8,
    cpu: Avx2,
) -> usize {
    let sp = spans.cast::<u64>();
    let mut w = 0usize;
    let mut j = 0usize;

    // AVX2-specific code
    while j + 8 <= m {
        let k8 = Vec128::load_low_unaligned(stage_kind.add(j), cpu);
        let triv = k8.sub_u8(TRIVIA_MIN).saturating_sub_u8(TRIVIA_MAX - TRIVIA_MIN).eq_u8(0);
        let drop = triv & !k8.eq_u8(tk!(Hashbang));
        let sig = !drop.movemask() & 0xff;

        let v0 = Vec256::load_unaligned(stage_pos.add(j).cast::<u8>(), cpu);
        let v1 = Vec256::load_unaligned(stage_pos.add(j + 1).cast::<u8>(), cpu);
        let lo = v0.unpack_lo_u32(v1);
        let hi = v0.unpack_hi_u32(v1);
        let a = lo.join_lo_halves(hi);
        let b = lo.join_hi_halves(hi);

        let ma = (sig & 0xf) as usize;
        let mb = (sig >> 4) as usize;
        let ca = a.shuffle_u32s(Vec256::read_unaligned(&QCOMPACT[ma], cpu));
        ca.store_unaligned(sp.add(w).cast::<u8>());
        let cb = b.shuffle_u32s(Vec256::read_unaligned(&QCOMPACT[mb], cpu));
        cb.store_unaligned(sp.add(w + ma.count_ones() as usize).cast::<u8>());

        let kc = k8.shuffle_u8s(Vec128::read_unaligned(&BCOMPACT[sig as usize], cpu));
        kc.store_low_unaligned(sig_kinds.add(w));
        w += sig.count_ones() as usize;
        j += 8;
    }

    // Same code as generic implementation
    while j < m {
        let k = *stage_kind.add(j);
        *sp.add(w) = stage_pos.add(j).cast::<u64>().read_unaligned();
        *sig_kinds.add(w) = k;
        w += usize::from(!is_trivia_byte(k) || k == tk!(Hashbang));
        j += 1;
    }

    w
}

#[target_feature(enable = "avx2,bmi2,popcnt")]
pub(super) unsafe fn lanes_post_avx2(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
    cpu: Avx2,
) {
    // `tk!(Invalid)` is the byte-class default for stray/control bytes and
    // reaches the output as a 1-byte token. Track "any seen" alongside the
    // value sweep; localize cold.
    let mut inv = Vec256::zero(cpu);
    let mut i = 0usize;
    while i + 64 <= m {
        let v0 = Vec256::load_unaligned(out_kinds.add(i), cpu);
        let v1 = Vec256::load_unaligned(out_kinds.add(i + 32), cpu);
        let h0 = hits(v0);
        let h1 = hits(v1);
        inv |= v0.eq_u8(tk!(Invalid)) | v1.eq_u8(tk!(Invalid));
        let mut mask = (h0.movemask() as u64) | ((h1.movemask() as u64) << 32);
        while mask != 0 {
            emit_value(src, out_kinds, out_spans, i + mask.trailing_zeros() as usize, lanes, cpu);
            mask &= mask - 1;
        }
        i += 64;
    }
    let mut inv_dirty = inv.movemask() != 0;
    while i < m {
        let v = Vec256::load_unaligned(out_kinds.add(i), cpu);
        let hit = hits(v);
        let mut mask = hit.movemask();
        let mut invm = v.eq_u8(tk!(Invalid)).movemask();
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
        invalid_diags(src, out_kinds, out_spans, m, nn, lanes);
    }
}

/// Get a mask of the bytes of `v` which are token kinds that [`emit_value`] handles.
#[inline(always)]
fn hits(v: Vec256) -> Vec256 {
    (v.eq_u8(tk!(Number)) | v.eq_u8(tk!(BigInt)))
        | (v.eq_u8(tk!(IdentEscaped)) | v.eq_u8(tk!(PrivateIdentEscaped)))
}
