use oxc_simd::{Neon, Vec128, Vec256, cpu_version};

use crate::{pipeline::chunk::mm64, token::tk};

use super::{
    Planes,
    luts::{
        DIGIT_MASK, DOT_MASK, KWINIT_MASK, MERGED_LUTS, OPCH_MASK, VT_FF_MASK, WORD_LUTS,
        WORD_MASK, WS_MASK,
    },
    punct::{PUNCT_HASH_HI, PUNCT_HASH_KINDS, PUNCT_HASH_LO},
};

struct Luts {
    wb_lo: Vec128<Neon>,
    wb_hi: Vec128<Neon>,
    mrg_lo: Vec128<Neon>,
    mrg_hi: Vec128<Neon>,
    punct_lo: Vec128<Neon>,
    punct_hi: Vec128<Neon>,
    punct_kinds: Vec256<Neon>,
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
pub(super) fn classify_neon(ts: bool, src: &[[u8; 64]], n: usize, out: Planes, cpu: Neon) {
    let mrg_lo = if ts { MERGED_LUTS.lo_ts } else { MERGED_LUTS.lo };
    let l = Luts {
        wb_lo: WORD_LUTS.lo.read(cpu),
        wb_hi: WORD_LUTS.hi.read(cpu),
        mrg_lo: mrg_lo.read(cpu),
        mrg_hi: MERGED_LUTS.hi.read(cpu),
        punct_lo: PUNCT_HASH_LO.read(cpu),
        punct_hi: PUNCT_HASH_HI.read(cpu),
        punct_kinds: PUNCT_HASH_KINDS.read(cpu),
    };

    let mut cw: u64 = 0;
    let mut cs: u64 = 0;
    let nb_ceil = n.div_ceil(64);
    let (src, out) = (&src[..nb_ceil], out.first(nb_ceil));
    let Planes {
        token_start: st,
        word,
        keyword_initial: kwinit,
        op_char: opch,
        digit,
        dot,
        misc,
        kinds: kind,
    } = out;
    for b in 0..nb_ceil {
        // The last block reads up to 63 bytes of PAD past n, masked out below.
        let (q, _) = src[b].as_chunks::<16>();
        let c0 = chunk(Vec128::read_unaligned(&q[0], cpu), &l);
        let c1 = chunk(Vec128::read_unaligned(&q[1], cpu), &l);
        let c2 = chunk(Vec128::read_unaligned(&q[2], cpu), &l);
        let c3 = chunk(Vec128::read_unaligned(&q[3], cpu), &l);

        let (k, _) = kind[b].as_chunks_mut::<16>();
        c0.kind.write_unaligned(&mut k[0]);
        c1.kind.write_unaligned(&mut k[1]);
        c2.kind.write_unaligned(&mut k[2]);
        c3.kind.write_unaligned(&mut k[3]);

        let wordm = mm64(c0.word, c1.word, c2.word, c3.word);
        let wsm = mm64(c0.ws, c1.ws, c2.ws, c3.ws);
        word[b] = wordm;
        digit[b] = mm64(c0.digit, c1.digit, c2.digit, c3.digit);
        dot[b] = mm64(c0.dot, c1.dot, c2.dot, c3.dot);
        kwinit[b] = mm64(c0.kwinit, c1.kwinit, c2.kwinit, c3.kwinit);
        opch[b] = mm64(c0.opch, c1.opch, c2.opch, c3.opch);
        misc[b] = mm64(c0.misc, c1.misc, c2.misc, c3.misc);

        let wprev = (wordm << 1) | cw;
        let sprev = (wsm << 1) | cs;
        cw = wordm >> 63;
        cs = wsm >> 63;
        // A byte starts a token unless it continues a whitespace or word run
        st[b] = !((wsm & sprev) | (wordm & wprev));
    }
    // Zero PAD bytes classify as a token start, which would mint spurious tokens past n.
    let rem = n & 63;
    if rem != 0 {
        let last = nb_ceil - 1;
        let m = (1u64 << rem) - 1;
        word[last] &= m;
        st[last] &= m;
        kwinit[last] &= m;
        opch[last] &= m;
        digit[last] &= m;
        dot[last] &= m;
        misc[last] &= m;
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
    let misc = v.eq_u8(b'#') | v.eq_u8(b'\\') | na | tb.test_u8(VT_FF_MASK);

    let kind = punct_kinds(v, lo, hn, l)
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
fn punct_kinds(v: Vec128<Neon>, lo: Vec128<Neon>, hn: Vec128<Neon>, l: &Luts) -> Vec128<Neon> {
    // Every `PUNCT_HASH_LO` and `PUNCT_HASH_HI` entry is under 32,
    // so `h` is in range for the 32-entry `l.punct_kinds`
    let h = l.punct_lo.shuffle_u8s(lo) ^ l.punct_hi.shuffle_u8s(hn);
    let ord = l.punct_kinds.shuffle_u8s(h);
    let ctl = v.lt_i8(0x20);
    ord.blend(tk!(Invalid), ctl)
}
