//! Bit scans over the token bitmaps: one bit per source byte, 64 to a word.
//!
//! Safe counterparts of the `bitmap` primitives the pipeline stages run on raw pointers. Positions
//! run to `n`, the source length: a forward scan that finds nothing before `n` answers `n`, and a
//! backward one that reaches the start answers `None`.

/// Is bit `i` set?
#[inline(always)]
pub(crate) fn get(bits: &[u64], i: usize) -> bool {
    (bits[i >> 6] >> (i & 63)) & 1 != 0
}

/// The first set bit at or after `i`, or `n`.
#[inline(always)]
pub(crate) fn next1(bits: &[u64], i: usize, n: usize) -> usize {
    let mut w = i >> 6;
    let x = bits[w] & !((1u64 << (i & 63)).wrapping_sub(1));
    if x != 0 {
        return ((w << 6) + x.trailing_zeros() as usize).min(n);
    }
    w += 1;
    while (w << 6) < n {
        let x = bits[w];
        if x != 0 {
            return ((w << 6) + x.trailing_zeros() as usize).min(n);
        }
        w += 1;
    }
    n
}

/// The first clear bit at or after `i`, or `n`.
#[inline(always)]
pub(crate) fn next0(bits: &[u64], i: usize, n: usize) -> usize {
    let mut w = i >> 6;
    let mut inv = !bits[w] & !((1u64 << (i & 63)).wrapping_sub(1));
    while inv == 0 {
        w += 1;
        if (w << 6) >= n {
            return n;
        }
        inv = !bits[w];
    }
    ((w << 6) + inv.trailing_zeros() as usize).min(n)
}

/// The last set bit before `p`, or `None`.
#[inline(always)]
pub(crate) fn prev1(bits: &[u64], p: usize) -> Option<usize> {
    if p == 0 {
        return None;
    }
    let i = p - 1;
    let mut w = i >> 6;
    let lim = i & 63;
    let mask = if lim == 63 { !0u64 } else { (1u64 << (lim + 1)) - 1 };
    let x = bits[w] & mask;
    if x != 0 {
        return Some((w << 6) + (63 - x.leading_zeros() as usize));
    }
    while w > 0 {
        w -= 1;
        let x = bits[w];
        if x != 0 {
            return Some((w << 6) + (63 - x.leading_zeros() as usize));
        }
    }
    None
}
