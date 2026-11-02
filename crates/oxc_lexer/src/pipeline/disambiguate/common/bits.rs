//! Bit scans over the token bitmaps: one bit per source byte, 64 to a word.
//!
//! Safe counterparts of the `bitmap` primitives the pipeline stages run on raw pointers. Positions
//! run to `n`, the source length: a forward scan that finds nothing before `n` answers `n`, and a
//! backward one that reaches the start answers `None`.

use crate::cpu::{Cpu, cpu_multiversion};

/// Is bit `i` set?
#[cpu_multiversion]
#[inline(always)]
pub fn get(bits: &[u64], i: usize, _cpu: impl Cpu) -> bool {
    (bits[i >> 6] >> (i & 63)) & 1 != 0
}

/// The first set bit at or after `i`, or `n`.
#[cpu_multiversion]
#[inline(always)]
pub fn next1(bits: &[u64], i: usize, n: usize, _cpu: impl Cpu) -> usize {
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

/// The last set bit before `p`, or `None`.
#[cpu_multiversion]
#[inline(always)]
pub fn prev1(bits: &[u64], p: usize, _cpu: impl Cpu) -> Option<usize> {
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
