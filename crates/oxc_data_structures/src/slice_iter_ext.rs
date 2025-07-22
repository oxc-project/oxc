//! Extension trait for slice iterators.
//!
//! Provides additional methods to advance iterators.
//!
//! See [`SliceIterExt`].

// All methods boil down to just a few instructions.
// https://godbolt.org/z/779nYjq9d
#![expect(clippy::inline_always)]

use std::slice::{Iter, IterMut};

use crate::assert_unchecked;

/// Extension trait for slice iterators.
#[expect(private_bounds)]
pub trait SliceIterExt: ExactSizeIterator + Sealed {
    /// Get next item without checking that iterator is not empty.
    ///
    /// Equivalent to [`Iterator::next`] but does not check that iterator is not exhausted,
    /// and therefore does not return an `Option`.
    ///
    /// # SAFETY
    /// Iterator must not be empty.
    unsafe fn next_unchecked(&mut self) -> Self::Item;

    /// Advance iterator by `count` items.
    ///
    /// # Panics
    /// Panics if iterator does not contain at least `count` more items.
    #[inline]
    fn advance(&mut self, count: usize) {
        assert!(self.len() >= count, "Iterator does not have `count` items remaining");
        // SAFETY: Just checked iterator contains at least `count` more items
        unsafe { self.advance_unchecked(count) };
    }

    /// Advance iterator by `count` items, without bounds checks.
    ///
    /// # SAFETY
    /// Iterator must contain at least `count` more items.
    unsafe fn advance_unchecked(&mut self, count: usize);
}

impl<'a, T: 'a> SliceIterExt for Iter<'a, T> {
    /// Get next item without checking that iterator is not empty.
    ///
    /// Equivalent to [`Iterator::next`] but does not check that iterator is not exhausted,
    /// and therefore does not return an `Option`.
    ///
    /// # SAFETY
    /// Iterator must not be empty.
    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> &'a T {
        // Unchecked assertion removes the bounds check in `unwrap`.
        // SAFETY: Caller guarantees iterator is not empty.
        unsafe { assert_unchecked!(self.len() != 0) };
        self.next().unwrap()
    }

    /// Advance iterator by `count` items, without bounds checks.
    ///
    /// # SAFETY
    /// Iterator must contain at least `count` more items.
    #[inline(always)]
    unsafe fn advance_unchecked(&mut self, count: usize) {
        // This function boils down to just adding `count` to the current pointer (1 instruction).
        // SAFETY: Caller guarantees there are at least `count` items remaining in the iterator
        let slice = unsafe { self.as_slice().get_unchecked(count..) };
        *self = slice.iter();
    }
}

impl<'a, T: 'a> SliceIterExt for IterMut<'a, T> {
    /// Get next item without checking that iterator is not empty.
    ///
    /// Equivalent to [`Iterator::next`] but does not check that iterator is not exhausted,
    /// and therefore does not return an `Option`.
    ///
    /// # SAFETY
    /// Iterator must not be empty.
    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> &'a mut T {
        // Unchecked assertion removes the bounds check in `unwrap`.
        // SAFETY: Caller guarantees iterator is not empty.
        unsafe { assert_unchecked!(self.len() != 0) };
        self.next().unwrap()
    }

    /// Advance iterator by `count` items, without bounds checks.
    ///
    /// # SAFETY
    /// Iterator must contain at least `count` more items.
    #[inline(always)]
    unsafe fn advance_unchecked(&mut self, count: usize) {
        // This function boils down to 3 instructions including 1 branch, or 1 instruction
        // if `count` is statically known.
        // Unfortunately can't make this quite as efficient as `Iter::advance_unchecked` when `count`
        // is not statically known, because `IterMut::as_mut_slice` is not available on stable Rust.
        if count > 0 {
            // SAFETY: Caller guarantees there are at least `count` items remaining in the iterator
            unsafe { assert_unchecked!(self.len() >= count) };
            self.nth(count - 1);
        }
    }
}

/// Private trait.
/// [`SliceIterExt`] extends `Sealed`, which prevents code outside this file implementing
/// `SliceIterExt` on other types.
trait Sealed {}

impl<'a, T: 'a> Sealed for Iter<'a, T> {}

impl<'a, T: 'a> Sealed for IterMut<'a, T> {}

#[cfg(test)]
mod test_iter {
    use super::*;

    #[test]
    fn next_unchecked() {
        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            assert_eq!(iter.next_unchecked(), &11);
            assert_eq!(iter.next_unchecked(), &22);
            assert_eq!(iter.next_unchecked(), &33);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn advance() {
        let mut iter = [11, 22, 33].iter();
        iter.advance(0);
        assert_eq!(iter.next(), Some(&11));

        let mut iter = [11, 22, 33].iter();
        iter.advance(1);
        assert_eq!(iter.next(), Some(&22));

        let mut iter = [11, 22, 33].iter();
        iter.advance(2);
        assert_eq!(iter.next(), Some(&33));

        let mut iter = [11, 22, 33].iter();
        iter.advance(3);
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic(expected = "Iterator does not have `count` items remaining")]
    fn advance_panic() {
        let mut iter = [11, 22, 33].iter();
        iter.advance(4);
    }

    #[test]
    fn advance_unchecked() {
        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(0);
            assert_eq!(iter.next(), Some(&11));
        }

        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(1);
            assert_eq!(iter.next(), Some(&22));
        }

        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(2);
            assert_eq!(iter.next(), Some(&33));
        }

        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(3);
            assert_eq!(iter.next(), None);
        }
    }
}

#[cfg(test)]
mod test_iter_mut {
    use super::*;

    #[test]
    fn next_unchecked() {
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            assert_eq!(iter.next_unchecked(), &11);
            assert_eq!(iter.next_unchecked(), &22);
            assert_eq!(iter.next_unchecked(), &33);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn advance() {
        let mut arr = [11, 22, 33];

        let mut iter = arr.iter_mut();
        iter.advance(0);
        assert_eq!(iter.next(), Some(&mut 11));

        let mut iter = arr.iter_mut();
        iter.advance(1);
        assert_eq!(iter.next(), Some(&mut 22));

        let mut iter = arr.iter_mut();
        iter.advance(2);
        assert_eq!(iter.next(), Some(&mut 33));

        let mut iter = arr.iter_mut();
        iter.advance(3);
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic(expected = "Iterator does not have `count` items remaining")]
    fn advance_panic() {
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();
        iter.advance(4);
    }

    #[test]
    fn advance_unchecked() {
        let mut arr = [11, 22, 33];

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(0);
            assert_eq!(iter.next(), Some(&mut 11));
        }

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(1);
            assert_eq!(iter.next(), Some(&mut 22));
        }

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(2);
            assert_eq!(iter.next(), Some(&mut 33));
        }

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(3);
            assert_eq!(iter.next(), None);
        }
    }
}
