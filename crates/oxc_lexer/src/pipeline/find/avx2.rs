use core::arch::x86_64::*;

use super::super::chunk::{load256, mm, veq};

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

macro_rules! finder {
    ($(#[$attr:meta])* $name:ident: $($needle:expr),+ $(,)?) => {
        $(#[$attr])*
        #[inline]
        pub unsafe fn $name(src: *const u8, n: usize, mut i: usize) -> usize {
            use core::arch::x86_64::_mm256_or_si256;
            use super::super::chunk::{load256, veq, mm};
            use super::avx2::vor;

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
pub(super) use finder;

/// OR-fold of `vpcmpeqb` results, associated as a tree.
macro_rules! vor {
    ($a:expr) => { $a };
    ($a:expr, $b:expr) => { _mm256_or_si256($a, $b) };
    ($a:expr, $b:expr, $($rest:expr),+) => {
        _mm256_or_si256(vor!($a, $b), vor!($($rest),+))
    };
}
pub(super) use vor;
