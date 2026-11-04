use crate::{
    cpu::{Neon, cpu_version},
    simd::{Vec128, Vec256, VecData128},
    token::tk,
};

use super::{
    luts::{
        DIGIT_MASK, DOT_MASK, KWINIT_MASK, MERGED_LUTS, OPCH_MASK, WORD_LUTS, WORD_MASK, WS_MASK,
    },
    punct1_luts::{PH_A, PH_B, PH_T},
};

struct Luts {
    wb_lo: Vec128<Neon>,
    wb_hi: Vec128<Neon>,
    mrg_lo: Vec128<Neon>,
    mrg_hi: Vec128<Neon>,
    ph_a: Vec128<Neon>,
    ph_b: Vec128<Neon>,
    ph_t: Vec256<Neon>,
}

/// One chunk's lanes, each 0xFF or 0x00 as mm64 requires, plus its kind bytes.
struct Classified {
    word: Vec128<Neon>,
    ws: Vec128<Neon>,
    digit: Vec128<Neon>,
    dot: Vec128<Neon>,
    kwinit: Vec128<Neon>,
    opch: Vec128<Neon>,
    misc: Vec128<Neon>,
    kind: Vec128<Neon>,
}

#[cpu_version(Neon)]
#[inline(never)]
pub(super) unsafe fn classify_neon(
    ts: bool,
    src: *const u8,
    n: usize,
    word: *mut u64,
    st: *mut u64,
    kwinit: *mut u64,
    opch: *mut u64,
    digit: *mut u64,
    dot: *mut u64,
    misc: *mut u64,
    kind: *mut u8,
    cpu: Neon,
) {
    let mrg_lo = if ts { MERGED_LUTS.lo_ts } else { MERGED_LUTS.lo };
    let l = Luts {
        wb_lo: WORD_LUTS.lo.read(cpu),
        wb_hi: WORD_LUTS.hi.read(cpu),
        mrg_lo: mrg_lo.read(cpu),
        mrg_hi: MERGED_LUTS.hi.read(cpu),
        ph_a: PH_A.read(cpu),
        ph_b: PH_B.read(cpu),
        ph_t: PH_T.read(cpu),
    };

    let mut cw: u64 = 0;
    let mut cs: u64 = 0;
    let mut b = 0usize;
    let mut i = 0usize;
    let nb_ceil = n.div_ceil(64);
    while b < nb_ceil {
        // The last block overreads up to 63 bytes into the caller-guaranteed zeroed PAD.
        let c0 = chunk(Vec128::load_unaligned(src.add(i), cpu), &l);
        let c1 = chunk(Vec128::load_unaligned(src.add(i + 16), cpu), &l);
        let c2 = chunk(Vec128::load_unaligned(src.add(i + 32), cpu), &l);
        let c3 = chunk(Vec128::load_unaligned(src.add(i + 48), cpu), &l);

        c0.kind.store_unaligned(kind.add(i));
        c1.kind.store_unaligned(kind.add(i + 16));
        c2.kind.store_unaligned(kind.add(i + 32));
        c3.kind.store_unaligned(kind.add(i + 48));

        let wordm = mm64(c0.word, c1.word, c2.word, c3.word);
        let wsm = mm64(c0.ws, c1.ws, c2.ws, c3.ws);
        *word.add(b) = wordm;
        *digit.add(b) = mm64(c0.digit, c1.digit, c2.digit, c3.digit);
        *dot.add(b) = mm64(c0.dot, c1.dot, c2.dot, c3.dot);
        *kwinit.add(b) = mm64(c0.kwinit, c1.kwinit, c2.kwinit, c3.kwinit);
        *opch.add(b) = mm64(c0.opch, c1.opch, c2.opch, c3.opch);
        *misc.add(b) = mm64(c0.misc, c1.misc, c2.misc, c3.misc);

        let wprev = (wordm << 1) | cw;
        let sprev = (wsm << 1) | cs;
        cw = wordm >> 63;
        cs = wsm >> 63;
        *st.add(b) = (wsm & !sprev) | (wordm & !wprev) | (!wsm & !wordm);

        i += 64;
        b += 1;
    }
    // Zero PAD bytes classify as a token start, which would mint spurious tokens past n.
    let rem = n & 63;
    if rem != 0 {
        let last = nb_ceil - 1;
        let m = (1u64 << rem) - 1;
        *word.add(last) &= m;
        *st.add(last) &= m;
        *kwinit.add(last) &= m;
        *opch.add(last) &= m;
        *digit.add(last) &= m;
        *dot.add(last) &= m;
        *misc.add(last) &= m;
    }
}

#[cpu_version(Neon)]
#[inline(always)]
fn chunk(v: Vec128<Neon>, l: &Luts) -> Classified {
    // Mask keeps the index in TBL range; hi has no 8-15 entry, so bytes >= 0x80 give zero.
    let lo = v & 0x0f;
    let hn = v.shr_u8::<4>();
    let tb = l.wb_lo.shuffle_u8s(lo) & l.wb_hi.shuffle_u8s(hn);
    let t = l.mrg_lo.shuffle_u8s(lo) & l.mrg_hi.shuffle_u8s(hn);

    let na = v.test_u8(0x80);
    let word = tb.test_u8(WORD_MASK) | na;
    let ws = tb.test_u8(WS_MASK);
    let digit = tb.test_u8(DIGIT_MASK);
    let misc = v.eq_u8(b'#') | v.eq_u8(b'\\') | na;

    let kind = punct1(v, lo, hn, l)
        .blend(tk!(Whitespace), ws)
        .blend(tk!(Ident), word)
        .blend(tk!(Number), digit);

    Classified {
        word,
        ws,
        digit,
        dot: t.test_u8(DOT_MASK),
        kwinit: t.test_u8(KWINIT_MASK),
        opch: t.test_u8(OPCH_MASK),
        misc,
        kind,
    }
}

#[cpu_version(Neon)]
#[inline(always)]
fn punct1(v: Vec128<Neon>, lo: Vec128<Neon>, hn: Vec128<Neon>, l: &Luts) -> Vec128<Neon> {
    // Every PH_A and PH_B entry is under 32, so h is in range for the 32-entry ph_t.
    let h = l.ph_a.shuffle_u8s(lo) ^ l.ph_b.shuffle_u8s(hn);
    let ord = l.ph_t.shuffle_u8s(h);
    let ctl = v.lt_i8(0x20);
    ord.blend(tk!(Invalid), ctl)
}

/// Bit i is the top bit of byte i of a-d, gathered by summing the lane bit values.
#[cpu_version(Neon)]
#[inline(always)]
fn mm64(a: Vec128<Neon>, b: Vec128<Neon>, c: Vec128<Neon>, d: Vec128<Neon>) -> u64 {
    const BITS: VecData128 =
        VecData128::from_bytes([1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128]);

    let m = BITS.read(a.cpu());
    let ab = (a & m).pairwise_add_u8(b & m);
    let cd = (c & m).pairwise_add_u8(d & m);
    let s = ab.pairwise_add_u8(cd);
    s.pairwise_add_u8(s).get_u64::<0>()
}
