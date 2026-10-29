use crate::pipeline::chunk::{eqm, load64};

/// First byte at or after i that is one of needles, or n.
#[inline(always)]
pub unsafe fn find_generic<const K: usize>(
    src: *const u8,
    n: usize,
    mut i: usize,
    needles: [u8; K],
) -> usize {
    while i + 8 <= n {
        let x = load64(src, i);
        let mut m = 0u64;
        for c in needles {
            m |= eqm(x, c);
        }
        if m != 0 {
            return i + (m.trailing_zeros() >> 3) as usize;
        }
        i += 8;
    }
    while i < n {
        if needles.contains(&*src.add(i)) {
            return i;
        }
        i += 1;
    }
    n
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`): bit `i` for byte `base + i`.
#[inline]
pub fn bracket_bits_generic(src: &[u8], base: usize) -> u64 {
    let mut out = 0u64;
    for (i, &c) in src[base..base + 64].iter().enumerate() {
        if matches!(c, b'(' | b')' | b'[' | b']' | b'{' | b'}') {
            out |= 1u64 << i;
        }
    }
    out
}
