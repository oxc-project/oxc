//! # A fast byte comparison library
//!
//! The library is intended to provide a faster byte slice comparison than the standard library.
//! Also raw string literals `b"like this"` are compareable this way.
//!
//! ## Example usage
//!
//! ```rust
//! use oxc_fastcmp::Compare;
//!
//! let vec = vec![1, 2, 3, 4, 5];
//! assert!(vec.feq(&[1, 2, 3, 4, 5]));
//! ```
//#![feature(i128_type)]
include!(concat!(env!("OUT_DIR"), "/compare.rs"));

// The pointer compare macro with offset support
macro_rules! cmp (
    ($left:expr, $right: expr, $var:ident, $offset:expr) => {
        unsafe {($left.offset($offset) as *const $var).read_unaligned() == ($right.offset($offset) as *const $var).read_unaligned()}
    }
);

/// Memory compare trait
pub trait Compare {
    /// Compares an `&[u8]` to another one
    fn feq(self: &Self, to: &Self) -> bool;
}

impl Compare for [u8] {
    #[inline(always)]
    fn feq(&self, to: &[u8]) -> bool {

        // Fallback if the slices are too large
        extern "C" {
            fn memcmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
        }

        // Get the comparison pointers
        let a = to.as_ptr() as *const i8;
        let b = self.as_ptr() as *const i8;
        let len = to.len();

        // Do the comparison
        self.len() == len && slice_compare!(a, b, len)
    }
}
