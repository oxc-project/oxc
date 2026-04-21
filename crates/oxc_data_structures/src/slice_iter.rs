//! Extension trait for slice iterators.
//!
//! Provides additional methods to inspect and advance iterators.
//!
//! See [`SliceIter`] and [`SliceIterMut`].

// All methods boil down to just a few instructions.
// https://godbolt.org/z/KrsTz9478
#![expect(clippy::inline_always)]

use std::{
    ptr::NonNull,
    slice::{Iter, IterMut},
};

use crate::assert_unchecked;

/// Extension trait for slice iterators.
#[expect(private_bounds)]
pub trait SliceIter<'slice, T>: ExactSizeIterator + AsRef<[T]> + Sealed {
    /// The type returned by `peek` method.
    type Peeked<'iter>
    where
        'slice: 'iter,
        Self: 'iter;

    /// Peek the next item in the iterator, without advancing it.
    fn peek(&self) -> Option<Self::Peeked<'_>>;

    /// Get the next item in the iterator, without advancing it.
    ///
    /// If testing for a specific value, `peek` is more efficient.
    /// `iter.peek() == Some(&b' ')` is less instructions than `iter.peek_copy() == Some(b' ')`.
    /// <https://godbolt.org/z/a4TzsdY95>
    ///
    /// Only available when `T` is `Copy`.
    fn peek_copy(&self) -> Option<T>
    where
        T: Copy;

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

    /// Advance iterator to end.
    #[inline(always)]
    fn advance_to_end(&mut self) {
        // This function boils down to just setting the current pointer to the end - 2 instructions.
        // https://godbolt.org/z/EceneefEe
        self.advance(self.len());
    }

    /// Get pointer to next item in the iterator.
    ///
    /// Pointer is only valid to read an item from if iterator is not empty.
    #[inline]
    fn ptr(&self) -> *const T {
        let slice = self.as_ref();
        slice.as_ptr()
    }

    /// Get pointer to after last item in the iterator.
    ///
    /// Pointer is the end bound of the slice, so is not valid for reads.
    #[inline]
    fn end_ptr(&self) -> *const T {
        let slice = self.as_ref();
        slice.as_ptr_range().end
    }
}

impl<'slice, T: 'slice> SliceIter<'slice, T> for Iter<'slice, T> {
    // `peek` method returns a reference which borrows the slice, not the iterator
    type Peeked<'iter>
        = &'slice T
    where
        'slice: 'iter;

    /// Peek the next item in the iterator, without advancing it.
    #[inline(always)]
    fn peek(&self) -> Option<&'slice T> {
        self.clone().next()
    }

    /// Get the next item in the iterator, without advancing it.
    ///
    /// If testing for a specific value, `peek` is more efficient.
    /// `iter.peek() == Some(&b' ')` is less instructions than `iter.peek_copy() == Some(b' ')`.
    /// <https://godbolt.org/z/a4TzsdY95>
    ///
    /// Only available when `T` is `Copy`.
    #[inline]
    fn peek_copy(&self) -> Option<T>
    where
        T: Copy,
    {
        self.peek().copied()
    }

    /// Get next item without checking that iterator is not empty.
    ///
    /// Equivalent to [`Iterator::next`] but does not check that iterator is not exhausted,
    /// and therefore does not return an `Option`.
    ///
    /// # SAFETY
    /// Iterator must not be empty.
    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> &'slice T {
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
        // https://godbolt.org/z/hPT7T6YjY

        // SAFETY: Caller guarantees there are at least `count` items remaining in the iterator
        let slice = unsafe { self.as_slice().get_unchecked(count..) };
        *self = slice.iter();
    }
}

impl<'slice, T: 'slice> SliceIter<'slice, T> for IterMut<'slice, T> {
    // `peek` method returns a reference which borrows the iterator
    type Peeked<'iter>
        = &'iter T
    where
        'slice: 'iter;

    /// Peek the next item in the iterator, without advancing it.
    #[inline(always)]
    fn peek(&self) -> Option<&T> {
        self.as_slice().first()
    }

