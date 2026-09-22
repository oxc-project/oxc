#[inline(always)]
pub fn get(bits: &[u64], i: usize) -> bool {
    (bits[i >> 6] >> (i & 63)) & 1 != 0
}

#[inline(always)]
pub fn next1(bits: &[u64], i: usize, n: usize) -> usize {
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

#[inline(always)]
pub fn next0(bits: &[u64], i: usize, n: usize) -> usize {
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

#[inline(always)]
pub fn prev1(bits: &[u64], p: usize) -> Option<usize> {
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
