//! Helper functions for AVX2 and Neon tests.

use std::array;

/// Get `N` bytes which cover a spread of values, with and without the top bit set.
pub(super) fn pattern<const N: usize>(multiplier: u8, offset: u8) -> [u8; N] {
    array::from_fn(|i| u8::try_from(i).unwrap().wrapping_mul(multiplier).wrapping_add(offset))
}

/// Combine 2 arrays byte by byte.
pub(super) fn map2<const N: usize>(a: [u8; N], b: [u8; N], f: impl Fn(u8, u8) -> u8) -> [u8; N] {
    array::from_fn(|i| f(a[i], b[i]))
}