    /// Get the next item in the iterator, without advancing it.
    ///
    /// If testing for a specific value, `peek` is more efficient.
    /// `iter.peek() == Some(&b' ')` is less instructions than `iter.peek_copy() == Some(b' ')`.
    /// <https://godbolt.org/z/a4TzsdY95>
    ///
    /// Only available when `T` is `Copy`.
    #[inline]
    fn peek_copy(&self) -> Option<T>
    where
        T: Copy,
    {
        self.peek().copied()
    }

    /// Get next item without checking that iterator is not empty.
    ///
    /// Equivalent to [`Iterator::next`] but does not check that iterator is not exhausted,
    /// and therefore does not return an `Option`.
    ///
    /// # SAFETY
    /// Iterator must not be empty.
    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> &'slice mut T {
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
        // https://godbolt.org/z/hPT7T6YjY

        // SAFETY: Caller guarantees there are at least `count` items remaining in the iterator
        #[expect(unstable_name_collisions)]
        let slice = unsafe { self.as_mut_slice().get_unchecked_mut(count..) };
        // Extend the lifetime of the slice to `'slice`.
        // SAFETY: This method takes `&mut self`, so no other references to remaining items
        // in the iterator can exist, so no aliasing violations are possible.
        // We immediately use this slice to create a new iterator, and the lifetime-extended slice
        // does not escape this method.
        let slice = unsafe { NonNull::from(slice).as_mut() };
        *self = slice.iter_mut();
    }
}

/// Extension trait for [`IterMut`] slice iterator.
pub trait SliceIterMut<'slice, T>: SliceIter<'slice, T> {
    /// Get the remaining items in the iterator as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [T];
}

impl<'slice, T: 'slice> SliceIterMut<'slice, T> for IterMut<'slice, T> {
    /// Get the remaining items in the iterator as a mutable slice.
    ///
    /// This method is present in standard library, but requires nightly Rust.
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: We create a temporary copy of the iterator, and convert it to a `&mut [T]` slice,
        // but we don't hold a slice from the original iterator at the same time, and this method takes
        // `&mut self`, so no other references to iterator's remaining items can exist.
        // The returned slice borrows the iterator, so the iterator cannot be used to create any other
        // references to items while the slice exists. Therefore, no aliasing violations are possible.
        //
        // `IterMut::into_slice` consumes and drops the copy of the iterator, but `IterMut` does not
        // implement `Drop`, so that's not a problem.
        unsafe {
            let iter_copy = NonNull::from(self).read();
            iter_copy.into_slice()
        }
    }
}

/// Private trait.
/// [`SliceIter`] extends `Sealed`, which prevents code outside this file implementing
/// `SliceIter` on other types.
trait Sealed {}

impl<'slice, T: 'slice> Sealed for Iter<'slice, T> {}

impl<'slice, T: 'slice> Sealed for IterMut<'slice, T> {}

#[cfg(test)]
mod test_iter {
    use super::*;

    #[test]
    fn peek() {
        let mut iter = [11, 22, 33].iter();
        assert_eq!(iter.peek(), Some(&11));
        assert_eq!(iter.peek(), Some(&11));
        let first = iter.peek();

        iter.next();
        assert_eq!(iter.peek(), Some(&22));
        let second = iter.peek();

        iter.next();
        assert_eq!(iter.peek(), Some(&33));
        let third = iter.peek();

        iter.next();
        assert_eq!(iter.peek(), None);

        // Check peeked values don't borrow iterator
        assert_eq!(first, Some(&11));
        assert_eq!(second, Some(&22));
        assert_eq!(third, Some(&33));
    }

    #[test]
    fn peek_copy() {
        let mut iter = [11, 22, 33].iter();
        assert_eq!(iter.peek_copy(), Some(11));
        assert_eq!(iter.peek_copy(), Some(11));

        iter.next();
        assert_eq!(iter.peek_copy(), Some(22));

        iter.next();
        assert_eq!(iter.peek_copy(), Some(33));

        iter.next();
        assert_eq!(iter.peek_copy(), None);
    }

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

