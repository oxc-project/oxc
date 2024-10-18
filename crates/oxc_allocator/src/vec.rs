//! Arena Vec.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/ast/src/arena.rs)

use std::{
    self,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    mem::ManuallyDrop,
    ops,
    ptr::NonNull,
};

use allocator_api2::vec;
use bumpalo::Bump;
#[cfg(any(feature = "serialize", test))]
use serde::{ser::SerializeSeq, Serialize, Serializer};

use crate::{Allocator, Box};

/// A `Vec` without [`Drop`], which stores its data in the arena allocator.
///
/// Should only be used for storing AST types.
///
/// Must NOT be used to store types which have a [`Drop`] implementation.
/// `T::drop` will NOT be called on the `Vec`'s contents when the `Vec` is dropped.
/// If `T` owns memory outside of the arena, this will be a memory leak.
///
/// Note: This is not a soundness issue, as Rust does not support relying on `drop`
/// being called to guarantee soundness.
#[derive(PartialEq, Eq)]
pub struct Vec<'alloc, T>(ManuallyDrop<vec::Vec<T, &'alloc Bump>>);

impl<'alloc, T> Vec<'alloc, T> {
    /// Constructs a new, empty `Vec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let arena = Allocator::default();
    ///
    /// let mut vec: Vec<i32> = Vec::new_in(&arena);
    /// assert!(vec.is_empty());
    /// ```
    #[inline]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        Self(ManuallyDrop::new(vec::Vec::new_in(allocator)))
    }

    /// Constructs a new, empty `Vec<T>` with at least the specified capacity
    /// with the provided allocator.
    ///
    /// The vector will be able to hold at least `capacity` elements without
    /// reallocating. This method is allowed to allocate for more elements than
    /// `capacity`. If `capacity` is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// minimum *capacity* specified, the vector will have a zero *length*.
    ///
    /// For `Vec<T>` where `T` is a zero-sized type, there will be no allocation
    /// and the capacity will always be `usize::MAX`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let arena = Allocator::default();
    ///
    /// let mut vec = Vec::with_capacity_in(10, &arena);
    ///
    /// // The vector contains no items, even though it has capacity for more
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 10);
    ///
    /// // These are all done without reallocating...
    /// for i in 0..10 {
    ///     vec.push(i);
    /// }
    /// assert_eq!(vec.len(), 10);
    /// assert_eq!(vec.capacity(), 10);
    ///
    /// // ...but this may make the vector reallocate
    /// vec.push(11);
    /// assert_eq!(vec.len(), 11);
    /// assert!(vec.capacity() >= 11);
    ///
    /// // A vector of a zero-sized type will always over-allocate, since no
    /// // allocation is necessary
    /// let vec_units = Vec::<()>::with_capacity_in(10, &arena);
    /// assert_eq!(vec_units.capacity(), usize::MAX);
    /// ```
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        Self(ManuallyDrop::new(vec::Vec::with_capacity_in(capacity, allocator)))
    }

    /// Create a new [`Vec`] whose elements are taken from an iterator and
    /// allocated in the given `allocator`.
    ///
    /// This is behaviorially identical to [`FromIterator::from_iter`].
    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self {
        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let capacity = hint.1.unwrap_or(hint.0);
        let mut vec = ManuallyDrop::new(vec::Vec::with_capacity_in(capacity, &**allocator));
        vec.extend(iter);
        Self(vec)
    }

    /// Converts the vector into [`Box<[T]>`][owned slice].
    ///
    /// Any excess capacity the vector has will not be included in the slice.
    /// The excess memory will be leaked in the arena (i.e. not reused by another allocation).
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    /// let mut v = Vec::with_capacity_in(10, &allocator);
    /// v.extend([1, 2, 3]);
    /// let b = v.into_boxed_slice();
    ///
    /// assert_eq!(&*b, &[1, 2, 3]);
    /// assert_eq!(b.len(), 3);
    /// ```
    ///
    /// [owned slice]: Box
    pub fn into_boxed_slice(self) -> Box<'alloc, [T]> {
        let inner = ManuallyDrop::into_inner(self.0);
        let slice = inner.leak();
        let ptr = NonNull::from(slice);
        // SAFETY: `ptr` points to a valid slice `[T]`.
        // `allocator_api2::vec::Vec::leak` consumes the inner `Vec` without dropping it.
        // Lifetime of returned `Box<'alloc, [T]>` is same as lifetime of consumed `Vec<'alloc, T>`,
        // so data in the `Box` must be valid for its lifetime.
        // `Vec` uniquely owned the data, and we have consumed the `Vec`, so the new `Box` has
        // unique ownership of the data (no aliasing).
        // `ptr` was created from a `&mut [T]`.
        unsafe { Box::from_non_null(ptr) }
    }
}

impl<'alloc, T> ops::Deref for Vec<'alloc, T> {
    type Target = vec::Vec<T, &'alloc Bump>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, T> ops::DerefMut for Vec<'alloc, T> {
    fn deref_mut(&mut self) -> &mut vec::Vec<T, &'alloc Bump> {
        &mut self.0
    }
}

impl<'alloc, T> IntoIterator for Vec<'alloc, T> {
    type IntoIter = <vec::Vec<T, &'alloc Bump> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        let inner = ManuallyDrop::into_inner(self.0);
        // TODO: `allocator_api2::vec::Vec::IntoIter` is `Drop`.
        // Wrap it in `ManuallyDrop` to prevent that.
        inner.into_iter()
    }
}

impl<'alloc, T> IntoIterator for &'alloc Vec<'alloc, T> {
    type IntoIter = std::slice::Iter<'alloc, T>;
    type Item = &'alloc T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'alloc, T> ops::Index<usize> for Vec<'alloc, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

// Unused right now.
// impl<'alloc, T> ops::IndexMut<usize> for Vec<'alloc, T> {
// fn index_mut(&mut self, index: usize) -> &mut Self::Output {
// self.0.index_mut(index)
// }
// }

#[cfg(any(feature = "serialize", test))]
impl<'alloc, T> Serialize for Vec<'alloc, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for e in self.0.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'alloc, T: Hash> Hash for Vec<'alloc, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for e in self.0.iter() {
            e.hash(state);
        }
    }
}

impl<'alloc, T: Debug> Debug for Vec<'alloc, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = &*self.0;
        f.debug_tuple("Vec").field(inner).finish()
    }
}

#[cfg(test)]
mod test {
    use super::Vec;
    use crate::{Allocator, Box};

    #[test]
    fn vec_with_capacity() {
        let allocator = Allocator::default();
        let v: Vec<i32> = Vec::with_capacity_in(10, &allocator);
        assert!(v.is_empty());
    }

    #[test]
    fn vec_debug() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = format!("{v:?}");
        assert_eq!(v, "Vec([\"x\"])");
    }

    #[test]
    fn vec_serialize() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = serde_json::to_string(&v).unwrap();
        assert_eq!(v, "[\"x\"]");
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_vec_variant_lifetime<'a: 'b, 'b, T>(program: Vec<'a, T>) -> Vec<'b, T> {
            program
        }
    }

    #[test]
    fn vec_to_boxed_slice() {
        let allocator = Allocator::default();
        let mut v = Vec::with_capacity_in(10, &allocator);
        v.extend([1, 2, 3]);

        let b = v.into_boxed_slice();
        // Check return value is an `oxc_allocator::Box`, not an `allocator_api2::boxed::Box`
        let b: Box<[u8]> = b;

        assert_eq!(&*b, &[1, 2, 3]);
        // Check length of slice is equal to what `v.len()` was, not `v.capacity()`
        assert_eq!(b.len(), 3);
    }
}
