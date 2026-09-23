use std::arch::x86_64::*;

use crate::pipeline::chunk::{load256, mm, veq};

#[inline]
pub unsafe fn find1(src: *const u8, n: usize, mut i: usize, a: u8) -> usize {
    while i + 32 <= n {
        let m = mm(veq(load256(src, i), a));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        if *src.add(i) == a {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find2(src: *const u8, n: usize, mut i: usize, a: u8, b: u8) -> usize {
    while i + 32 <= n {
        let v = load256(src, i);
        let m = mm(_mm256_or_si256(veq(v, a), veq(v, b)));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        let c = *src.add(i);
        if c == a || c == b {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find3(src: *const u8, n: usize, mut i: usize, a: u8, b: u8, c: u8) -> usize {
    while i + 32 <= n {
        let v = load256(src, i);
        let m = mm(_mm256_or_si256(_mm256_or_si256(veq(v, a), veq(v, b)), veq(v, c)));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        let ch = *src.add(i);
        if ch == a || ch == b || ch == c {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find4(src: *const u8, n: usize, mut i: usize, a: u8, b: u8, c: u8, d: u8) -> usize {
    while i + 32 <= n {
        let v = load256(src, i);
        let m = mm(_mm256_or_si256(
            _mm256_or_si256(veq(v, a), veq(v, b)),
            _mm256_or_si256(veq(v, c), veq(v, d)),
        ));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        let x = *src.add(i);
        if x == a || x == b || x == c || x == d {
            return i;
        }
        i += 1;
    }
    n
}

macro_rules! define_find_function {
    ($(#[$attr:meta])* $name:ident: $($needle:expr),+ $(,)?) => {
        $(#[$attr])*
        #[inline]
        pub unsafe fn $name(src: *const u8, n: usize, mut i: usize) -> usize {
            use std::arch::x86_64::_mm256_or_si256;
            use crate::pipeline::{find::avx2::vor, chunk::{load256, veq, mm}};

            while i + 32 <= n {
                let v = load256(src, i);
                let m = mm(vor!($(veq(v, $needle)),+));
                if m != 0 {
                    return i + m.trailing_zeros() as usize;
                }
                i += 32;
            }
            while i < n {
                let c = *src.add(i);
                if $(c == $needle)||+ {
                    return i;
                }
                i += 1;
            }
            n
        }
    };
}
pub(super) use define_find_function;

/// OR-fold of `vpcmpeqb` results, associated as a tree.
macro_rules! vor {
    ($a:expr) => { $a };
    ($a:expr, $b:expr) => { _mm256_or_si256($a, $b) };
    ($a:expr, $b:expr, $($rest:expr),+) => {
        _mm256_or_si256(vor!($a, $b), vor!($($rest),+))
    };
}
pub(super) use vor;

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`): bit `i` for byte `base + i`.
#[inline]
pub fn bracket_bits(src: &[u8], base: usize) -> u64 {
    let block = &src[base..base + 64];
    let mut out = 0u64;
    for half in 0..2 {
        // SAFETY: `block` holds 64 bytes, so a 32-byte load at offset 0 or 32 stays inside it;
        // the OR-fold touches no memory and needs only `avx2`, which this module's `#[cfg]`
        // guarantees.
        let v = unsafe { load256(block.as_ptr(), half * 32) };
        let m = unsafe {
            vor!(veq(v, b'('), veq(v, b')'), veq(v, b'['), veq(v, b']'), veq(v, b'{'), veq(v, b'}'))
        };
        out |= u64::from(mm(m)) << (half * 32);
    }
    out
}