    #[test]
    fn advance_to_end() {
        let arr = [11, 22, 33];

        let mut iter = arr.iter();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter();
        assert_eq!(iter.next(), Some(&11));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter();
        assert_eq!(iter.next(), Some(&11));
        assert_eq!(iter.next(), Some(&22));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter();
        assert_eq!(iter.next(), Some(&11));
        assert_eq!(iter.next(), Some(&22));
        assert_eq!(iter.next(), Some(&33));
        assert_eq!(iter.next(), None);
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let empty_arr: [u32; 0] = [];
        let mut iter = empty_arr.iter();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ptr() {
        let slice = [11u32, 22, 33];
        let start_addr = slice.as_ptr() as usize;

        let mut iter = slice.iter();
        assert_eq!(iter.ptr() as usize, start_addr);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>());

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 2);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 3);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 3);
    }

    #[test]
    fn end_ptr() {
        let slice = [11u32, 22, 33];
        let end_addr = slice.as_ptr() as usize + size_of::<u32>() * 3;

        let mut iter = slice.iter();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);
    }
}

#[cfg(test)]
mod test_iter_mut {
    use super::*;

    #[test]
    fn peek() {
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();

        assert_eq!(iter.peek(), Some(&11));
        assert_eq!(iter.peek(), Some(&11));

        iter.next();
        assert_eq!(iter.peek(), Some(&22));

        iter.next();
        assert_eq!(iter.peek(), Some(&33));

        iter.next();
        assert_eq!(iter.peek(), None);
    }

    #[test]
    fn peek_copy() {
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();

        assert_eq!(iter.peek_copy(), Some(11));
        assert_eq!(iter.peek_copy(), Some(11));

        iter.next();
        assert_eq!(iter.peek_copy(), Some(22));

        iter.next();
        assert_eq!(iter.peek_copy(), Some(33));

        iter.next();
        assert_eq!(iter.peek_copy(), None);
    }

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

    #[test]
    fn advance_to_end() {
        let mut arr = [11, 22, 33];

        let mut iter = arr.iter_mut();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter_mut();
        assert_eq!(iter.next(), Some(&mut 11));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter_mut();
        assert_eq!(iter.next(), Some(&mut 11));
        assert_eq!(iter.next(), Some(&mut 22));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter_mut();
        assert_eq!(iter.next(), Some(&mut 11));
        assert_eq!(iter.next(), Some(&mut 22));
        assert_eq!(iter.next(), Some(&mut 33));
        assert_eq!(iter.next(), None);
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut empty_arr: [u32; 0] = [];
        let mut iter = empty_arr.iter_mut();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ptr() {
        let mut slice = [11u32, 22, 33];
        let start_addr = slice.as_ptr() as usize;

        let mut iter = slice.iter_mut();
        assert_eq!(iter.ptr() as usize, start_addr);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>());

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 2);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 3);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 3);
    }

    #[test]
    fn end_ptr() {
        let mut slice = [11u32, 22, 33];
        let end_addr = slice.as_ptr() as usize + size_of::<u32>() * 3;

        let mut iter = slice.iter_mut();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);
    }

    #[test]
    #[expect(unstable_name_collisions)]
    fn as_mut_slice() {
        let mut slice = [11u32, 22, 33];
        let mut iter = slice.iter_mut();

        assert_eq!(iter.as_mut_slice(), &mut [11, 22, 33]);

        assert_eq!(iter.next(), Some(&mut 11));
        assert_eq!(iter.as_mut_slice(), &mut [22, 33]);

        iter.as_mut_slice()[0] = 222;
        iter.as_mut_slice()[1] = 333;
        assert_eq!(iter.as_mut_slice(), &mut [222, 333]);

        assert_eq!(iter.next(), Some(&mut 222));
        assert_eq!(iter.as_mut_slice(), &mut [333]);

        assert_eq!(iter.next(), Some(&mut 333));
        assert_eq!(iter.as_mut_slice(), &mut []);

        assert_eq!(iter.next(), None);
    }
}
